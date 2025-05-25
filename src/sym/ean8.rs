//! Encoder for EAN-8 barcodes.
//!
//! EAN-8 barcodes are EAN style barcodes for smaller packages on products like
//! cigaretts, chewing gum, etc where package space is limited.

use crate::error::{Error, Result};
use crate::sym::ean13::{ENCODINGS, LEFT_GUARD, MIDDLE_GUARD, RIGHT_GUARD};
use crate::sym::{helpers, Parse};
use core::char;
use core::ops::Range;
use helpers::{vec, Vec};

/// The EAN-8 barcode type.
#[derive(Debug)]
pub struct EAN8(Vec<u8>);

impl EAN8 {
    /// Creates a new barcode.
    ///
    /// # Errors
    /// Returns an `Error::Checksum` if the provided checksum digit is invalid.
    /// Returns an `Error::Character` if the input contains invalid characters.
    /// Returns an `Error::Length` if the input length is not valid.
    ///
    /// # Panics
    /// Panics if a character in the input cannot be converted to a digit.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Self> {
        let d = Self::parse(data.as_ref())?;
        #[allow(clippy::cast_possible_truncation)] // Safe: to_digit(10) returns values in 0..=9
        let digits: Vec<u8> = d
            .chars()
            .map(|c| {
                c.to_digit(10)
                    .expect("Failed to convert character to digit") as u8
            })
            .collect();

        let ean8 = Self(digits[0..7].to_vec());

        // If checksum digit is provided, check the checksum.
        if digits.len() == 8 && ean8.checksum_digit() != digits[7] {
            return Err(Error::Checksum);
        }

        Ok(ean8)
    }

    /// Calculates the checksum digit using a weighting algorithm.
    fn checksum_digit(&self) -> u8 {
        helpers::modulo_10_checksum(&self.0[..], false)
    }

    fn number_system_digits(&self) -> &[u8] {
        &self.0[0..2]
    }

    fn number_system_encoding(&self) -> Vec<u8> {
        let mut ns = vec![];

        for d in self.number_system_digits() {
            ns.extend(Self::char_encoding(0, *d).iter().copied());
        }

        ns
    }

    fn checksum_encoding(&self) -> [u8; 7] {
        Self::char_encoding(2, self.checksum_digit())
    }

    pub(crate) const fn char_encoding(side: usize, d: u8) -> [u8; 7] {
        ENCODINGS[side][d as usize]
    }

    fn left_digits(&self) -> &[u8] {
        &self.0[2..4]
    }

    fn right_digits(&self) -> &[u8] {
        &self.0[4..]
    }

    fn left_payload(&self) -> Vec<u8> {
        let slices: Vec<[u8; 7]> = self
            .left_digits()
            .iter()
            .map(|d| Self::char_encoding(0, *d))
            .collect();

        helpers::join_iters(slices.iter())
    }

    fn right_payload(&self) -> Vec<u8> {
        let slices: Vec<[u8; 7]> = self
            .right_digits()
            .iter()
            .map(|d| Self::char_encoding(2, *d))
            .collect();

        helpers::join_iters(slices.iter())
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        helpers::join_slices(
            &[
                &LEFT_GUARD[..],
                &self.number_system_encoding()[..],
                &self.left_payload()[..],
                &MIDDLE_GUARD[..],
                &self.right_payload()[..],
                &self.checksum_encoding()[..],
                &RIGHT_GUARD[..],
            ][..],
        )
    }
}

impl Parse for EAN8 {
    /// Returns the valid length of data acceptable in this type of barcode.
    fn valid_len() -> Range<u32> {
        7..8
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
    use crate::sym::ean8::*;
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
    fn new_ean8() {
        let ean8 = EAN8::new("1234567");

        assert!(ean8.is_ok());
    }

    #[test]
    fn invalid_data_ean8() {
        let ean8 = EAN8::new("1234er1");

        assert_eq!(
            ean8.expect_err("Expected an Error::Character but got None"),
            Error::Character
        );
    }

    #[test]
    fn invalid_len_ean8() {
        let ean8 = EAN8::new("1111112222222333333");

        assert_eq!(
            ean8.expect_err("Expected an Error::Length but got None"),
            Error::Length
        );
    }

    #[test]
    fn invalid_checksum_ean8() {
        let ean8 = EAN8::new("88023020");

        assert_eq!(
            ean8.expect_err("Expected an Error::Checksum but got None"),
            Error::Checksum
        );
    }

    #[test]
    fn ean8_encode() {
        let ean81 = EAN8::new("5512345").expect("Failed to create EAN8 barcode for '5512345'"); // Check digit: 7
        let ean82 = EAN8::new("9834651").expect("Failed to create EAN8 barcode for '9834651'"); // Check digit: 3

        assert_eq!(
            collapse_vec(&ean81.encode()),
            "1010110001011000100110010010011010101000010101110010011101000100101"
        );
        assert_eq!(
            collapse_vec(&ean82.encode()),
            "1010001011011011101111010100011010101010000100111011001101010000101"
        );
    }
}
