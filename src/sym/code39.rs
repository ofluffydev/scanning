//! Encoder for Code39 barcodes.
//!
//! Code39 is a discrete, variable-length barcode. They are often referred to as "3-of-9".
//!
//! Code39 is the standard barcode used by the United States Department of Defense and is also
//! popular in non-retail environments. It was one of the first symbologies to support encoding
//! of the ASCII alphabet.

use crate::error::Result;
use crate::sym::{helpers, Parse};
use core::ops::Range;
use helpers::{vec, Vec};

// Character -> Binary mappings for each of the 43 allowable character.
const CHARS: [(char, [u8; 12]); 43] = [
    ('0', [1, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 1]),
    ('1', [1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1]),
    ('2', [1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1]),
    ('3', [1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1]),
    ('4', [1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1]),
    ('5', [1, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1]),
    ('6', [1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1]),
    ('7', [1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1]),
    ('8', [1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1]),
    ('9', [1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1]),
    ('A', [1, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1]),
    ('B', [1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1]),
    ('C', [1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1]),
    ('D', [1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1]),
    ('E', [1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1]),
    ('F', [1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1]),
    ('G', [1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1]),
    ('H', [1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1]),
    ('I', [1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 1]),
    ('J', [1, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1]),
    ('K', [1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1]),
    ('L', [1, 0, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1]),
    ('M', [1, 1, 0, 1, 1, 0, 1, 0, 1, 0, 0, 1]),
    ('N', [1, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1]),
    ('O', [1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1]),
    ('P', [1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1]),
    ('Q', [1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1]),
    ('R', [1, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1]),
    ('S', [1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1]),
    ('T', [1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1]),
    ('U', [1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1]),
    ('V', [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1]),
    ('W', [1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1]),
    ('X', [1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1]),
    ('Y', [1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1]),
    ('Z', [1, 0, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1]),
    ('-', [1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1]),
    ('.', [1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1]),
    (' ', [1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1]),
    ('$', [1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1]),
    ('/', [1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1]),
    ('+', [1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1]),
    ('%', [1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1]),
];

// Code39 barcodes must start and end with the '*' special character.
const GUARD: [u8; 12] = [1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1];

/// The Code39 barcode type.
#[derive(Debug)]
pub struct Code39 {
    data: Vec<char>,
    /// Indicates whether to encode a checksum digit.
    pub checksum: bool,
}

impl Code39 {
    fn init(data: &str, checksum: bool) -> Result<Self> {
        Self::parse(data).map(|d| Self {
            data: d.chars().collect(),
            checksum,
        })
    }

    /// Creates a new barcode.
    ///
    /// # Errors
    ///
    /// Returns an `Error::Character` if the input contains invalid characters.
    /// Returns an `Error::Length` if the input length is outside the valid range.
    ///
    /// Returns Result<Code39, Error> indicating parse success.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Self> {
        Self::init(data.as_ref(), false)
    }

    /// Creates a new barcode with an appended check-digit, calculated using modulo-43.
    ///
    /// # Errors
    ///
    /// Returns an `Error::Character` if the input contains invalid characters.
    /// Returns an `Error::Length` if the input length is outside the valid range.
    ///
    /// Returns Result<Code39, Error> indicating parse success.
    pub fn with_checksum<T: AsRef<str>>(data: T) -> Result<Self> {
        Self::init(data.as_ref(), true)
    }

    /// Calculates the checksum character using a modulo-43 algorithm.
    fn checksum_char(&self) -> Option<char> {
        let get_char_pos = |&c| {
            CHARS
                .iter()
                .position(|t| t.0 == c)
                .expect("Character not found in CHARS mapping")
        };
        let indices = self.data.iter().map(&get_char_pos);
        let index = indices.sum::<usize>() % CHARS.len();

        CHARS.get(index).map(|&(c, _)| c)
    }

    fn checksum_encoding(&self) -> [u8; 12] {
        self.checksum_char()
            .map_or_else(|| panic!("Cannot compute checksum"), Self::char_encoding)
    }

    pub(crate) fn char_encoding(c: char) -> [u8; 12] {
        match CHARS.iter().find(|&ch| ch.0 == c) {
            Some(&(_, enc)) => enc,
            None => panic!("Unknown char: {c}"),
        }
    }

    // Encoded characters are separated by a single "narrow" bar in
    // Code39 barcodes.
    pub(crate) fn push_encoding(into: &mut Vec<u8>, from: [u8; 12]) {
        into.extend(from.iter().copied());
        into.push(0);
    }

    fn payload(&self) -> Vec<u8> {
        let mut enc = vec![0];

        for c in &self.data {
            Self::push_encoding(&mut enc, Self::char_encoding(*c));
        }

        if self.checksum {
            Self::push_encoding(&mut enc, self.checksum_encoding());
        }

        enc
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let guard = &GUARD[..];

        helpers::join_slices(&[guard, &self.payload()[..], guard][..])
    }
}

impl Parse for Code39 {
    fn valid_len() -> Range<u32> {
        1..256
    }

    fn valid_chars() -> Vec<char> {
        let (chars, _): (Vec<_>, Vec<_>) = CHARS.iter().copied().unzip();
        chars
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::sym::code39::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: &[u8]) -> String {
        let chars = v
            .iter()
            .map(|d| char::from_digit(u32::from(*d), 10).expect("Failed to convert digit to char"));
        chars.collect()
    }

    #[test]
    fn new_code39() {
        let code39 = Code39::new("12345");

        assert!(code39.is_ok());
    }

    #[test]
    fn invalid_data_code39() {
        let code39 = Code39::new("1212s");

        assert_eq!(
            code39.expect_err("Expected Error::Character"),
            Error::Character
        );
    }

    #[test]
    fn invalid_len_code39() {
        let code39 = Code39::new("");

        assert_eq!(code39.expect_err("Expected Error::Length"), Error::Length);
    }

    #[test]
    fn code39_encode() {
        let code391 = Code39::new("1234").expect("Failed to create Code39 for '1234'");
        let code392 = Code39::new("983RD512").expect("Failed to create Code39 for '983RD512'");
        let code393 = Code39::new("TEST8052").expect("Failed to create Code39 for 'TEST8052'");

        assert_eq!(
            collapse_vec(&code391.encode()),
            "10010110110101101001010110101100101011011011001010101010011010110100101101101"
        );
        assert_eq!(collapse_vec(&code392.encode()), "100101101101010110010110101101001011010110110010101011010101100101010110010110110100110101011010010101101011001010110100101101101");
        assert_eq!(collapse_vec(&code393.encode()), "100101101101010101101100101101011001010101101011001010101101100101101001011010101001101101011010011010101011001010110100101101101");
    }

    #[test]
    fn code39_encode_with_checksum() {
        let code391 = Code39::with_checksum("1234")
            .expect("Failed to create Code39 with checksum for '1234'");
        let code392 = Code39::with_checksum("983RD512")
            .expect("Failed to create Code39 with checksum for '983RD512'");

        assert_eq!(collapse_vec(&code391.encode()), "100101101101011010010101101011001010110110110010101010100110101101101010010110100101101101");
        assert_eq!(collapse_vec(&code392.encode()), "1001011011010101100101101011010010110101101100101010110101011001010101100101101101001101010110100101011010110010101101011011010010100101101101");
    }
}
