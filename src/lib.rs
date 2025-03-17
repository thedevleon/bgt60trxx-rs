use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;
use register::Register;

pub mod config;
pub mod register;

pub struct Radar<SPI, RST, IRQ, DLY>
{
    spi: SPI,
    reset_pin: RST,
    interrupt_pin: IRQ,
    delay: DLY,
}

const READ_BIT: u8 = 0 << 7;
const WRITE_BIT: u8 = 1 << 7;

impl <SPI, RST, IRQ, DLY> Radar<SPI, RST, IRQ, DLY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    IRQ: Wait,
    DLY: DelayNs
{
    pub fn new(spi: SPI, reset_pin: RST, interrupt_pin: IRQ, delay: DLY) -> Self {
        Radar {
            spi,
            reset_pin,
            interrupt_pin,
            delay,
        }
    }

    pub async fn hw_reset(&mut self) -> () {
        self.delay.delay_ns(100).await; // T_CS_BRES = 100ns
        self.reset_pin.set_low().unwrap();
        self.delay.delay_ns(100).await; // T_RES = 100ns
        self.reset_pin.set_high().unwrap();
        self.delay.delay_ns(100).await; // T_CS_ARES = 100ns
    }

    pub async fn sw_reset(&mut self) -> Result<(), SPI::Error> {
        self.write_register(Register::MAIN, 0b0010).await?;
        self.delay.delay_ns(100).await;
        Ok(())
    }

    pub async fn fifo_reset(&mut self) -> Result<(), SPI::Error> {
        self.write_register(Register::MAIN, 0b1000).await
    }

    pub async fn fsm_reset(&mut self) -> Result<(), SPI::Error> {
        self.write_register(Register::MAIN, 0b0100).await
    }

    pub async fn start(&mut self) -> Result<(), SPI::Error> {
        self.write_register(Register::MAIN, 0b1).await
    }

    pub async fn get_chip_id(&mut self) -> Result<(u16, u8), SPI::Error> {
        let reg = self.read_register(Register::CHIP_ID).await?;
        // 23:8: DIGITAL_ID, 7:0: RF_ID
        Ok(((reg >> 8) as u16, reg as u8))
    }

    async fn read_register(&mut self, reg: Register) -> Result<u32, SPI::Error> {
        let mut buffer: [u8; 4] = [reg as u8 | READ_BIT, 0, 0, 0];
        self.spi.transfer_in_place(&mut buffer).await?;
        // buffer[0] will contain GSR0 (Global Status)
        Ok((buffer[1] as u32) << 16 | (buffer[2] as u32) << 8 | (buffer[3] as u32))
    }

    async fn write_register(&mut self, reg: Register, data: u32) -> Result<(), SPI::Error> {
        let mut buffer: [u8; 4] = [reg as u8 | WRITE_BIT, ((data >> 16) & 0xFF) as u8, ((data >> 8) & 0xFF) as u8, (data & 0xFF) as u8];
        self.spi.transfer_in_place(&mut buffer).await
        // buffer[0] will contain GSR0 (Global Status)
        // If want, we could also verify that the write was successfull by ccomparing the data with buffer[1] to buffer[3] 
    }
}
