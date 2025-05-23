//! Encoder for Code128 barcodes.
//!
//! Code128 is a popular, high-density symbology that allows for the encoding of alphanumeric
//! data along with many special characters by utilising three separate character-sets.
//!
//! Code128 also offers double-density encoding of digits.
//!
//! ## Character sets
//!
//! Barcoders provides special Unicode syntax for specifying the character set(s) which should be
//! used in the barcode:
//!
//! <ul><li>\u{00C0} = Switch to character-set A (À)</li>
//! <li>\u{0181} = Switch to character-set B (Ɓ)</li>
//! <li>\u{0106} = Switch to character-set C (Ć)</li></ul>
//!
//! You must provide both the starting character-set along with any changes during the data. This
//! means all Code128 barcodes must start with either "À", "Ɓ" or "Ć". Simple alphanumeric data
//! can generally use character-set A solely.
//!
//! As an example, this barcode uses character-set B:
//!
//! <ul><li>\u{0181}HE1234A*1</li></ul>
//!
//! Or:
//!
//! <ul><li>ƁHE1234A*1</li></ul>
//!
//! And this one starts at character-set A (the default) and then switches to C to encode the digits more
//! effectively:
//!
//! <ul><li>\u{00C0}HE@$A\u{0106}123456</li></ul>
//!
//! Or:
//!
//! <ul><li>ÀHE@$AĆ123456</li></ul>
//!
//! ## Unicode characters
//!
//! The invisible unicode characters that are available in character set A should be represented as
//! their Unicode sequences. For example, to represent the 'ACK' character:
//!
//! <ul><li>À\u{0006}</li></ul>
//!
//! ## Special-purpose function characters (FNC1 - 4)
//!
//! The function sequences can be represented via the following unicode characters:
//!
//! - FNC1: ```Ź``` (```\u{0179}```)
//! - FNC2: ```ź``` (```\u{017A}```)
//! - FNC3: ```Ż``` (```\u{017B}```)
//! - FNC4: ```ż``` (```\u{017C}```)
//! - SHIFT: ```Ž``` (```\u{017D}```)

