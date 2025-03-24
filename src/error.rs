use core::fmt::{Debug, Display, Formatter, Result};

use embedded_hal::digital::ErrorKind as DigitalErrorKind;
use embedded_hal::spi::ErrorKind as SpiErrorKind;

#[derive(Debug)]
pub enum Error {
    Spi(SpiErrorKind),
    Gpio(DigitalErrorKind),
    ChipIdMismatch,
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::Spi(err) => write!(f, "SPI error: {}", err),
            Error::Gpio(err) => write!(f, "GPIO error: {}", err),
            Error::ChipIdMismatch => write!(f, "Chip ID mismatch"),
        }
    }
}

impl core::error::Error for Error
{
}

// impl From<SpiErrorKind> for Error {
//     fn from(error: SpiErrorKind) -> Self {
//         Self::Spi(error)
//     }
// }

// impl From<DigitalErrorKind> for Error {
//     fn from(error: DigitalErrorKind) -> Self {
//         Self::Gpio(error)
//     }
// }