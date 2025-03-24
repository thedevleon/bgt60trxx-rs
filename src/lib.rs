pub mod config;
pub mod register;
pub mod error;

use config::Config;
use error::Error as RadarError;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;
use embedded_hal_async::spi::Error as SpiError;
use embedded_hal::digital::Error as DigitalError;

use register::{Register, SFCTL};
use register::{CHIP_ID, MAIN};

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
        spi: SPI,
        reset_pin: RST,
        interrupt_pin: IRQ,
        delay: DLY,
        variant: Variant,
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

    pub async fn init(&mut self) -> Result<(), RadarError> {
        let _ = self.hw_reset().await?;

        let chip_id = self.get_chip_id().await?;

        match self.variant {
            Variant::BGT60TR13C => {
                if chip_id.digital_id() != 3 && chip_id.rf_id() != 3 {
                    return Err(RadarError::ChipIdMismatch);
                }
            }
            Variant::BGT60UTR11AIP => {
                if chip_id.digital_id() != 7
                    && (chip_id.rf_id() != 7 || chip_id.rf_id() != 9 || chip_id.rf_id() != 12)
                {
                    return Err(RadarError::ChipIdMismatch);
                }
            }
        }

        // write initial configuration
        self.write_registers(self.config.registers).await?;

        // set fifo limit
        self.set_fifo_limit(self.config.fifo_limit).await?;

        Ok(())
    }

    pub async fn configure(&mut self, config: Config) -> Result<(), RadarError> {
        // SW reset
        self.sw_reset().await?;

        self.config = config;

        // write registers
        self.write_registers(self.config.registers).await?;

        // set fifo limit
        self.set_fifo_limit(self.config.fifo_limit).await?;

        Ok(())
    }

    async fn set_fifo_limit(&mut self, limit: u32) -> Result<(), RadarError> {
        // Check if limit is a power of two
        if limit % 2 != 0 {
            return Err(RadarError::NotAPowerOfTwo);
        }

        // Check if fifo is large enough
        match self.variant {
            Variant::BGT60TR13C => {
                if limit >= 8192 {
                    return Err(RadarError::FifoTooSmall);
                }
            }
            Variant::BGT60UTR11AIP => {
                if limit >= 2048 {
                    return Err(RadarError::FifoTooSmall);
                }
            }
        }

        let mut reg: SFCTL = self.read_register(register::Register::SFCTL).await?.into();
        reg.set_fifo_cref(((limit / 2) - 1) as usize);
        self.write_register(Register::SFCTL, reg.into()).await?;

        Ok(())
    }

    pub async fn hw_reset(&mut self) -> Result<(), RadarError> {
        self.delay.delay_ns(100).await; // T_CS_BRES = 100ns
        self.reset_pin.set_low().map_err(|e| RadarError::Gpio(e.kind()))?;
        self.delay.delay_ns(100).await; // T_RES = 100ns
        self.reset_pin.set_high().map_err(|e| RadarError::Gpio(e.kind()))?;
        self.delay.delay_ns(100).await; // T_CS_ARES = 100ns
        Ok(())
    }

    pub async fn sw_reset(&mut self) -> Result<(), RadarError> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_sw_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn fifo_reset(&mut self) -> Result<(), RadarError> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_fifo_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn fsm_reset(&mut self) -> Result<(), RadarError> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_fsm_reset(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn get_chip_id(&mut self) -> Result<CHIP_ID, RadarError> {
        let reg = self.read_register(Register::CHIP_ID).await?;
        Ok(CHIP_ID::from(reg))
    }

    pub async fn start(&mut self) -> Result<(), RadarError> {
        let mut reg: MAIN = self.read_register(Register::MAIN).await?.into();
        reg.set_frame_start(true);
        self.write_register(Register::MAIN, reg.into()).await
    }

    pub async fn stop(&mut self) -> Result<(), RadarError> {
        self.sw_reset().await?;
        Ok(())
    }

    // TODO: make this a stream
    pub async fn get_fifo_data(&mut self) -> Result<(), RadarError> {
        self.interrupt_pin.wait_for_high().await.map_err(|e| RadarError::Gpio(e.kind()))?;
        //TODO: read data from fifo via burst mode
        Ok(())
    }

    async fn read_register(&mut self, reg: Register) -> Result<u32, RadarError> {
        let mut buffer: [u8; 4] = [reg as u8 | READ_BIT, 0, 0, 0];
        self.spi.transfer_in_place(&mut buffer).await.map_err(|e| RadarError::Spi(e.kind()))?;
        // buffer[0] will contain GSR0 (Global Status)
        Ok((buffer[1] as u32) << 16 | (buffer[2] as u32) << 8 | (buffer[3] as u32))
    }

    async fn write_register(&mut self, reg: Register, data: u32) -> Result<(), RadarError> {
        let mut buffer: [u8; 4] = [
            reg as u8 | WRITE_BIT,
            ((data >> 16) & 0xFF) as u8,
            ((data >> 8) & 0xFF) as u8,
            (data & 0xFF) as u8,
        ];
        self.spi.transfer_in_place(&mut buffer).await.map_err(|e| RadarError::Spi(e.kind()))
        // buffer[0] will contain GSR0 (Global Status)
        // If want, we could also verify that the write was successfull by comparing the data with buffer[1] to buffer[3]
    }

    async fn write_registers(&mut self, registers: [u32; 38]) -> Result<(), RadarError> {
        // TODO
        Ok(())
    }
}
