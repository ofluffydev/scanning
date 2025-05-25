//! Functionality for generating JSON representations of barcodes.
//!
//! This is useful for passing encoded data to third-party systems in a conventional format.
//!
//! Output will be of the format:
//! ```javascript
//! {
//!   "height": 10,
//!   "xdim": 1,
//!   "encoding": [1, 0, 0, 1, 1, 0, ...],
//! }
//! ```

use crate::error::Result;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// The JSON  barcode generator type.
#[derive(Copy, Clone, Debug)]
pub struct JSON {
    /// The height of the barcode.
    pub height: usize,
    /// The X dimension. Specifies the width of the "narrow" bars.
    pub xdim: usize,
}

impl Default for JSON {
    fn default() -> Self {
        Self::new()
    }
}

impl JSON {
    /// Returns a new JSON with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            height: 10,
            xdim: 1,
        }
    }

    /// Generates the given barcode.
    ///
    /// Returns a `Result<String, Error>` indicating success.
    ///
    /// # Errors
    ///
    /// This function will return an error if the barcode data cannot be processed
    /// into a valid JSON representation.
    pub fn generate<T: AsRef<[u8]>>(&self, barcode: T) -> Result<String> {
        let mut bits = barcode.as_ref().iter().fold(String::new(), |acc, &b| {
            let n = match b {
                0 => "0",
                _ => "1",
            };

            acc + n + ","
        });

        // Kill trailing comma.
        bits.pop();

        let output = format!(
            "{{\"height\":{},\"xdim\":{},\"encoding\":[{}]}}",
            self.height, self.xdim, bits
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::generators::json::*;
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
    fn ean_13_as_json() {
        let ean13 = EAN13::new("750103131130").expect("Failed to create EAN13 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&ean13.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,1,1,0,0,0,1,0,1,0,0,1,1,1,0,0,1,1,0,0,1,0,1,0,0,1,1,1,0,1,1,1,1,0,1,0,1,1,0,0,1,1,0,1,0,1,0,1,0,0,0,0,1,0,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1,0,0,0,0,1,0,1,1,1,0,0,1,0,1,1,1,0,1,0,0,1,0,1]}".trim());
    }

    #[test]
    fn ean_13_as_json_small_height_double_width() {
        let ean13 = EAN13::new("750103131130").expect("Failed to create EAN13 barcode");
        let json = JSON { height: 6, xdim: 2 };
        let generated = json
            .generate(&ean13.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":6,\"xdim\":2,\"encoding\":[1,0,1,0,1,1,0,0,0,1,0,1,0,0,1,1,1,0,0,1,1,0,0,1,0,1,0,0,1,1,1,0,1,1,1,1,0,1,0,1,1,0,0,1,1,0,1,0,1,0,1,0,0,0,0,1,0,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1,0,0,0,0,1,0,1,1,1,0,0,1,0,1,1,1,0,1,0,0,1,0,1]}".trim());
    }

    #[test]
    fn ean_8_as_json() {
        let ean8 = EAN8::new("1234567").expect("Failed to create EAN8 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&ean8.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,0,1,1,0,0,1,0,0,1,0,0,1,1,0,1,1,1,1,0,1,0,1,0,0,0,1,1,0,1,0,1,0,1,0,0,1,1,1,0,1,0,1,0,0,0,0,1,0,0,0,1,0,0,1,1,1,0,0,1,0,1,0,1]}".trim());
    }

    #[test]
    fn ean_8_as_json_small_height_double_width() {
        let ean8 = EAN8::new("1234567").expect("Failed to create EAN8 barcode");
        let json = JSON { height: 5, xdim: 2 };
        let generated = json
            .generate(&ean8.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":5,\"xdim\":2,\"encoding\":[1,0,1,0,0,1,1,0,0,1,0,0,1,0,0,1,1,0,1,1,1,1,0,1,0,1,0,0,0,1,1,0,1,0,1,0,1,0,0,1,1,1,0,1,0,1,0,0,0,0,1,0,0,0,1,0,0,1,1,1,0,0,1,0,1,0,1]}".trim());
    }

    #[test]
    fn code_93_as_json() {
        let code93 = Code93::new("MONKEYMAGIC").expect("Failed to create Code93 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&code93.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,1,1,1,1,0,1,0,1,0,0,1,1,0,0,1,0,0,1,0,1,1,0,0,1,0,1,0,0,0,1,1,0,1,0,0,0,1,1,0,1,0,1,1,0,0,1,0,0,1,0,1,0,0,1,1,0,1,1,0,1,0,1,0,0,1,1,0,0,1,1,0,1,0,1,0,0,0,1,0,1,1,0,1,0,0,0,1,0,1,1,0,0,0,1,0,1,1,0,1,0,0,0,1,0,1,0,0,1,1,0,1,1,0,1,0,1,0,0,1,0,0,0,1,0,1,0,1,1,1,1,0,1]}".trim());
    }

    #[test]
    fn code_93_as_json_small_height_double_weight() {
        let code93 = Code93::new("1234").expect("Failed to create Code93 barcode");
        let json = JSON { height: 7, xdim: 2 };
        let generated = json
            .generate(&code93.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,0,1,0,1,1,1,1,0,1,0,1,0,0,1,0,0,0,1,0,1,0,0,0,1,0,0,1,0,1,0,0,0,0,1,0,1,0,0,1,0,1,0,0,0,1,0,0,0,1,1,0,1,0,1,0,1,0,0,0,0,1,0,1,0,1,0,1,1,1,1,0,1]}".trim());
    }

    #[test]
    fn code_39_as_json() {
        let code39 = Code39::new("TEST8052").expect("Failed to create Code39 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&code39.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,0,1,0,1,1,0,1,1,0,1,0,1,0,1,0,1,1,0,1,1,0,0,1,0,1,1,0,1,0,1,1,0,0,1,0,1,0,1,0,1,1,0,1,0,1,1,0,0,1,0,1,0,1,0,1,1,0,1,1,0,0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,0,1,0,1,0,0,1,1,0,1,1,0,1,0,1,1,0,1,0,0,1,1,0,1,0,1,0,1,0,1,1,0,0,1,0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,1,0,1]}".trim());
    }

    #[test]
    fn code_39_as_json_small_height_double_weight() {
        let code39 = Code39::new("1234").expect("Failed to create Code39 barcode");
        let json = JSON { height: 7, xdim: 2 };
        let generated = json
            .generate(&code39.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,0,0,1,0,1,1,0,1,1,0,1,0,1,1,0,1,0,0,1,0,1,0,1,1,0,1,0,1,1,0,0,1,0,1,0,1,1,0,1,1,0,1,1,0,0,1,0,1,0,1,0,1,0,1,0,0,1,1,0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,1,0,1]}".trim());
    }

    #[test]
    fn codabar_as_json() {
        let codabar = Codabar::new("A98B").expect("Failed to create Codabar barcode");
        let json = JSON::new();
        let generated = json
            .generate(&codabar.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,0,1,0,0,1,0,1,1,0,1,0,0,1,0,1,0,1,0,0,1,1,0,1,0,1,0,1,0,1,0,0,1,0,0,1,1]}".trim());
    }

    #[test]
    fn codabar_as_json_small_height_double_weight() {
        let codabar = Codabar::new("A40156B").expect("Failed to create Codabar barcode");
        let json = JSON { height: 7, xdim: 2 };
        let generated = json
            .generate(&codabar.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,0,1,1,0,0,1,0,0,1,0,1,0,1,1,0,1,0,0,1,0,1,0,1,0,1,0,0,1,1,0,1,0,1,0,1,1,0,0,1,0,1,1,0,1,0,1,0,0,1,0,1,0,0,1,0,1,0,1,1,0,1,0,1,0,0,1,0,0,1,1]}".trim());
    }

    #[test]
    fn code_128_as_json() {
        let code128 =
            Code128::new("HELLO", CharacterSet::A).expect("Failed to create Code128 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&code128.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,1,0,1,0,0,0,0,1,0,0,1,1,0,0,0,1,0,1,0,0,0,1,0,0,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,1,0,1,1,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,0,0,1,1,1,0,1,0,1,1]}".trim());
    }

    #[test]
    fn code_128_as_json_small_height_double_weight() {
        let code128 =
            Code128::new("HELLO", CharacterSet::A).expect("Failed to create Code128 barcode");
        let json = JSON { height: 7, xdim: 2 };
        let generated = json
            .generate(&code128.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,1,0,1,0,0,0,0,1,0,0,1,1,0,0,0,1,0,1,0,0,0,1,0,0,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,1,0,1,1,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,0,0,1,1,1,0,1,0,1,1]}".trim());
    }

    #[test]
    fn ean2_as_json() {
        let ean2 = EANSUPP::new("34").expect("Failed to create EAN2 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&ean2.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(
            generated,
            "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,1,0,0,0,0,1,0,1,0,1,0,0,0,1,1]}"
                .trim()
        );
    }

    #[test]
    fn ean5_as_json() {
        let ean5 = EANSUPP::new("50799").expect("Failed to create EAN5 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&ean5.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,1,1,0,0,0,1,0,1,0,1,0,0,1,1,1,0,1,0,0,1,0,0,0,1,0,1,0,0,0,1,0,1,1,0,1,0,0,0,1,0,1,1]}".trim());
    }

    #[test]
    fn itf_as_json() {
        let itf = TF::interleaved("12345").expect("Failed to create ITF barcode");
        let json = JSON::new();
        let generated = json
            .generate(&itf.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,1,1,1,0,1,0,0,0,1,0,1,0,1,1,1,0,0,0,1,1,1,0,1,1,1,0,1,0,0,0,1,0,1,0,0,0,1,1,1,0,1,0,1,1,1,0,1,0,0,0,1,0,0,0,1,1,0,1]}".trim());
    }

    #[test]
    fn code11_as_json() {
        let code11 = Code11::new("111-999-8").expect("Failed to create Code11 barcode");
        let json = JSON::new();
        let generated = json
            .generate(&code11.encode()[..])
            .expect("Failed to generate JSON");

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,0,1,0,1,1,0,1,0,1,1,0,1,1,0,1,0,1,1,0,1,1,0,1,0,1,1,0,1,0,1,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1,0,1,0,1,1,0,1,0,1,1,0,1,0,0,1,0,1,0,1,0,1,1,0,1,0,1,1,0,0,1]}".trim());
    }
}
