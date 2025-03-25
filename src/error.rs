use core::fmt::{Debug, Display, Formatter, Result};
use embedded_hal::digital::ErrorKind as DigitalErrorKind;
use embedded_hal::spi::ErrorKind as SpiErrorKind;

use crate::register::GSR0;

#[derive(Debug)]
pub enum Error {
    Spi(SpiErrorKind),
    Gpio(DigitalErrorKind),
    NoConfigSet,
    VariantMismatch,
    NotAPowerOfTwo,
    FifoTooSmall(u32, u32),
    BufferWrongSize(usize, usize),
    GlobalStatusRegisterError(GSR0),
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::Spi(err) => write!(f, "SPI error: {}", err),
            Error::Gpio(err) => write!(f, "GPIO error: {}", err),
            Error::NoConfigSet => write!(f, "No configuration set"),
            Error::VariantMismatch => write!(f, "Variant does not match chip ID"),
            Error::FifoTooSmall(provided, max) => write!(f, "FIFO too small, provided: {}, max: {}", provided, max),
            Error::NotAPowerOfTwo => write!(f, "Value is not a power of two"),
            Error::BufferWrongSize(provided, expected) => write!(f, "Buffer wrong size, provided: {}, expected: {}", provided, expected),
            Error::GlobalStatusRegisterError(gsr0) => write!(f, "Global status register error: {:?}", gsr0),
        }
    }
}

impl core::error::Error for Error
{
}