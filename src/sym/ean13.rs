//! Encoder for EAN barcodes.
//!
//! EAN13 barcodes are very common in retail. 90% of the products you purchase from a supermarket
//! will use EAN13.
//!
//! This module defines types for:
//!   * EAN-13
//!   * Bookland
//!   * JAN

use crate::error::{Error, Result};
use crate::sym::{helpers, Parse};
use core::char;
use core::ops::Range;
use helpers::Vec;

/// Encoding mappings for EAN barcodes.
/// 1 = bar, 0 = no bar.
///
/// The three indices are:
/// * Left side A (odd parity).
/// * Left side B (even parity).
/// * Right side encodings.
pub const ENCODINGS: [[[u8; 7]; 10]; 3] = [
    [
        [0, 0, 0, 1, 1, 0, 1],
        [0, 0, 1, 1, 0, 0, 1],
        [0, 0, 1, 0, 0, 1, 1],
        [0, 1, 1, 1, 1, 0, 1],
        [0, 1, 0, 0, 0, 1, 1],
        [0, 1, 1, 0, 0, 0, 1],
        [0, 1, 0, 1, 1, 1, 1],
        [0, 1, 1, 1, 0, 1, 1],
        [0, 1, 1, 0, 1, 1, 1],
        [0, 0, 0, 1, 0, 1, 1],
    ],
    [
        [0, 1, 0, 0, 1, 1, 1],
        [0, 1, 1, 0, 0, 1, 1],
        [0, 0, 1, 1, 0, 1, 1],
        [0, 1, 0, 0, 0, 0, 1],
        [0, 0, 1, 1, 1, 0, 1],
        [0, 1, 1, 1, 0, 0, 1],
        [0, 0, 0, 0, 1, 0, 1],
        [0, 0, 1, 0, 0, 0, 1],
        [0, 0, 0, 1, 0, 0, 1],
        [0, 0, 1, 0, 1, 1, 1],
    ],
    [
        [1, 1, 1, 0, 0, 1, 0],
        [1, 1, 0, 0, 1, 1, 0],
        [1, 1, 0, 1, 1, 0, 0],
        [1, 0, 0, 0, 0, 1, 0],
        [1, 0, 1, 1, 1, 0, 0],
        [1, 0, 0, 1, 1, 1, 0],
        [1, 0, 1, 0, 0, 0, 0],
        [1, 0, 0, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 0, 0],
        [1, 1, 1, 0, 1, 0, 0],
    ],
];

/// Maps parity (odd/even) for the left-side digits based on the first digit in
/// the number system portion of the barcode data.
const PARITY: [[usize; 5]; 10] = [
    [0, 0, 0, 0, 0],
    [0, 1, 0, 1, 1],
    [0, 1, 1, 0, 1],
    [0, 1, 1, 1, 0],
    [1, 0, 0, 1, 1],
    [1, 1, 0, 0, 1],
    [1, 1, 1, 0, 0],
    [1, 0, 1, 0, 1],
    [1, 0, 1, 1, 0],
    [1, 1, 0, 1, 0],
];

/// The left-hand guard pattern.
pub const LEFT_GUARD: [u8; 3] = [1, 0, 1];
/// The middle guard pattern.
pub const MIDDLE_GUARD: [u8; 5] = [0, 1, 0, 1, 0];
/// The right-hand guard pattern.
pub const RIGHT_GUARD: [u8; 3] = [1, 0, 1];

/// The EAN-13 barcode type.
#[derive(Debug)]
pub struct EAN13(Vec<u8>);

/// The Bookland barcode type.
/// Bookland are EAN-13 that use number system 978.
pub type Bookland = EAN13;

/// The JAN barcode type.
/// JAN are EAN-13 that use number system 49.
pub type JAN = EAN13;

impl EAN13 {
    /// Creates a new barcode.
    ///
    /// # Errors
    /// Returns an `Error::Checksum` if the checksum digit is invalid.
    /// Returns an `Error::Character` if the input contains invalid characters.
    /// Returns an `Error::Length` if the input length is not valid.
    ///
    /// # Panics
    /// Panics if the input contains a character that cannot be converted to a digit.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Self> {
        let d = Self::parse(data.as_ref())?;
        #[allow(clippy::cast_possible_truncation)] // Safe: to_digit(10) returns values in 0..=9
        let digits: Vec<u8> = d
            .chars()
            .map(|c| c.to_digit(10).expect("Unknown character") as u8)
            .collect();

