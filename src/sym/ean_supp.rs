//! Encoders for supplemental 2-digit and 5-digit EAN barcodes.
//!
//! EAN-2 barcodes are used in magazines and newspapers to indicate issue number.
//!
//! EAN-5 barcodes are often used to indicate the suggested retail price of books.
//!
//! These supplemental barcodes never appear without a full EAN-13 barcode alongside them.

use crate::error::{Error, Result};
use crate::sym::ean13::ENCODINGS;
use crate::sym::{helpers, Parse};
use core::char;
use core::ops::Range;
use helpers::{vec, Vec};

const LEFT_GUARD: [u8; 4] = [1, 0, 1, 1];

/// Maps parity (odd/even) for the EAN-5 barcodes based on the check digit.
const EAN5_PARITY: [[usize; 5]; 10] = [
    [0, 0, 1, 1, 1],
    [1, 0, 1, 0, 0],
    [1, 0, 0, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 0, 0],
    [0, 0, 1, 1, 0],
    [0, 0, 0, 1, 1],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 0, 1],
    [0, 0, 1, 0, 1],
];

/// Maps parity (odd/even) for the EAN-2 barcodes based on the check digit.
const EAN2_PARITY: [[usize; 5]; 4] = [
    [0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0],
    [1, 0, 0, 0, 0],
    [1, 1, 0, 0, 0],
];

/// The Supplemental EAN barcode type.
#[derive(Debug)]
pub enum EANSUPP {
    /// EAN-2 supplemental barcode type.
    EAN2(Vec<u8>),
    /// EAN-5 supplemental barcode type.
    EAN5(Vec<u8>),
}

impl EANSUPP {
    /// Creates a new barcode.
    ///
    /// Returns `Result<EANSUPP, Error>` indicating parse success.
    /// Either an `EAN2` or `EAN5` variant will be returned depending on
    /// the length of `data`.
    ///
    /// # Errors
    /// Returns `Error::Length` if the length of `data` is not 2 or 5.
    /// Returns `Error::Character` if `data` contains invalid characters.
    ///
    /// # Panics
    /// Panics if `data` contains a character that cannot be converted to a digit.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Self> {
        Self::parse(data.as_ref()).and_then(|d| {
            #[allow(clippy::cast_possible_truncation)] // Safe: to_digit(10) returns values in 0..=9
            let digits: Vec<u8> = d
                .chars()
                .map(|c| {
                    c.to_digit(10)
                        .expect("Failed to convert character to digit") as u8
                })
                .collect();

            match digits.len() {
                2 => Ok(Self::EAN2(digits)),
                5 => Ok(Self::EAN5(digits)),
                _ => Err(Error::Length),
            }
        })
    }

    fn raw_data(&self) -> &[u8] {
        match *self {
            Self::EAN2(ref d) | Self::EAN5(ref d) => &d[..],
        }
    }

    const fn char_encoding(side: usize, d: u8) -> [u8; 7] {
        ENCODINGS[side][d as usize]
    }

    /// Calculates the checksum digit using a modified modulo-10 weighting
    /// algorithm. This only makes sense for EAN5 barcodes.
    fn checksum_digit(&self) -> u8 {
        let mut odds = 0;
        let mut evens = 0;
        let data = self.raw_data();

        for (i, d) in data.iter().enumerate() {
            match i % 2 {
                1 => evens += *d,
                _ => odds += *d,
            }
        }

        match ((odds * 3) + (evens * 9)) % 10 {
            10 => 0,
            n => n,
        }
    }

    fn parity(&self) -> [usize; 5] {
        match *self {
            Self::EAN2(ref d) => {
                let modulo = ((d[0] * 10) + d[1]) % 4;
                EAN2_PARITY[modulo as usize]
            }
            Self::EAN5(ref _d) => {
                let check = self.checksum_digit() as usize;
                EAN5_PARITY[check]
            }
        }
    }

    fn payload(&self) -> Vec<u8> {
        let mut p = vec![];
        let slices: Vec<[u8; 7]> = self
            .raw_data()
            .iter()
            .zip(self.parity().iter())
            .map(|(d, s)| Self::char_encoding(*s, *d))
            .collect();

        for (i, d) in slices.iter().enumerate() {
            if i > 0 {
                p.push(0);
                p.push(1);
            }

            p.extend(d.iter().copied());
        }

        p
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        helpers::join_slices(&[&LEFT_GUARD[..], &self.payload()[..]][..])
    }
}

impl Parse for EANSUPP {
    /// Returns the valid length of data acceptable in this type of barcode.
    fn valid_len() -> Range<u32> {
        2..5
    }

    /// Returns the set of valid characters allowed in this type of barcode.
    fn valid_chars() -> Vec<char> {
        (0..10)
            .map(|i| char::from_digit(i, 10).expect("Failed to convert digit to character"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::sym::ean_supp::*;
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
    fn new_ean2() {
        let ean2 = EANSUPP::new("12");

        assert!(ean2.is_ok());
    }

    #[test]
    fn new_ean5() {
        let ean5 = EANSUPP::new("12345");

        assert!(ean5.is_ok());
    }

    #[test]
    fn invalid_data_ean2() {
        let ean2 = EANSUPP::new("AT");

        assert_eq!(
            ean2.expect_err("Expected Error::Character but got None"),
            Error::Character
        );
    }

    #[test]
    fn invalid_len_ean2() {
        let ean2 = EANSUPP::new("123");

        assert_eq!(
            ean2.expect_err("Expected Error::Length but got None"),
            Error::Length
        );
    }

    #[test]
    fn ean2_encode() {
        let ean21 = EANSUPP::new("34").expect("Failed to create EAN2 barcode from input '34'");

        assert_eq!(collapse_vec(&ean21.encode()), "10110100001010100011");
    }

    #[test]
    fn ean5_encode() {
        let ean51 =
            EANSUPP::new("51234").expect("Failed to create EAN5 barcode from input '51234'");

        assert_eq!(
            collapse_vec(&ean51.encode()),
            "10110110001010011001010011011010111101010011101"
        );
    }
}
