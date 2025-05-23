//! Encoder for Code93 barcodes.
//!
//! Code93 is intented to improve upon Code39 barcodes by offering a wider array of encodable
//! ASCII characters. It also produces denser barcodes than Code39.
//!
//! Code93 is a continuous, variable-length symbology.
//!
//! NOTE: This encoder currently only supports the basic Code93 implementation and not full-ASCII
//! mode.

use super::helpers::{vec, Vec};
use crate::error::Result;
use crate::sym::{helpers, Parse};
use core::ops::Range;

// Character -> Binary mappings for each of the 47 allowable character.
// The special "full-ASCII" characters are represented with (, ), [, ].
const CHARS: [(char, [u8; 9]); 47] = [
    ('0', [1, 0, 0, 0, 1, 0, 1, 0, 0]),
    ('1', [1, 0, 1, 0, 0, 1, 0, 0, 0]),
    ('2', [1, 0, 1, 0, 0, 0, 1, 0, 0]),
    ('3', [1, 0, 1, 0, 0, 0, 0, 1, 0]),
    ('4', [1, 0, 0, 1, 0, 1, 0, 0, 0]),
    ('5', [1, 0, 0, 1, 0, 0, 1, 0, 0]),
    ('6', [1, 0, 0, 1, 0, 0, 0, 1, 0]),
    ('7', [1, 0, 1, 0, 1, 0, 0, 0, 0]),
    ('8', [1, 0, 0, 0, 1, 0, 0, 1, 0]),
    ('9', [1, 0, 0, 0, 0, 1, 0, 1, 0]),
    ('A', [1, 1, 0, 1, 0, 1, 0, 0, 0]),
    ('B', [1, 1, 0, 1, 0, 0, 1, 0, 0]),
    ('C', [1, 1, 0, 1, 0, 0, 0, 1, 0]),
    ('D', [1, 1, 0, 0, 1, 0, 1, 0, 0]),
    ('E', [1, 1, 0, 0, 1, 0, 0, 1, 0]),
    ('F', [1, 1, 0, 0, 0, 1, 0, 1, 0]),
    ('G', [1, 0, 1, 1, 0, 1, 0, 0, 0]),
    ('H', [1, 0, 1, 1, 0, 0, 1, 0, 0]),
    ('I', [1, 0, 1, 1, 0, 0, 0, 1, 0]),
    ('J', [1, 0, 0, 1, 1, 0, 1, 0, 0]),
    ('K', [1, 0, 0, 0, 1, 1, 0, 1, 0]),
    ('L', [1, 0, 1, 0, 1, 1, 0, 0, 0]),
    ('M', [1, 0, 1, 0, 0, 1, 1, 0, 0]),
    ('N', [1, 0, 1, 0, 0, 0, 1, 1, 0]),
    ('O', [1, 0, 0, 1, 0, 1, 1, 0, 0]),
    ('P', [1, 0, 0, 0, 1, 0, 1, 1, 0]),
    ('Q', [1, 1, 0, 1, 1, 0, 1, 0, 0]),
    ('R', [1, 1, 0, 1, 1, 0, 0, 1, 0]),
    ('S', [1, 1, 0, 1, 0, 1, 1, 0, 0]),
    ('T', [1, 1, 0, 1, 0, 0, 1, 1, 0]),
    ('U', [1, 1, 0, 0, 1, 0, 1, 1, 0]),
    ('V', [1, 1, 0, 0, 1, 1, 0, 1, 0]),
    ('W', [1, 0, 1, 1, 0, 1, 1, 0, 0]),
    ('X', [1, 0, 1, 1, 0, 0, 1, 1, 0]),
    ('Y', [1, 0, 0, 1, 1, 0, 1, 1, 0]),
    ('Z', [1, 0, 0, 1, 1, 1, 0, 1, 0]),
    ('-', [1, 0, 0, 1, 0, 1, 1, 1, 0]),
    ('.', [1, 1, 1, 0, 1, 0, 1, 0, 0]),
    (' ', [1, 1, 1, 0, 1, 0, 0, 1, 0]),
    ('$', [1, 1, 1, 0, 0, 1, 0, 1, 0]),
    ('/', [1, 0, 1, 1, 0, 1, 1, 1, 0]),
    ('+', [1, 0, 1, 1, 1, 0, 1, 1, 0]),
    ('%', [1, 1, 0, 1, 0, 1, 1, 1, 0]),
    ('(', [1, 0, 0, 1, 0, 0, 1, 1, 0]),
    (')', [1, 1, 1, 0, 1, 1, 0, 1, 0]),
    ('[', [1, 1, 1, 0, 1, 0, 1, 1, 0]),
    (']', [1, 0, 0, 1, 1, 0, 0, 1, 0]),
];

// Code93 barcodes must start and end with the '*' special character.
const GUARD: [u8; 9] = [1, 0, 1, 0, 1, 1, 1, 1, 0];
const TERMINATOR: [u8; 1] = [1];

