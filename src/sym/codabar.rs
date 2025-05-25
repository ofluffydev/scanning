//! Encoder for Codabar barcodes.
//!
//! Codabar is a simple, self-checking symbology without a standard for a checksum digit.
//!
//! Codabar is used in the USA by `FedEx`, some Hospitals, and photo labs.
//!
//! Barcodes of this variant should start and end with either A, B, C, or D depending on
//! the industry.

use super::helpers::{vec, Vec};
use crate::error::{Error, Result};
use crate::sym::Parse;
use core::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Dash,
    Dollar,
    Colon,
    Slash,
    Point,
    Plus,
    A,
    B,
    C,
    D,
}

impl Unit {
    fn lookup(self) -> Vec<u8> {
        match self {
            Self::Zero => vec![1, 0, 1, 0, 1, 0, 0, 1, 1],
            Self::One => vec![1, 0, 1, 0, 1, 1, 0, 0, 1],
            Self::Two => vec![1, 0, 1, 0, 0, 1, 0, 1, 1],
            Self::Three => vec![1, 1, 0, 0, 1, 0, 1, 0, 1],
            Self::Four => vec![1, 0, 1, 1, 0, 1, 0, 0, 1],
            Self::Five => vec![1, 1, 0, 1, 0, 1, 0, 0, 1],
            Self::Six => vec![1, 0, 0, 1, 0, 1, 0, 1, 1],
            Self::Seven => vec![1, 0, 0, 1, 0, 1, 1, 0, 1],
            Self::Eight => vec![1, 0, 0, 1, 1, 0, 1, 0, 1],
            Self::Nine => vec![1, 1, 0, 1, 0, 0, 1, 0, 1],
            Self::Dash => vec![1, 0, 1, 0, 0, 1, 1, 0, 1],
            Self::Dollar => vec![1, 0, 1, 1, 0, 0, 1, 0, 1],
            Self::Colon => vec![1, 1, 0, 1, 0, 1, 1, 0, 1, 1],
            Self::Slash => vec![1, 1, 0, 1, 1, 0, 1, 0, 1, 1],
            Self::Point => vec![1, 1, 0, 1, 1, 0, 1, 1, 0, 1],
            Self::Plus => vec![1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1],
            Self::A => vec![1, 0, 1, 1, 0, 0, 1, 0, 0, 1],
            Self::B => vec![1, 0, 1, 0, 0, 1, 0, 0, 1, 1],
            Self::C => vec![1, 0, 0, 1, 0, 0, 1, 0, 1, 1],
            Self::D => vec![1, 0, 1, 0, 0, 1, 1, 0, 0, 1],
        }
    }

    const fn from_char(c: char) -> Option<Self> {
        match c {
            '0' => Some(Self::Zero),
            '1' => Some(Self::One),
            '2' => Some(Self::Two),
            '3' => Some(Self::Three),
            '4' => Some(Self::Four),
            '5' => Some(Self::Five),
            '6' => Some(Self::Six),
            '7' => Some(Self::Seven),
            '8' => Some(Self::Eight),
            '9' => Some(Self::Nine),
            '-' => Some(Self::Dash),
            '$' => Some(Self::Dollar),
            '/' => Some(Self::Slash),
            ':' => Some(Self::Colon),
            '.' => Some(Self::Point),
            '+' => Some(Self::Plus),
            'A' => Some(Self::A),
            'B' => Some(Self::B),
            'C' => Some(Self::C),
            'D' => Some(Self::D),
            _ => None,
        }
    }
}

/// The Codabar barcode type.
#[derive(Debug)]
pub struct Codabar(Vec<Unit>);

impl Codabar {
    /// Creates a new barcode.
    ///
    /// # Errors
    /// Returns an `Error::Length` if the input string is empty or exceeds the valid length.
    /// Returns an `Error::Character` if the input string contains invalid characters.
    ///
    /// # Panics
    /// Panics if an invalid character is encountered during the conversion to `Unit`.
    ///
    /// Returns `Result<Codabar, Error>` indicating parse success.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Self> {
        let d = Self::parse(data.as_ref())?;
        let units = d
            .chars()
            .map(|c| Unit::from_char(c).ok_or(Error::Character))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self(units))
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut enc: Vec<u8> = vec![];

        for (i, u) in self.0.iter().enumerate() {
            enc.extend(u.lookup().iter().copied());

            if i < self.0.len() - 1 {
                enc.push(0);
            }
        }

        enc
    }
}

impl Parse for Codabar {
    /// Returns the valid length of data acceptable in this type of barcode.
    /// Codabar barcodes are variable-length.
    fn valid_len() -> Range<u32> {
        1..256
    }

    /// Returns the set of valid characters allowed in this type of barcode.
    fn valid_chars() -> Vec<char> {
        vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '/', '.', ':', '+', '$', 'A',
            'B', 'C', 'D',
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::sym::codabar::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: &[u8]) -> String {
        let chars = v.iter().map(|d| {
            char::from_digit(u32::from(*d), 10).expect("Failed to convert digit to character")
        });
        chars.collect()
    }

    #[test]
    fn invalid_length_codabar() {
        let codabar = Codabar::new("");

        assert_eq!(
            codabar.expect_err("Expected an Error::Length but got None"),
            Error::Length
        );
    }

    #[test]
    fn invalid_data_codabar() {
        let codabar = Codabar::new("A12345G");

        assert_eq!(
            codabar.expect_err("Expected an Error::Character but got None"),
            Error::Character
        );
    }

    #[test]
    fn codabar_encode() {
        let codabar_a =
            Codabar::new("A1234B").expect("Failed to create Codabar instance for 'A1234B'");
        let codabar_b =
            Codabar::new("A40156B").expect("Failed to create Codabar instance for 'A40156B'");

        assert_eq!(
            collapse_vec(&codabar_a.encode()),
            "1011001001010101100101010010110110010101010110100101010010011"
        );
        assert_eq!(
            collapse_vec(&codabar_b.encode()),
            "10110010010101101001010101001101010110010110101001010010101101010010011"
        );
    }
}
