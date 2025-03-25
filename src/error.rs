use core::fmt::{Debug, Display, Formatter, Result};
use embedded_hal::digital::ErrorKind as DigitalErrorKind;
use embedded_hal::spi::ErrorKind as SpiErrorKind;

#[derive(Debug)]
pub enum Error {
    Spi(SpiErrorKind),
    Gpio(DigitalErrorKind),
    ChipIdMismatch,
    NotAPowerOfTwo,
    FifoTooSmall(u32, u32),
    BufferWrongSize(usize, usize),
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::Spi(err) => write!(f, "SPI error: {}", err),
            Error::Gpio(err) => write!(f, "GPIO error: {}", err),
            Error::ChipIdMismatch => write!(f, "Chip ID mismatch"),
            Error::FifoTooSmall(provided, max) => write!(f, "FIFO too small, provided: {}, max: {}", provided, max),
            Error::NotAPowerOfTwo => write!(f, "Value is not a power of two"),
            Error::BufferWrongSize(provided, expected) => write!(f, "Buffer wrong size, provided: {}, expected: {}", provided, expected),
        }
    }
}

impl core::error::Error for Error
{
}