/// The Code93 barcode type.
#[derive(Debug)]
pub struct Code93(Vec<char>);

impl Code93 {
    /// Creates a new barcode.
    ///
    /// # Returns
    /// Returns `Result<Code93, Error>` indicating parse success.
    ///
    /// # Errors
    /// Returns an `Error::Length` if the input data length is invalid.
    /// Returns an `Error::Character` if the input data contains invalid characters.
    ///
    /// # Panics
    /// Panics if the input data cannot be parsed due to an unexpected error.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Self> {
        Ok(Self::parse(data.as_ref())
            .map(|d| Self(d.chars().collect()))
            .expect("Failed to parse input data"))
    }

    pub(crate) fn char_encoding(c: char) -> [u8; 9] {
        match CHARS.iter().find(|&ch| ch.0 == c) {
            Some(&(_, enc)) => enc,
            None => panic!("Unknown char: {c}"),
        }
    }

    /// Calculates a checksum character using a weighted modulo-47 algorithm.
    pub(crate) fn checksum_char(data: &[char], weight_threshold: usize) -> Option<char> {
        let get_char_pos = |&c| {
            CHARS
                .iter()
                .position(|t| t.0 == c)
                .expect("Character not found in CHARS mapping")
        };
        let weight = |i| match (data.len() - i) % weight_threshold {
            0 => weight_threshold,
            n => n,
        };
        let positions = data.iter().map(get_char_pos);
        let index = positions
            .enumerate()
            .fold(0, |acc, (i, pos)| acc + (weight(i) * pos));

        CHARS.get(index % CHARS.len()).map(|&(c, _)| c)
    }

    /// Calculates the C checksum character using a weighted modulo-47 algorithm.
    pub(crate) fn c_checksum_char(data: &[char]) -> Option<char> {
        Self::checksum_char(data, 20)
    }

    /// Calculates the K checksum character using a weighted modulo-47 algorithm.
    pub(crate) fn k_checksum_char(data: &[char], c_checksum: char) -> Option<char> {
        let mut extended_data: Vec<char> = data.to_vec();
        extended_data.push(c_checksum);

        Self::checksum_char(&extended_data, 15)
    }

    fn push_encoding(into: &mut Vec<u8>, from: [u8; 9]) {
        into.extend(from.iter().copied());
    }

    fn payload(&self) -> Vec<u8> {
        let mut enc = vec![];
        let c_checksum = Self::c_checksum_char(&self.0).expect("Cannot compute checksum C");
        let k_checksum =
            Self::k_checksum_char(&self.0, c_checksum).expect("Cannot compute checksum K");

        for &c in &self.0 {
            Self::push_encoding(&mut enc, Self::char_encoding(c));
        }

        // Checksums.
        Self::push_encoding(&mut enc, Self::char_encoding(c_checksum));
        Self::push_encoding(&mut enc, Self::char_encoding(k_checksum));

        enc
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of encoded binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let guard = &GUARD[..];
        let terminator = &TERMINATOR[..];

        helpers::join_slices(&[guard, &self.payload()[..], guard, terminator][..])
    }
}

impl Parse for Code93 {
    /// Returns the valid length of data acceptable in this type of barcode.
    /// Code93 barcodes are variable-length.
    fn valid_len() -> Range<u32> {
        1..256
    }

    /// Returns the set of valid characters allowed in this type of barcode.
    fn valid_chars() -> Vec<char> {
        let (chars, _): (Vec<_>, Vec<_>) = CHARS.iter().copied().unzip();
        chars
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::sym::code93::*;
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
    fn invalid_length_code93() {
        let code93 = Code93::new("").expect_err("Expected an error for empty input");

        assert_eq!(code93, Error::Length);
    }

    #[test]
    fn invalid_data_code93() {
        let code93 =
            Code93::new("lowerCASE").expect_err("Expected an error for invalid characters");

        assert_eq!(
            code93,
            Error::Character,
            "Expected Error::Character, but got {code93:?}"
        );
    }

    #[test]
    fn code93_encode() {
        // Tests for data longer than 15, data longer than 20
        let code931 = Code93::new("TEST93").expect("Failed to create Code93 for 'TEST93'");
        let code932 = Code93::new("FLAM").expect("Failed to create Code93 for 'FLAM'");
        let code933 = Code93::new("99").expect("Failed to create Code93 for '99'");
        let code934 =
            Code93::new("1111111111111111111111").expect("Failed to create Code93 for long input");

        assert_eq!(collapse_vec(&code931.encode()), "1010111101101001101100100101101011001101001101000010101010000101011101101001000101010111101");
        assert_eq!(
            collapse_vec(&code932.encode()),
            "1010111101100010101010110001101010001010011001001011001010011001010111101"
        );
        assert_eq!(
            collapse_vec(&code933.encode()),
            "1010111101000010101000010101101100101000101101010111101"
        );
        assert_eq!(collapse_vec(&code934.encode()), "1010111101010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001000101101110010101010111101");
    }
}
