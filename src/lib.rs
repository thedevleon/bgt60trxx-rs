#![deny(unsafe_code)]
#![no_std]
#![doc = include_str!("../README.md")]

pub mod config;
pub mod error;
pub mod register;

use embedded_hal::digital::Error as DigitalError;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::Error as SpiError;
use embedded_hal_async::spi::SpiDevice;

#[cfg(feature = "debug")]
use log::info;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use ndarray::prelude::*;

use config::Config;
use error::Error;
use register::Register;
use register::{BURST, CHIP_ID, GSR0, MAIN, SFCTL};

pub enum Variant {
    BGT60TR13C,
    // BGT60UTR13D has been omitted
    BGT60UTR11AIP,
}

pub struct Radar<SPI, RST, IRQ, DLY> {
    spi: SPI,
    reset_pin: RST,
    interrupt_pin: IRQ,
    delay: DLY,
    variant: Variant,
    config: Option<Config>,
}

const READ_BIT: u8 = 0;
const WRITE_BIT: u8 = 1;

impl<SPI, RST, IRQ, DLY> Radar<SPI, RST, IRQ, DLY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    IRQ: Wait,
    DLY: DelayNs,
{
    /// Initializes the radar by performing a hardware reset and checking that the chip ID matches the expected variant.
    pub async fn new(
        variant: Variant,
        spi: SPI,
        reset_pin: RST,
        interrupt_pin: IRQ,
        delay: DLY,
    ) -> Result<Self, Error> {
        let mut this = Radar {
            spi,
            reset_pin,
            interrupt_pin,
            delay,
            variant,
            config: None,
        };

        this.reset_hw().await?;

        // reset SFCTL register to default state
        let sfctl = SFCTL::default();
        this.write_register(Register::SFCTL, sfctl.into()).await?;

        let chip_id = this.get_chip_id().await?;

        match this.variant {
            //
            Variant::BGT60TR13C => {
                if chip_id.digital_id() != 3 && chip_id.rf_id() != 3 {
                    return Err(Error::VariantMismatch);
                }
            }
            Variant::BGT60UTR11AIP => {
                if (chip_id.digital_id() != 7
                    && (chip_id.rf_id() != 7 || chip_id.rf_id() != 9 || chip_id.rf_id() != 12))
                    && (chip_id.digital_id() != 8 && chip_id.rf_id() != 12)
                {
                    return Err(Error::VariantMismatch);
                }
            }
        }

        Ok(this)
    }

    /// Configures the radar.
    ///
    /// - Performs a software reset (clearing all registers)
    /// - Writes the raw registers as generated by the bgt60-configurator-cli.
    /// - Sets the FIFO limit to a single frame (number of samples per chirp * number of chirps per frame * number of RX antennas)
    /// - Once the FIFO limit is reached, the interrupt pin will be pulled high.
    ///
    /// ### FIFO considerations:
    /// - The FIFO limit is the number of 12-bit ADC results that can be stored in the FIFO.
    /// - The FIFO limit must be a power of two, because the 12-bit ADC results are stored in 24-bit data blocks.
    /// - The FIFO limit must not exceed the maximum number of 24-bit data blocks that can be stored in the FIFO.
    pub async fn configure(&mut self, config: Config) -> Result<(), Error> {
        // TODO checks we might want to do on the config (i.e. if RX antennas match the variant)

        let fifo_limit = config.get_fifo_limit() as u32;

        // Check if limit is a power of two
        if fifo_limit % 2 != 0 {
            return Err(Error::NotAPowerOfTwo);
        }

        // Check if fifo is large enough
        // We divide by two, because two 12-bit ADC results are packed into one 24-bit data block
        match self.variant {
            Variant::BGT60TR13C => {
                if (fifo_limit / 2) > 8192 {
                    return Err(Error::FifoTooSmall(fifo_limit, 8192));
                }
            }
            Variant::BGT60UTR11AIP => {
                if (fifo_limit / 2) > 2048 {
                    return Err(Error::FifoTooSmall(fifo_limit, 2048));
                }
            }
        }

        // SW reset
        self.reset_sw().await?;

        // Write registers
        // TODO: Parse the register address and convert to the enum so that we can just use self.write_register(reg, data)
        for reg in config.registers {
            let addr = ((reg & 0xFE000000) >> 25) as u8;
            let data = reg & 0x00FFFFFF;

            let mut buffer: [u8; 4] = [
                (addr << 1) | WRITE_BIT,
                ((data >> 16) & 0xFF) as u8,
                ((data >> 8) & 0xFF) as u8,
                (data & 0xFF) as u8,
            ];

            // TODO if register is SFCTL, disable MISO_HD_RD

            #[cfg(feature = "debug")]
            info!(
                "Write CONFIG register request:  {:#04X} {:#04X}{:02X}{:02X} - {:#010b} {:#010b}{:08b}{:08b}",
                addr, buffer[1], buffer[2], buffer[3], buffer[0], buffer[1], buffer[2], buffer[3]
            );

            self.spi
                .transfer_in_place(&mut buffer)
                .await
                .map_err(|e| Error::Spi(e.kind()))?;

            #[cfg(feature = "debug")]
            info!(
                "Write CONFIG register response: {:#04X} {:#04X}{:02X}{:02X} - {:#010b} {:#010b}{:08b}{:08b}",
                buffer[0],
                buffer[1],
                buffer[2],
                buffer[3],
                buffer[0],
                buffer[1],
                buffer[2],
                buffer[3]
            );

            let gsr0 = GSR0::from(buffer[0]);
            if gsr0.has_error() {
                return Err(Error::GlobalStatusRegisterError(gsr0));
            }
        }

        // Set FIFO limit to a single frame
        let mut reg: SFCTL = self.read_register(register::Register::SFCTL).await?.into();
        reg.set_fifo_cref(((fifo_limit / 2) - 1) as usize);
        self.write_register(Register::SFCTL, reg.into()).await?;

        self.config = Some(config);

        Ok(())
    }

    /// Resets the hardware by pulling the reset pin high (== reset) and then low (== normal operation).
    pub async fn reset_hw(&mut self) -> Result<(), Error> {
        self.reset_pin
            .set_high()
            .map_err(|e| Error::Gpio(e.kind()))?;
        self.delay.delay_ns(100).await; // T_CS_BRES = 100ns
        self.reset_pin
            .set_low()
            .map_err(|e| Error::Gpio(e.kind()))?;
        self.delay.delay_ns(100).await; // T_RES = 100ns
        self.reset_pin
            .set_high()
            .map_err(|e| Error::Gpio(e.kind()))?;
        self.delay.delay_ns(100).await; // T_CS_ARES = 100ns
        Ok(())
    }

    /// Resets the software state machine.
    ///
    /// - Resets all registers to default state
    /// - Resets all internal counters (e.g. shape, frame)
    /// - Perform FIFO reset
    /// - Performs FSM reset
    pub async fn reset_sw(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_sw_reset(true);
        self.write_register(Register::MAIN, reg.into()).await?;

        // A delay of 100ns is necessary after a SW reset
        self.delay.delay_ns(100).await;

        // Check if the reset was successful by reading the register again until the sw_reset bit is cleared
        // 5 tries should be enough, right?
        for n in 0..5 {
            let reg = self.read_register(Register::MAIN).await;

            if let Ok(reg) = reg {
                let main: MAIN = reg.into();
                if !main.sw_reset() {
                    break;
                }
            } else if n == 4 {
                return Err(Error::ResetError);
            } else {
                self.delay.delay_ms(10).await; // wait 10ms before trying again
            }
        }

        // A final delay of 10ms is present in the C SDK
        self.delay.delay_ms(10).await;

        Ok(())
    }

    /// Clears and resets the FIFO.
    ///
    /// - Resets the read and write pointers of the FIFO
    /// - Array content will not be reset, but cannot be read out
    /// - FIFO empty is signaled, filling status = 0
    /// - Resets register FSTAT
    /// - Performs an implicit FSM rese
    pub async fn reset_fifo(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_fifo_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    /// Resets the FSM, which stops the frame generation.
    ///
    /// - Resets FSM to deep sleep mode
    /// - Resets FSM internal counters for channel/shape set and timers
    /// - Resets STAT0 and STAT1 register
    /// - Reset PLL ramp start signal
    /// - Reset PA_ON
    /// - Terminates frame (shape and frame counters incremented although maybe not complete)
    pub async fn reset_fsm(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_fsm_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    /// Returns the chip ID of the radar, which consists of a digital ID and an RF ID.
    pub async fn get_chip_id(&mut self) -> Result<CHIP_ID, Error> {
        let reg = self.read_register(Register::CHIP_ID).await?;
        Ok(CHIP_ID::from(reg))
    }

    /// Enables a test mode, which will fill the FIFO with a test pattern after the start command.
    ///
    /// The test pattern can be verified with the [`crate::get_next_test_word()`] method.
    pub async fn enable_test_mode(&mut self) -> Result<(), Error> {
        let mut reg: SFCTL = self.read_register(Register::SFCTL).await?.into();
        reg.set_lfsr_en(true);
        self.write_register(Register::SFCTL, reg.into()).await
    }

    /// Starts the frame generation.
    ///
    /// FIFO will be filled with samples after this command.
    /// The interrupt pin will be pulled high when then fifo has reached the set limit.
    pub async fn start(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_frame_start(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    /// Stops the frame generation by resetting the FSM
    pub async fn stop(&mut self) -> Result<(), Error> {
        self.reset_fsm().await?;
        Ok(())
    }

    /// Gets the frame(s) from the FIFO and returns them as a 3D array
    /// with the shape of [rx_antennas, num_chirps_per_frame, num_samples_per_chirp].
    /// 
    /// This function requires the alloc feature, since it dynamically allocates memory for the frames.
    #[cfg(feature = "alloc")]
    pub async fn get_frames(&mut self) -> Result<Array3<u16>, Error> {
        let config = self.config.as_ref().ok_or(Error::NoConfigSet)?;

        let shape = (
            config.rx_antennas as usize, 
            config.num_chirps_per_frame as usize, 
            config.num_samples_per_chirp as usize
        );

        // the data is arranged in the raw buffer in the following way
        // 1 channel: aa aa aa aa
        // 2 channels: ab ab ab ab
        // 3 channels: ab ca bc ab
        let strides = 
        (
            1 as usize, // stride for rx_antennas (innermost dimension)
            config.rx_antennas as usize * config.num_samples_per_chirp as usize,  // stride for chirps
            config.rx_antennas as usize // stride for samples
        );

        let mut frames =  vec![0u16; config.get_fifo_limit()];
        let mut buffer: Vec<u8> = vec![0u8; config.get_u8_buffer_size()];

        self.get_fifo_data(&mut buffer, &mut frames).await?;

        // We can ignore the error, since we know the frames vector is the correct size
        Ok(Array3::from_shape_vec(shape.strides(strides), frames).unwrap())
    }

    /// Reads the data from the FIFO by performing a burst read of the FIFO register.
    /// The function will wait for the interrupt pin to be pulled high before reading the data.
    ///
    /// The buffer must be the correct size to hold a single frame.
    /// The size of the buffer can be calculated with the formula:
    ///
    /// `buffer_size = (num_samples_per_chirp * num_chirps_per_frame * rx_antennas * 12) / 8`
    /// `output_size = num_samples_per_chirp * num_chirps_per_frame * rx_antennas`
    ///
    pub async fn get_fifo_data(
        &mut self,
        buffer: &mut [u8],
        output: &mut [u16],
    ) -> Result<(), Error> {
        let config = self.config.as_ref().ok_or(Error::NoConfigSet)?;
        let needed_buffer_size = config.get_u8_buffer_size();
        let fifo_limit = config.get_fifo_limit();
        
        // FIFO has a limit of 8192 or 2048 24-bit data blocks, depending on the chip variant
        if buffer.len() != needed_buffer_size {
            return Err(Error::BufferWrongSize(buffer.len(), needed_buffer_size));
        }
        if output.len() != fifo_limit {
            return Err(Error::OutputWrongSize(output.len(), fifo_limit));
        }

        self.interrupt_pin
            .wait_for_high()
            .await
            .map_err(|e| Error::Gpio(e.kind()))?;

        // The C implementation has the burst command hardcoded to XENSIV_BGT60TRXX_SPI_BURST_MODE_CMD 0xFF000000
        // and only adds the address of the FIFO register to it
        // however, the datasheet specifies the ADDR to be 0x7F, not 0xFF
        let burst = BURST::new()
            .with_addr(0x7F)
            .with_rw(true)
            .with_saddr(match self.variant {
                Variant::BGT60TR13C => register::Register::FIFO_TR13C as usize,
                Variant::BGT60UTR11AIP => register::Register::FIFO_UTR11 as usize,
            })
            .with_rwb(false)
            .with_nbursts(0);

        let burst_raw: u32 = burst.into();
        buffer[0] = ((burst_raw >> 24) & 0xFF) as u8;
        buffer[1] = ((burst_raw >> 16) & 0xFF) as u8;
        buffer[2] = ((burst_raw >> 8) & 0xFF) as u8;
        buffer[3] = burst_raw as u8;

        #[cfg(feature = "debug")]
        info!(
            "Burst command: {:#04X}{:02X}{:02X}{:02X} - {:#010b}{:08b}{:08b}{:08b}",
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[0], buffer[1], buffer[2], buffer[3]
        );

        // The C implementation first sends the burst command, checks the returned GSR0, and then continues to burst read the data only if no error flags are set in GSR0
        // Since we don't have control over the CS line (which needs to stay low between burst command and burst read), we can't do that

        self.spi
            .transfer_in_place(buffer)
            .await
            .map_err(|e| Error::Spi(e.kind()))?;

        // We can however check the GSR0 after the burst read
        let gsr0 = GSR0::from(buffer[0]);
        if gsr0.has_error() || gsr0.fou_err() {
            return Err(Error::GlobalStatusRegisterError(gsr0));
        }

        // Unpack the raw buffer into the output buffer, converting from 12-bit to 16-bit
        let buffer_view = &buffer[4..]; // skip the first 4 bytes, which are the burst command and GSR0
        for (i, result) in output.iter_mut().enumerate() {
            let index = (i * 12) / 8; // this will round down
            let odd = i % 2 == 0; // odd means the data starts in the middle of the block, even means it starts at the beginning

            let value: u16 = if odd {
                // the first value we only need to shift 4 bits to the left
                // the second value needs to be shifted 4 bits to the right to shift out the adjacent next value
                ((buffer_view[index] as u16) << 4) | ((buffer_view[index + 1] as u16) >> 4)
            }               
            else {
                // the first value we need to ignore the first 4 bits, and shift the remaining 4 bits 8 bits to the left
                // the second value we can take as is
                (((buffer_view[index] as u16) & 0x0F) << 8) | (buffer_view[index + 1] as u16)
            };

            *result = value;
        }

        Ok(())
    }

    async fn read_register(&mut self, reg: Register) -> Result<u32, Error> {
        let mut buffer: [u8; 4] = [((reg as u8) << 1) | READ_BIT, 0, 0, 0];

        #[cfg(feature = "debug")]
        info!("Read register {:#04X} - {:#010b}", reg as u8, buffer[0]);

        self.spi
            .transfer_in_place(&mut buffer)
            .await
            .map_err(|e| Error::Spi(e.kind()))?;

        #[cfg(feature = "debug")]
        info!(
            "Read register response: {:#04X} {:#04X}{:02X}{:02X} - {:#010b} {:#010b}{:08b}{:08b}",
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[0], buffer[1], buffer[2], buffer[3]
        );

        let gsr0 = GSR0::from(buffer[0]);
        if gsr0.has_error() {
            Err(Error::GlobalStatusRegisterError(gsr0))
        } else {
            Ok(((buffer[1] as u32) << 16) | ((buffer[2] as u32) << 8) | (buffer[3] as u32))
        }
    }

    async fn write_register(&mut self, reg: Register, data: u32) -> Result<(), Error> {
        let mut buffer: [u8; 4] = [
            ((reg as u8) << 1) | WRITE_BIT,
            ((data >> 16) & 0xFF) as u8,
            ((data >> 8) & 0xFF) as u8,
            (data & 0xFF) as u8,
        ];

        #[cfg(feature = "debug")]
        info!(
            "Write register request:  {:#04X} {:#04X}{:02X}{:02X} - {:#010b} {:#010b}{:08b}{:08b}",
            reg as u8, buffer[1], buffer[2], buffer[3], buffer[0], buffer[1], buffer[2], buffer[3]
        );

        self.spi
            .transfer_in_place(&mut buffer)
            .await
            .map_err(|e| Error::Spi(e.kind()))?;

        #[cfg(feature = "debug")]
        info!(
            "Write register response: {:#04X} {:#04X}{:02X}{:02X} - {:#010b} {:#010b}{:08b}{:08b}",
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[0], buffer[1], buffer[2], buffer[3]
        );

        let gsr0 = GSR0::from(buffer[0]);
        if gsr0.has_error() {
            Err(Error::GlobalStatusRegisterError(gsr0))
        } else {
            Ok(())
        }
    }
}

/// Generates the next test word based on the current word.
///
/// To be used in conjunction with [`Radar::enable_test_mode()`].
pub fn get_next_test_word(current: u16) -> u16 {
    (current >> 1)
        | (((current << 11) ^ (current << 10) ^ (current << 9) ^ (current << 3)) & 0x0800)
}