        let ean13 = Self(digits[0..12].to_vec());

        // If checksum digit is provided, check the checksum.
        if digits.len() == 13 && ean13.checksum_digit() != digits[12] {
            return Err(Error::Checksum);
        }

        Ok(ean13)
    }

    /// Calculates the checksum digit using a modulo-10 weighting algorithm.
    fn checksum_digit(&self) -> u8 {
        helpers::modulo_10_checksum(&self.0[..], true)
    }

    fn number_system_digit(&self) -> u8 {
        self.0[1]
    }

    fn number_system_encoding(&self) -> [u8; 7] {
        Self::char_encoding(0, self.number_system_digit())
    }

    fn checksum_encoding(&self) -> [u8; 7] {
        Self::char_encoding(2, self.checksum_digit())
    }

    const fn char_encoding(side: usize, d: u8) -> [u8; 7] {
        ENCODINGS[side][d as usize]
    }

    fn left_digits(&self) -> &[u8] {
        &self.0[2..7]
    }

    fn right_digits(&self) -> &[u8] {
        &self.0[7..]
    }

    fn parity_mapping(&self) -> [usize; 5] {
        PARITY[self.0[0] as usize]
    }

    fn left_payload(&self) -> Vec<u8> {
        let slices: Vec<[u8; 7]> = self
            .left_digits()
            .iter()
            .zip(self.parity_mapping().iter())
            .map(|(d, s)| Self::char_encoding(*s, *d))
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

impl Parse for EAN13 {
    /// Returns the valid length of data acceptable in this type of barcode.
    fn valid_len() -> Range<u32> {
        12..13
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
    use crate::sym::ean13::*;
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
    fn new_ean13() {
        let ean13 = EAN13::new("123456123456");

        assert!(ean13.is_ok());
    }

    #[test]
    fn new_bookland() {
        let bookland = Bookland::new("978456123456");

        assert!(bookland.is_ok());
    }

    #[test]
    fn invalid_data_ean13() {
        let ean13 = EAN13::new("1234er123412");

        assert_eq!(
            ean13.expect_err("Expected an Error::Character but got None"),
            Error::Character
        );
    }

    #[test]
    fn invalid_len_ean13() {
        let ean13 = EAN13::new("1111112222222333333");

        assert_eq!(
            ean13.expect_err("Expected an Error::Length but got None"),
            Error::Length
        );
    }

    #[test]
    fn invalid_checksum_ean13() {
        let ean13 = EAN13::new("8801051294881");

        assert_eq!(
            ean13.expect_err("Expected an Error::Checksum but got None"),
            Error::Checksum
        );
    }

    #[test]
    fn ean13_encode_as_bookland() {
        let bookland1 = Bookland::new("978345612345")
            .expect("Failed to create Bookland barcode with valid data"); // Check digit: 5
        let bookland2 = Bookland::new("978118999561")
            .expect("Failed to create Bookland barcode with valid data"); // Check digit: 5

        assert_eq!(collapse_vec(&bookland1.encode()), "10101110110001001010000101000110111001010111101010110011011011001000010101110010011101001110101");
        assert_eq!(collapse_vec(&bookland2.encode()), "10101110110001001011001100110010001001000101101010111010011101001001110101000011001101001110101");
    }

    #[test]
    fn ean13_encode() {
        let ean131 =
            EAN13::new("750103131130").expect("Failed to create EAN13 barcode with valid data"); // Check digit: 5
        let ean132 =
            EAN13::new("983465123499").expect("Failed to create EAN13 barcode with valid data"); // Check digit: 5

        assert_eq!(collapse_vec(&ean131.encode()), "10101100010100111001100101001110111101011001101010100001011001101100110100001011100101110100101");
        assert_eq!(collapse_vec(&ean132.encode()), "10101101110100001001110101011110111001001100101010110110010000101011100111010011101001000010101");
    }
}
