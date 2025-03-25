pub mod config;
pub mod error;
pub mod register;

use embedded_hal::digital::Error as DigitalError;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::Error as SpiError;
use embedded_hal_async::spi::SpiDevice;

use config::Config;
use error::Error;
use register::Register;
use register::{BURST, CHIP_ID, MAIN, SFCTL};

pub enum Variant {
    BGT60TR13C,
    BGT60UTR11AIP,
}

pub struct Radar<SPI, RST, IRQ, DLY> {
    spi: SPI,
    reset_pin: RST,
    interrupt_pin: IRQ,
    delay: DLY,
    variant: Variant,
    config: Config,
}

const READ_BIT: u8 = 0 << 7;
const WRITE_BIT: u8 = 1 << 7;

impl<SPI, RST, IRQ, DLY> Radar<SPI, RST, IRQ, DLY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    IRQ: Wait,
    DLY: DelayNs,
{
    pub fn new(
        variant: Variant,
        spi: SPI,
        reset_pin: RST,
        interrupt_pin: IRQ,
        delay: DLY,
        config: Config,
    ) -> Self {
        Radar {
            spi,
            reset_pin,
            interrupt_pin,
            delay,
            variant,
            config,
        }
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        self.reset_hw().await?;

        let chip_id = self.get_chip_id().await?;

        match self.variant {
            Variant::BGT60TR13C => {
                if chip_id.digital_id() != 3 && chip_id.rf_id() != 3 {
                    return Err(Error::ChipIdMismatch);
                }
            }
            Variant::BGT60UTR11AIP => {
                if chip_id.digital_id() != 7
                    && (chip_id.rf_id() != 7 || chip_id.rf_id() != 9 || chip_id.rf_id() != 12)
                {
                    return Err(Error::ChipIdMismatch);
                }
            }
        }

        // write initial configuration
        self.write_raw_registers(self.config.registers).await?;

        // set fifo limit
        self.set_fifo_limit(self.config.fifo_limit).await?;

        Ok(())
    }

    pub async fn configure(&mut self, config: Config) -> Result<(), Error> {
        // SW reset
        self.reset_sw().await?;

        self.config = config;

        // write registers
        self.write_raw_registers(self.config.registers).await?;

        // set fifo limit
        self.set_fifo_limit(self.config.fifo_limit).await?;

        Ok(())
    }

    async fn set_fifo_limit(&mut self, limit: u32) -> Result<(), Error> {
        // Check if limit is a power of two
        if limit % 2 != 0 {
            return Err(Error::NotAPowerOfTwo);
        }

        // Check if fifo is large enough
        match self.variant {
            Variant::BGT60TR13C => {
                if (limit / 2) > 8192 {
                    return Err(Error::FifoTooSmall(limit, 8192));
                }
            }
            Variant::BGT60UTR11AIP => {
                if (limit / 2) > 2048 {
                    return Err(Error::FifoTooSmall(limit, 2048));
                }
            }
        }

        let mut reg: SFCTL = self.read_register(register::Register::SFCTL).await?.into();
        reg.set_fifo_cref(((limit / 2) - 1) as usize);
        self.write_register(Register::SFCTL, reg.into()).await?;

        Ok(())
    }

    pub async fn reset_hw(&mut self) -> Result<(), Error> {
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

    pub async fn reset_sw(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_sw_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn reset_fifo(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_fifo_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn reset_fsm(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_fsm_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn get_chip_id(&mut self) -> Result<CHIP_ID, Error> {
        let reg = self.read_register(Register::CHIP_ID).await?;
        Ok(CHIP_ID::from(reg))
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_frame_start(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        self.reset_sw().await?;
        Ok(())
    }

    // TODO: make this a stream
    pub async fn get_fifo_data(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // ADC results are 12-bits, and two ADC results are packed into one 24-bit data block
        // FIFO has a limit of 8192 or 2048 24-bit data blocks, depending on the chip variant
        let needed_buffer_size = (self.config.fifo_limit as usize * 12) / 8;
        if buffer.len() != needed_buffer_size {
            return Err(Error::BufferWrongSize(buffer.len(), needed_buffer_size));
        }

        self.interrupt_pin
            .wait_for_high()
            .await
            .map_err(|e| Error::Gpio(e.kind()))?;

        let mut burst = BURST::new();
        burst.set_rw(true);
        match self.variant {
            Variant::BGT60TR13C => {
                burst.set_addr(register::Register::FIFO_TR13C as usize);
            }
            Variant::BGT60UTR11AIP => {
                burst.set_addr(register::Register::FIFO_UTR11 as usize);
            }
        }
        // RWB = 0, NBURSTS = 0
        let burst_raw: u32 = burst.into();
        buffer[0] = (burst_raw >> 24) as u8;
        buffer[1] = (burst_raw >> 16) as u8;
        buffer[2] = (burst_raw >> 8) as u8;
        buffer[3] = burst_raw as u8;

        self.spi
            .transfer_in_place(buffer)
            .await
            .map_err(|e| Error::Spi(e.kind()))?;

        // TODO use an ndarray instead of a u8 buffer and correctly unpack the 24-bit data blocks into 1, 2 or 3 12-bit ADC results depending on the number of active RX antennas

        Ok(())
    }

    async fn read_register(&mut self, reg: Register) -> Result<u32, Error> {
        let mut buffer: [u8; 4] = [reg as u8 | READ_BIT, 0, 0, 0];
        self.spi
            .transfer_in_place(&mut buffer)
            .await
            .map_err(|e| Error::Spi(e.kind()))?;
        // buffer[0] will contain GSR0 (Global Status)
        Ok(((buffer[1] as u32) << 16) | ((buffer[2] as u32) << 8) | (buffer[3] as u32))
    }

    async fn write_register(&mut self, reg: Register, data: u32) -> Result<(), Error> {
        let mut buffer: [u8; 4] = [
            reg as u8 | WRITE_BIT,
            ((data >> 16) & 0xFF) as u8,
            ((data >> 8) & 0xFF) as u8,
            (data & 0xFF) as u8,
        ];
        self.spi
            .transfer_in_place(&mut buffer)
            .await
            .map_err(|e| Error::Spi(e.kind()))
        // buffer[0] will contain GSR0 (Global Status)
        // If want, we could also verify that the write was successfull by comparing the data with buffer[1] to buffer[3]
    }

    async fn write_raw_registers(&mut self, registers: [u32; 38]) -> Result<(), Error> {
        for reg in registers {
            let mut buffer: [u8; 4] = [
                (((reg >> 24) & 0xFF) as u8 | WRITE_BIT),
                ((reg >> 16) & 0xFF) as u8,
                ((reg >> 8) & 0xFF) as u8,
                (reg & 0xFF) as u8,
            ];
            self.spi
                .transfer_in_place(&mut buffer)
                .await
                .map_err(|e| Error::Spi(e.kind()))?;
            // buffer[0] will contain GSR0 (Global Status)
        }
        Ok(())
    }
}
