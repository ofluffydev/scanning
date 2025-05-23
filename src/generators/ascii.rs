//! Functionality for generating ASCII representations of barcodes.
//!
//! This is useful for testing and simple verification of barcode correctness.
//!
//! You will pretty much never need to turn this feature on unless you are adding new functionality
//! or running the test suite.

use crate::error::Result;
#[cfg(not(feature = "std"))]
use alloc::string::String;

/// The ASCII barcode generator type.
#[derive(Copy, Clone, Debug)]
pub struct ASCII {
    /// The height of the barcode (```self.height``` characters high for ASCII).
    pub height: usize,
    /// The X dimension. Specifies the width of the "narrow" bars.
    /// For ASCII, each will be ```self.xdim``` characters wide.
    pub xdim: usize,
}

/// Maps binary digits to ASCII representation (0=' ', 1='#')
const CHARS: [char; 2] = [' ', '#'];

impl Default for ASCII {
    fn default() -> Self {
        Self::new()
    }
}

impl ASCII {
    /// Returns a new ASCII with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            height: 10,
            xdim: 1,
        }
    }

    fn generate_row(&self, barcode: &[u8]) -> String {
        barcode
            .iter()
            .flat_map(|&d| std::iter::repeat_n(CHARS[d as usize], self.xdim))
            .collect()
    }

    /// Generates the given barcode.
    ///
    /// Returns a `Result<String, Error>` indicating success.
    ///
    /// # Errors
    ///
    /// This function will return an error if the barcode data is invalid or cannot be processed.
    pub fn generate<T: AsRef<[u8]>>(&self, barcode: T) -> Result<String> {
        let mut output = String::new();
        let row = self.generate_row(barcode.as_ref());

        for (i, _l) in (0..self.height).enumerate() {
            output.push_str(&row[..]);

            if i < self.height - 1 {
                output.push('\n');
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::generators::ascii::*;
    use crate::sym::codabar::*;
    use crate::sym::code11::*;
    use crate::sym::code128::*;
    use crate::sym::code39::*;
    use crate::sym::code93::*;
    use crate::sym::ean13::*;
    use crate::sym::ean8::*;
    use crate::sym::ean_supp::*;
    use crate::sym::tf::*;

    #[test]
    fn ean_13_as_ascii() {
        let ean13 = EAN13::new("750103131130").expect("Failed to create EAN13 barcode");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&ean13.encode()[..])
            .expect("Failed to generate ASCII representation for EAN13 barcode");

        assert_eq!(
            generated,
            "
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
"
            .trim()
        );
    }

    #[test]
    fn ean_13_as_ascii_small_height_double_width() {
        let ean13 = EAN13::new("750103131130").expect("Failed to create EAN13 barcode");
        let ascii = ASCII { height: 6, xdim: 2 };
        let generated = ascii
            .generate(&ean13.encode()[..])
            .expect("Failed to generate ASCII representation for EAN13 barcode");

        assert_eq!(generated,
"
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
".trim());
    }

    #[test]
    fn ean_8_as_ascii() {
        let ean8 = EAN8::new("1234567").expect("Failed to create EAN8 barcode");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&ean8.encode()[..])
            .expect("Failed to generate ASCII representation for EAN8 barcode");

        assert_eq!(
            generated,
            "
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
"
            .trim()
        );
    }

    #[test]
    fn ean_8_as_ascii_small_height_double_width() {
        let ean8 = EAN8::new("1234567").expect("Failed to create EAN8 barcode");
        let ascii = ASCII { height: 5, xdim: 2 };
        let generated = ascii
            .generate(&ean8.encode()[..])
            .expect("Failed to generate ASCII representation for EAN8 barcode");

        assert_eq!(generated,
"
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
".trim());
    }

    #[test]
    fn code_39_as_ascii() {
        let code39 = Code39::new("TEST8052").expect("Failed to create Code39 barcode");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&code39.encode()[..])
            .expect("Failed to generate ASCII representation for Code39 barcode");

        assert_eq!(generated,
"
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
".trim());
    }

    #[test]
    fn code_39_as_ascii_small_height_double_weight() {
        let code39 = Code39::new("1234").expect("Failed to create Code39 barcode");
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii
            .generate(&code39.encode()[..])
            .expect("Failed to generate ASCII representation for Code39 barcode");

        assert_eq!(generated,
"
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
".trim());
    }

    #[test]
    fn codabar_as_ascii() {
        let codabar = Codabar::new("A98B").expect("Failed to create Codabar barcode");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&codabar.encode()[..])
            .expect("Failed to generate ASCII representation for Codabar barcode");

        assert_eq!(
            generated,
            "
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
"
            .trim()
        );
    }

    #[test]
    fn codabar_as_ascii_small_height_double_weight() {
        let codabar = Codabar::new("A40156B").expect("Failed to create Codabar barcode");
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii
            .generate(&codabar.encode()[..])
            .expect("Failed to generate ASCII representation for Codabar barcode");

        assert_eq!(generated,
"
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
".trim());
    }

    #[test]
    fn code_128_as_ascii() {
        let code128 =
            Code128::new("HELLO", CharacterSet::A).expect("Failed to create Code128 barcode");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&code128.encode()[..])
            .expect("Failed to generate ASCII representation for Code128 barcode");

        assert_eq!(
            generated,
            "
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
"
            .trim()
        );
    }

    #[test]
    fn code_128_as_ascii_small_height_double_weight() {
        let code128 = Code128::new("HELLO", CharacterSet::A)
            .expect("Failed to create Code128 barcode with CharacterSet::A");
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii
            .generate(&code128.encode()[..])
            .expect("Failed to generate ASCII representation for Code128 barcode");

        assert_eq!(generated,
"
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
".trim());
    }

    #[test]
    fn ean2_as_ascii() {
        let ean2 = EANSUPP::new("34").expect("Failed to create EAN2 barcode");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&ean2.encode()[..])
            .expect("Failed to generate ASCII representation for EAN2 barcode");

        assert_eq!(
            generated,
            "
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
"
            .trim()
        );
    }

    #[test]
    fn ean5_as_ascii() {
        let ean5 = EANSUPP::new("50799").expect("Failed to create EAN5 barcode");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&ean5.encode()[..])
            .expect("Failed to generate ASCII representation for EAN5 barcode");

        assert_eq!(
            generated,
            "
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
"
            .trim()
        );
    }

    #[test]
    fn itf_as_ascii() {
        let itf = TF::interleaved("12345")
            .expect("Failed to create interleaved TF barcode with input '12345'");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&itf.encode()[..])
            .expect("Failed to generate ASCII representation for interleaved TF barcode");

        assert_eq!(
            generated,
            "
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
"
            .trim()
        );
    }

    #[test]
    fn code_93_as_ascii() {
        let code93 =
            Code93::new("TEST93").expect("Failed to create Code93 barcode with input 'TEST93'");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&code93.encode()[..])
            .expect("Failed to generate ASCII representation for Code93 barcode");

        assert_eq!(
            generated,
            "
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
"
            .trim()
        );
    }

    #[test]
    fn code_93_as_ascii_small_height_double_weight() {
        let code93 =
            Code93::new("TEST93").expect("Failed to create Code93 barcode with input 'TEST93'");
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii
            .generate(&code93.encode()[..])
            .expect("Failed to generate ASCII representation for Code93 barcode");

        assert_eq!(generated,
"
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
".trim());
    }

    #[test]
    fn code_11_as_ascii() {
        let code11 =
            Code11::new("12-9").expect("Failed to create Code11 barcode with input '12-9'");
        let ascii = ASCII::new();
        let generated = ascii
            .generate(&code11.encode()[..])
            .expect("Failed to generate ASCII representation for Code11 barcode");

        assert_eq!(
            generated,
            "
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
"
            .trim()
        );
    }
}
