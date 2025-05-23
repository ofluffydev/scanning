//! Custom error types.

use core::fmt;
#[cfg(feature = "std")]
use std::error::Error as StdError;

/// The possible errors that can occur during barcode encoding and generation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// An invalid character found during encoding.
    Character,
    /// An invalid data length during encoding.
    Length,
    /// An error during barcode generation.
    Generate,
    /// Invalid checksum.
    Checksum,
    /// Invalid data.
    Conversion,
}

/// Alias-type for Result<T, `barcoders::error::Error`>.
pub type Result<T> = ::core::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Character => write!(f, "Barcode data is invalid"),
            Self::Length => write!(f, "Barcode data length is invalid"),
            Self::Generate => write!(f, "Could not generate barcode data"),
            Self::Checksum => write!(f, "Invalid checksum"),
            Self::Conversion => write!(f, "Invalid data conversion"),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for Error {}
