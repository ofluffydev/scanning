//! Encoder for Code11 (USD-8) barcodes.
//!
//! Code11 is able to encode all of the decimal digits and the dash character. It is mainly
//! used in the telecommunications industry.
//!
//! Code11 is a discrete symbology. This encoder always provides a C checksum. For barcodes longer
//! than 10 characters, a second checksum digit (K) is appended.

use crate::error::Result;
use crate::sym::{helpers, Parse};
use core::ops::Range;
use helpers::{vec, Vec};

// Character -> Binary mappings for each of the allowable characters.
// The special "full-ASCII" characters are represented with (, ), [, ].
const CHARS: [(char, &[u8]); 11] = [
    ('0', &[1, 0, 1, 0, 1, 1]),
    ('1', &[1, 1, 0, 1, 0, 1, 1]),
    ('2', &[1, 0, 0, 1, 0, 1, 1]),
    ('3', &[1, 1, 0, 0, 1, 0, 1]),
    ('4', &[1, 0, 1, 1, 0, 1, 1]),
    ('5', &[1, 1, 0, 1, 1, 0, 1]),
    ('6', &[1, 0, 0, 1, 1, 0, 1]),
    ('7', &[1, 0, 1, 0, 0, 1, 1]),
    ('8', &[1, 1, 0, 1, 0, 0, 1]),
    ('9', &[1, 1, 0, 1, 0, 1]),
    ('-', &[1, 0, 1, 1, 0, 1]),
];

// Code11 barcodes must start and end with a special character.
const GUARD: [u8; 7] = [1, 0, 1, 1, 0, 0, 1];
const SEPARATOR: [u8; 1] = [0];

/// The Code11 barcode type.
#[derive(Debug)]
pub struct Code11(Vec<char>);

/// The USD-8 barcode type.
pub type USD8 = Code11;

impl Code11 {
    /// Creates a new barcode.
    ///
    /// # Returns
    /// Returns `Result<Code11, Error>` indicating parse success.
    ///
    /// # Errors
    /// Returns an error if the input data is invalid, such as containing
    /// unsupported characters or having an invalid length.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Self> {
        Self::parse(data.as_ref()).map(|d| Self(d.chars().collect()))
    }

    fn char_encoding(c: char) -> &'static [u8] {
        match CHARS.iter().find(|&ch| ch.0 == c) {
            Some(&(_, enc)) => enc,
            None => panic!("Unknown char: {c}"),
        }
    }

    /// Calculates a checksum character using a weighted modulo-11 algorithm.
    fn checksum_char(data: &[char], weight_threshold: usize) -> Option<char> {
        let get_char_pos = |&c| {
            CHARS
                .iter()
                .position(|t| t.0 == c)
                .expect("Character not found in CHARS mapping")
        };
        let weight = |i| match i % weight_threshold {
            0 => weight_threshold,
            n => n,
        };
        let positions = data.iter().map(&get_char_pos);
        let index = positions
            .rev()
            .enumerate()
            .fold(0, |acc, (i, pos)| acc + (weight(i + 1) * pos));

        // Some sources suggest that the C checksum should use modulo-11, whilst the K
        // checksum should use modulo-9. But most generators always use modulo-11.
        // This algorithm currently just uses 11 for both checksums, but can be easily
        // changed at a later date.
        CHARS.get(index % CHARS.len()).map(|&(c, _)| c)
    }

    /// Calculates the C checksum character using a weighted modulo-11 algorithm.
    fn c_checksum_char(&self) -> Option<char> {
        Self::checksum_char(&self.0, 10)
    }

    /// Calculates the K checksum character using a weighted modulo-11 algorithm.
    fn k_checksum_char(&self, c_checksum: char) -> Option<char> {
        let mut data: Vec<char> = self.0.clone();
        data.push(c_checksum);

        Self::checksum_char(&data, 9)
    }

    fn push_encoding(into: &mut Vec<u8>, from: &[u8]) {
        into.extend(from.iter().copied());
        into.extend(&SEPARATOR);
    }

    fn payload(&self) -> Vec<u8> {
        let mut enc = vec![];
        let c_checksum = self.c_checksum_char().expect("Cannot compute checksum C");

        for &c in &self.0 {
            Self::push_encoding(&mut enc, Self::char_encoding(c));
        }

        Self::push_encoding(&mut enc, Self::char_encoding(c_checksum));

        // K-checksum is only appended on barcodes greater than 10 characters.
        if self.0.len() > 10 {
            let k_checksum = self
                .k_checksum_char(c_checksum)
                .expect("Cannot compute checksum K");

            Self::push_encoding(&mut enc, Self::char_encoding(k_checksum));
        }

        enc
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of encoded binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let guard = &GUARD[..];

        helpers::join_slices(&[guard, &SEPARATOR, &self.payload()[..], guard][..])
    }
}

impl Parse for Code11 {
    /// Returns the valid length of data acceptable in this type of barcode.
    /// Code11 barcodes are variable-length.
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
    use crate::sym::code11::*;
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
    fn invalid_length_code11() {
        let code11 = Code11::new("");

        assert_eq!(
            code11.expect_err("Expected an Error::Length but got None"),
            Error::Length
        );
    }

    #[test]
    fn invalid_data_code11() {
        let code11 = Code11::new("NOTDIGITS");

        assert_eq!(
            code11.expect_err("Expected an Error::Character but got None"),
            Error::Character
        );
    }

    #[test]
    fn code11_encode_less_than_10_chars() {
        let code111 = Code11::new("123-45").expect("Failed to create Code11 barcode for '123-45'");
        let code112 = Code11::new("666").expect("Failed to create Code11 barcode for '666'");
        let code113 = Code11::new("12-9").expect("Failed to create Code11 barcode for '12-9'");

        assert_eq!(
            collapse_vec(&code111.encode()),
            "1011001011010110100101101100101010110101011011011011010110110101011001"
        );
        assert_eq!(
            collapse_vec(&code112.encode()),
            "10110010100110101001101010011010110010101011001"
        );
        assert_eq!(
            collapse_vec(&code113.encode()),
            "10110010110101101001011010110101101010100110101011001"
        );
    }

    #[test]
    fn code11_encode_more_than_10_chars() {
        let code111 = Code11::new("1234-5678-4321")
            .expect("Failed to create Code11 barcode for '1234-5678-4321'");

        assert_eq!(collapse_vec(&code111.encode()), "101100101101011010010110110010101011011010110101101101010011010101001101101001010110101011011011001010100101101101011011011010100110101011001");
    }
}
