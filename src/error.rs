use core::error::Error;
use core::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug)]
pub enum RadarError<SPI, GPIO> {
    Spi(SPI),
    Gpio(GPIO),
    ChipIdError,
}

impl<SPI, GPIO> Display for RadarError<SPI, GPIO>
where
    SPI: Display,
    GPIO: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            RadarError::Spi(err) => write!(f, "SPI error: {}", err),
            RadarError::Gpio(err) => write!(f, "GPIO error: {}", err),
            RadarError::ChipIdError => write!(f, "Chip ID error"),
        }
    }
}

impl<SPI, GPIO> Error for RadarError<SPI, GPIO>
where
    SPI: Debug + Display,
    GPIO: Debug + Display,
{
}

impl<SPI, GPIO> From<embedded_hal::spi::ErrorKind> for RadarError<SPI, GPIO> 
where
    SPI: From<embedded_hal::spi::ErrorKind>,
{
    fn from(err: embedded_hal::spi::ErrorKind) -> Self {
        RadarError::Spi(SPI::from(err))
    }
}

impl<SPI, GPIO> From<embedded_hal::digital::ErrorKind> for RadarError<SPI, GPIO> 
where
    GPIO: From<embedded_hal::digital::ErrorKind>,
{
    fn from(err: embedded_hal::digital::ErrorKind) -> Self {
        RadarError::Gpio(GPIO::from(err))
    }
}