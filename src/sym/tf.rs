//! Encoder for 2-of-5 barcodes.
//!
//! 2-of-5 barcodes are often used by Airlines and in some industrial settings.
//!
//! They also make an appearance in retail where they are sometimes used for the outer cartons on
//! groups of products (cartons of Cola, etc).
//!
//! Most of the time you will want to use the interleaved barcode over the standard option.

use crate::error::Result;
use crate::sym::helpers;
use crate::sym::Parse;
use core::char;
use core::ops::Range;
use helpers::{vec, Vec};

#[rustfmt::skip]
const WIDTHS: [&str; 10] = [
    "NNWWN", "WNNNW", "NWNNW",
    "WWNNN", "NNWNW", "WNWNN",
    "NWWNN", "NNNWW", "WNNWN",
    "NWNWN",
];

const ITF_START: [u8; 4] = [1, 0, 1, 0];
const ITF_STOP: [u8; 4] = [1, 1, 0, 1];
const STF_START: [u8; 8] = [1, 1, 0, 1, 1, 0, 1, 0];
const STF_STOP: [u8; 8] = [1, 1, 0, 1, 0, 1, 1, 0];

/// The 2-of-5 barcode type.
#[derive(Debug)]
pub enum TF {
    /// The standard 2-of-5 barcode type.
    Standard(Vec<u8>),
    /// The interleaved 2-of-5 barcode type.
    Interleaved(Vec<u8>),
}

impl TF {
    /// Creates a new ITF barcode.
    /// If the length of the given data is odd, a checksum value will be computed and appended to
    /// the data for encoding.
    ///
    /// # Errors
    /// This function returns an error if the input data contains invalid characters that are not digits.
    ///
    /// # Panics
    /// This function will panic if a character in the input data cannot be converted to a digit.
    ///
    /// Returns Result<`TF::Interleaved`, Error> indicating parse success.
    pub fn interleaved<T: AsRef<str>>(data: T) -> Result<Self> {
        Self::parse(data.as_ref()).map(|d| {
            #[allow(clippy::cast_possible_truncation)] // Safe: to_digit(10) returns values in 0..=9
            let mut digits: Vec<u8> = d
                .chars()
                .map(|c| {
                    c.to_digit(10)
                        .expect("Failed to convert character to digit") as u8
                })
                .collect();
            let checksum_required = digits.len() % 2 == 1;

            if checksum_required {
                let check_digit = helpers::modulo_10_checksum(&digits[..], false);
                digits.push(check_digit);
            }

            Self::Interleaved(digits)
        })
    }

    /// Creates a new STF barcode.
    ///
    /// Creates a new STF barcode.
    ///
    /// # Errors
    /// This function returns an error if the input data contains invalid characters that are not digits.
    ///
    /// # Panics
    /// This function will panic if a character in the input data cannot be converted to a digit.
    ///
    /// Returns Result<`TF::Standard`, Error> indicating parse success.
    pub fn standard<T: AsRef<str>>(data: T) -> Result<Self> {
        Self::parse(data.as_ref()).map(|d| {
            #[allow(clippy::cast_possible_truncation)] // Safe: to_digit(10) returns values in 0..=9
            let digits: Vec<u8> = d
                .chars()
                .map(|c| {
                    c.to_digit(10)
                        .expect("Failed to convert character to digit") as u8
                })
                .collect();
            Self::Standard(digits)
        })
    }

    fn raw_data(&self) -> &[u8] {
        match *self {
            Self::Standard(ref d) | Self::Interleaved(ref d) => &d[..],
        }
    }

    pub(crate) fn interleave(bars: u8, spaces: u8) -> Vec<u8> {
        let bwidths = WIDTHS[bars as usize].chars();
        let swidths = WIDTHS[spaces as usize].chars();
        let mut encoding: Vec<u8> = vec![];

        for (b, s) in bwidths.zip(swidths) {
            for &(c, i) in &[(b, 1), (s, 0)] {
                match c {
                    'W' => encoding.extend([i; 3].iter().copied()),
                    _ => encoding.push(i),
                }
            }
        }

        encoding
    }

    fn char_encoding(d: u8) -> Vec<u8> {
        let bars: Vec<Vec<u8>> = Self::char_widths(d)
            .chars()
            .map(|c| match c {
                'W' => vec![1, 1, 1, 0],
                _ => vec![1, 0],
            })
            .collect();

        helpers::join_iters(bars.iter())
    }

    const fn char_widths(d: u8) -> &'static str {
        WIDTHS[d as usize]
    }

    fn stf_payload(&self) -> Vec<u8> {
        let mut encodings = vec![];

        for d in self.raw_data() {
            encodings.extend(Self::char_encoding(*d).iter().copied());
        }

        encodings
    }

    fn itf_payload(&self) -> Vec<u8> {
        let weaves: Vec<Vec<u8>> = self
            .raw_data()
            .chunks(2)
            .map(|c| Self::interleave(c[0], c[1]))
            .collect();

        helpers::join_iters(weaves.iter())
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        match *self {
            Self::Standard(_) => {
                helpers::join_slices(&[&STF_START[..], &self.stf_payload()[..], &STF_STOP[..]][..])
            }
            Self::Interleaved(_) => {
                helpers::join_slices(&[&ITF_START[..], &self.itf_payload()[..], &ITF_STOP[..]][..])
            }
        }
    }
}

impl Parse for TF {
    /// Returns the valid length of data acceptable in this type of barcode.
    /// 2-of-5 barcodes are variable-length.
    fn valid_len() -> Range<u32> {
        1..256
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
    use crate::sym::tf::*;
    #[cfg(not(feature = "std"))]
    pub(crate) use alloc::string::{String, ToString};
    use core::char;

    fn collapse_vec(v: &[u8]) -> String {
        let chars = v.iter().map(|d| {
            char::from_digit(u32::from(*d), 10).expect("Failed to convert digit to character")
        });
        chars.collect()
    }

    #[test]
    fn new_itf() {
        let itf = TF::interleaved("12345679");

        assert!(itf.is_ok());
    }

    #[test]
    fn new_stf() {
        let stf = TF::standard("12345");

        assert!(stf.is_ok());
    }

    #[test]
    fn invalid_data_itf() {
        let itf = TF::interleaved("1234er123412");

        assert_eq!(itf.expect_err("Expected an error"), Error::Character);
    }

    #[test]
    fn invalid_data_stf() {
        let stf = TF::standard("WORDUP");

        assert_eq!(stf.expect_err("Expected an error"), Error::Character);
    }

    #[test]
    fn itf_raw_data() {
        let itf = TF::interleaved("12345679").expect("Failed to create interleaved barcode");

        assert_eq!(itf.raw_data(), &[1, 2, 3, 4, 5, 6, 7, 9]);
    }

    #[test]
    fn itf_encode() {
        let itf = TF::interleaved("1234567").expect("Failed to create interleaved barcode"); // Check digit: 0

        assert_eq!(
            collapse_vec(&itf.encode()),
            "10101110100010101110001110111010001010001110100011100010101010100011100011101101"
                .to_string()
        );
    }

    #[test]
    fn stf_encode() {
        let stf = TF::standard("1234567").expect("Failed to create standard barcode");

        assert_eq!(collapse_vec(&stf.encode()), "110110101110101010111010111010101110111011101010101010111010111011101011101010101110111010101010101110111011010110".to_string());
    }
}