use crate::error::{Error, Result};
use crate::sym::helpers;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
use core::cmp;
use helpers::{vec, Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Unit {
    kind: UnitKind,
    index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitKind {
    /// Represents character set A in Code128 barcodes.
    A,
    /// Represents character set B in Code128 barcodes.
    B,
    /// Represents character set C in Code128 barcodes.
    C,
}

type Encoding = [u8; 11];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents the character sets available in Code128 barcodes.
/// Using special characters to switch between character sets is still supported.
///
/// - `A`: Character set A, which includes ASCII characters 00 to 95.
/// - `B`: Character set B, which includes ASCII characters 32 to 127.
/// - `C`: Character set C, which encodes pairs of digits (00–99).
/// - `None`: No character set specified.
pub enum CharacterSet {
    /// Character set A
    A,
    /// Character set B
    B,
    /// Character set C
    C,
    /// No character set specified, will error if you don't use a special character to switch.
    None,
}

// Character -> Binary mappings for each of the allowable characters in each character-set.
const CHARS: [([&str; 3], Encoding); 106] = [
    ([" ", " ", "00"], [1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0]),
    (["!", "!", "01"], [1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0]),
    (["\"", "\"", "02"], [1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0]),
    (["#", "#", "03"], [1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0]),
    (["$", "$", "04"], [1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0]),
    (["%", "%", "05"], [1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0]),
    (["&", "&", "06"], [1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0]),
    (["'", "'", "07"], [1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0]),
    (["(", "(", "08"], [1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0]),
    ([")", ")", "09"], [1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0]),
    (["*", "*", "10"], [1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0]),
    (["+", "+", "11"], [1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0]),
    ([",", ",", "12"], [1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0]),
    (["-", "-", "13"], [1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 0]),
    ([".", ".", "14"], [1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0]),
    (["/", "/", "15"], [1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0]),
    (["0", "0", "16"], [1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0]),
    (["1", "1", "17"], [1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0]),
    (["2", "2", "18"], [1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0]),
    (["3", "3", "19"], [1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0]),
    (["4", "4", "20"], [1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0]),
    (["5", "5", "21"], [1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0]),
    (["6", "6", "22"], [1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 0]),
    (["7", "7", "23"], [1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 0]),
    (["8", "8", "24"], [1, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0]),
    (["9", "9", "25"], [1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0]),
    ([":", ":", "26"], [1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0]),
    ([";", ";", "27"], [1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0]),
    (["<", "<", "28"], [1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0]),
    (["=", "=", "29"], [1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0]),
    ([">", ">", "30"], [1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 0]),
    (["?", "?", "31"], [1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0]),
    (["@", "@", "32"], [1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0]),
    (["A", "A", "33"], [1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0]),
    (["B", "B", "34"], [1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0]),
    (["C", "C", "35"], [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0]),
    (["D", "D", "36"], [1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0]),
    (["E", "E", "37"], [1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0]),
    (["F", "F", "38"], [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0]),
    (["G", "G", "39"], [1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0]),
    (["H", "H", "40"], [1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0]),
    (["I", "I", "41"], [1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0]),
    (["J", "J", "42"], [1, 0, 1, 1, 0, 1, 1, 1, 0, 0, 0]),
    (["K", "K", "43"], [1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0]),
    (["L", "L", "44"], [1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0]),
    (["M", "M", "45"], [1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0]),
    (["N", "N", "46"], [1, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0]),
    (["O", "O", "47"], [1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0]),
    (["P", "P", "48"], [1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0]),
    (["Q", "Q", "49"], [1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0]),
    (["R", "R", "50"], [1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0]),
    (["S", "S", "51"], [1, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0]),
    (["T", "T", "52"], [1, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0]),
    (["U", "U", "53"], [1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0]),
    (["V", "V", "54"], [1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0]),
    (["W", "W", "55"], [1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0]),
    (["X", "X", "56"], [1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0]),
    (["Y", "Y", "57"], [1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0]),
    (["Z", "Z", "58"], [1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0]),
    (["[", "[", "59"], [1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0]),
    (["\\", "\\", "60"], [1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0]),
    (["]", "]", "61"], [1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0]),
    (["^", "^", "62"], [1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0]),
    (["_", "_", "63"], [1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0]),
    (["\u{0000}", "`", "64"], [1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0]),
    (["\u{0001}", "a", "65"], [1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0]),
    (["\u{0002}", "b", "66"], [1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0]),
    (["\u{0003}", "c", "67"], [1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0]),
    (["\u{0004}", "d", "68"], [1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0]),
    (["\u{0005}", "e", "69"], [1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0]),
    (["\u{0006}", "f", "70"], [1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0]),
    (["\u{0007}", "g", "71"], [1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0]),
    (["\u{0008}", "h", "72"], [1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0]),
    (["\u{0009}", "i", "73"], [1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0]),
    (["\u{000A}", "j", "74"], [1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0]),
    (["\u{000B}", "k", "75"], [1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0]),
    (["\u{000C}", "l", "76"], [1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0]),
    (["\u{000D}", "m", "77"], [1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0]),
    (["\u{000E}", "n", "78"], [1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0]),
    (["\u{000F}", "o", "79"], [1, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0]),
    (["\u{0010}", "p", "80"], [1, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0]),
    (["\u{0011}", "q", "81"], [1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0]),
    (["\u{0012}", "r", "82"], [1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0]),
    (["\u{0013}", "s", "83"], [1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0]),
    (["\u{0014}", "t", "84"], [1, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0]),
    (["\u{0015}", "u", "85"], [1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0]),
    (["\u{0016}", "v", "86"], [1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0]),
    (["\u{0017}", "w", "87"], [1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0]),
    (["\u{0018}", "x", "88"], [1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0]),
    (["\u{0019}", "y", "89"], [1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0]),
    (["\u{001A}", "z", "90"], [1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0]),
    (["\u{001B}", "{", "91"], [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0]),
    (["\u{001C}", "|", "92"], [1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0]),
    (["\u{001D}", "}", "93"], [1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0]),
    (["\u{001E}", "~", "94"], [1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0]),
    (
        ["\u{001F}", "\u{00F7}", "95"],
        [1, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0],
    ),
    (
        ["\u{017B}", "\u{017B}", "96"],
        [1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0],
    ),
    (
        ["\u{017A}", "\u{017A}", "97"],
        [1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0],
    ),
    (
        ["\u{017D}", "\u{017D}", "98"],
        [1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0],
    ),
    (["Ć", "Ć", "99"], [1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0]),
    (["Ɓ", "\u{017C}", "Ɓ"], [1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0]),
    (["\u{017C}", "À", "À"], [1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0]),
    (
        ["\u{0179}", "\u{0179}", "\u{0179}"],
        [1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0],
    ),
    (
        ["START-À", "START-À", "START-À"],
        [1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0],
    ),
    (
        ["START-Ɓ", "START-Ɓ", "START-Ɓ"],
        [1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0],
    ),
    (
        ["START-Ć", "START-Ć", "START-Ć"],
        [1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0],
    ),
];

// Stop sequence.
const STOP: Encoding = [1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0];

// Termination sequence.
const TERM: [u8; 2] = [1, 1];

/// The Code128 barcode type.
///
/// # Character sets
///
/// * 128A (Code Set A) – ASCII characters 00 to 95 (0–9, A–Z and control
///   codes), special characters, and FNC 1–4
/// * 128B (Code Set B) – ASCII characters 32 to 127 (0–9, A–Z, a–z), special
///   characters, and FNC 1–4
/// * 128C (Code Set C) – 00–99 (encodes two digits with a single code point)
///   and FNC1
///
/// See [module] docs for additional information.
///
/// [module]: crate::sym::code128
#[derive(Debug)]
pub struct Code128(Vec<Unit>);

impl Unit {
    const fn index(&self) -> usize {
        self.index
    }
}

impl CharacterSet {
    const fn from_char(c: char) -> Result<Self> {
        match c {
            'À' => Ok(Self::A),
            'Ɓ' => Ok(Self::B),
            'Ć' => Ok(Self::C),
            _ => Err(Error::Character),
        }
    }

    const fn unit(self, n: usize) -> Result<Unit> {
        let kind = match self {
            Self::A => UnitKind::A,
            Self::B => UnitKind::B,
            Self::C => UnitKind::C,
            Self::None => return Err(Error::Character),
        };
        Ok(Unit { kind, index: n })
    }

    const fn index(self) -> Result<usize> {
        match self {
            Self::A => Ok(0),
            Self::B => Ok(1),
            Self::C => Ok(2),
            Self::None => Err(Error::Character),
        }
    }

    fn lookup(self, s: &str) -> Result<Unit> {
        let p = self.index()?;

        CHARS
            .iter()
            .position(|&c| c.0[p] == s)
            .map_or(Err(Error::Character), |i| self.unit(i))
    }
}

impl Code128 {
    /// Creates a new barcode.
    ///
    /// # Errors
    ///
    /// Returns an `Error::Length` if the input data is too short.
    /// Returns an `Error::Character` if the input data contains invalid characters or an invalid character set.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Code128` barcode on success.
    pub fn new<T: AsRef<str>>(data: T, character_set: CharacterSet) -> Result<Self> {
        let data = data.as_ref();

        if data.len() < 2 {
            return Err(Error::Length);
        }

        // Append a letter depending on the character-set.
        let starting_char = match character_set {
            CharacterSet::A => 'À',                             // Character set A
            CharacterSet::B => 'Ɓ',                             // Character set B
            CharacterSet::C => 'Ć',                             // Character set C
            CharacterSet::None => return Err(Error::Character), // No character set
        };

        // Prepend the starting character to the data.
        let data = format!("{starting_char}{data}");

        Self::parse(data.chars().collect()).map(Code128)
    }

    // Tokenizes and collects the data into the appropriate character-sets.
    fn parse(chars: Vec<char>) -> Result<Vec<Unit>> {
        let mut units: Vec<Unit> = vec![];
        let mut char_set = CharacterSet::None;
        let mut carry: Option<char> = None;

        for ch in chars {
            match ch {
                'À' | 'Ɓ' | 'Ć' if units.is_empty() => {
                    char_set = CharacterSet::from_char(ch)?;

                    let c = format!("START-{ch}");
                    let u = char_set.lookup(&c)?;
                    units.push(u);
                }
                'À' | 'Ɓ' | 'Ć' => {
                    if char_set == CharacterSet::C && carry.is_some() {
                        return Err(Error::Character);
                    }
                    let u = char_set.lookup(&ch.to_string())?;
                    units.push(u);

                    char_set = CharacterSet::from_char(ch)?;
                }
                d if d.is_ascii_digit() && char_set == CharacterSet::C => match carry {
                    None => carry = Some(d),
                    Some(n) => {
                        let num = format!("{n}{d}");
                        let u = char_set.lookup(&num)?;
                        units.push(u);
                        carry = None;
                    }
                },
                _ => {
                    let u = char_set.lookup(&ch.to_string())?;
                    units.push(u);
                }
            }
        }

        match carry {
            Some(_) => Err(Error::Character),
            None => Ok(units),
        }
    }

    /// Calculates the checksum index using a modulo-103 algorithm.
    fn checksum_value(&self) -> u8 {
        let sum: usize = self
            .0
            .iter()
            .zip(0..self.0.len())
            .fold(0, |t, (u, i)| t + (u.index() * cmp::max(1, i)));

        (sum % 103)
            .try_into()
            .expect("Checksum value should always be non-negative and fit in u8")
    }

    fn checksum_encoding(&self) -> Encoding {
        let v = self.checksum_value();
        Self::unit_encoding(&Unit {
            kind: UnitKind::A,
            index: v as usize,
        })
    }

    const fn unit_encoding(c: &Unit) -> Encoding {
        CHARS[c.index()].1
    }

    fn payload(&self) -> Vec<u8> {
        let slices: Vec<Encoding> = self.0.iter().map(Self::unit_encoding).collect();

        helpers::join_iters(slices.iter())
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        helpers::join_slices(
            &[
                &self.payload()[..],
                &self.checksum_encoding()[..],
                &STOP[..],
                &TERM[..],
            ][..],
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::sym::code128::*;
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
    fn new_code128() {
        let code128_a = Code128::new(" !! Ć0201", CharacterSet::A);
        let code128_b = Code128::new("!!  \" ", CharacterSet::A);

        assert!(code128_a.is_ok());
        assert!(code128_b.is_ok());
    }

    #[test]
    fn invalid_length_code128() {
        let code128_a = Code128::new("", CharacterSet::None);

        assert_eq!(
            code128_a.expect_err("Expected Error::Length but got None"),
            Error::Length
        );
    }

    #[test]
    fn invalid_data_code128() {
        let code128_a = Code128::new("☺ ", CharacterSet::A); // Unknown character.
        let code128_b = Code128::new("HELLOĆ12352", CharacterSet::A); // Trailing carry at the end.
        let code128_c = Code128::new("HELLO", CharacterSet::None); // No Character-Set specified.

        assert_eq!(
            code128_a.expect_err("Expected Error::Character but got None"),
            Error::Character
        );
        assert_eq!(
            code128_b.expect_err("Expected Error::Character but got None"),
            Error::Character
        );
        assert_eq!(
            code128_c.expect_err("Expected Error::Character but got None"),
            Error::Character
        );
    }

    #[test]
    fn code128_encode() {
        let code128_a = Code128::new("HELLO", CharacterSet::A)
            .expect("Failed to create Code128 barcode with CharacterSet A");
        let code128_b = Code128::new("XYĆ2199", CharacterSet::A)
            .expect("Failed to create Code128 barcode with CharacterSet A");
        let code128_c = Code128::new("xyZÀ199!*1", CharacterSet::B)
            .expect("Failed to create Code128 barcode with CharacterSet B");

        assert_eq!(collapse_vec(&code128_a.encode()), "110100001001100010100010001101000100011011101000110111010001110110110100010001100011101011");
        assert_eq!(collapse_vec(&code128_b.encode()), "110100001001110001011011101101000101110111101101110010010111011110100111011001100011101011");
        assert_eq!(collapse_vec(&code128_c.encode()), "1101001000011110010010110110111101110110001011101011110100111001101110010110011100101100110011011001100100010010011100110100101111001100011101011");
    }

    #[test]
    fn code128_encode_special_chars() {
        let code128_a = Code128::new("B\u{0006}", CharacterSet::A)
            .expect("Failed to create Code128 barcode with special character");

        assert_eq!(
            collapse_vec(&code128_a.encode()),
            "110100001001000101100010110000100100110100001100011101011"
        );
    }

    #[test]
    fn code128_encode_fnc_chars() {
        let code128_a = Code128::new("Ź4218402050À0", CharacterSet::A)
            .expect("Failed to create Code128 barcode with FNC characters");

        assert_eq!(collapse_vec(&code128_a.encode()), "110100111001111010111010110111000110011100101100010100011001001110110001011101110101111010011101100101011110001100011101011");
    }

    #[test]
    fn code128_encode_longhand() {
        let code128_a = Code128::new("\u{00C0}HELLO", CharacterSet::None)
            .expect("Failed to create Code128 barcode with longhand syntax");
        let code128_b = Code128::new("\u{00C0}XY\u{0106}2199", CharacterSet::None)
            .expect("Failed to create Code128 barcode with longhand syntax");
        let code128_c = Code128::new("\u{0181}xyZ\u{00C0}199!*1", CharacterSet::None)
            .expect("Failed to create Code128 barcode with longhand syntax");

        assert_eq!(collapse_vec(&code128_a.encode()), "110100001001100010100010001101000100011011101000110111010001110110110100010001100011101011");
        assert_eq!(collapse_vec(&code128_b.encode()), "110100001001110001011011101101000101110111101101110010010111011110100111011001100011101011");
        assert_eq!(collapse_vec(&code128_c.encode()), "1101001000011110010010110110111101110110001011101011110100111001101110010110011100101100110011011001100100010010011100110100101111001100011101011");
    }
}
