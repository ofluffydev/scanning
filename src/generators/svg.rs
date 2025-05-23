//! Functionality for generating SVG representations of barcodes.
//!
//! An SVG can be constructed via the standard constructor pattern
//! or via a constructor method if you want default values.
//!
//! For example:
//!
//! ```rust
//! use barcoders::generators::svg::*;
//!
//! // Specify your own struct fields.
//! let svg = SVG{height: 80,
//!               xdim: 1,
//!               background: Color{rgba: [255, 0, 0, 255]},
//!               foreground: Color::black(),
//!               xmlns: Some(String::from("http://www.w3.org/2000/svg"))};
//!
//! // Or use the constructor for defaults (you must specify the height).
//! let svg = SVG::new(100)
//!               .xdim(2)
//!               .background(Color::white())
//!               .foreground(Color::black())
//!               .xmlns(String::from("http://www.w3.org/2000/svg"));
//! ```

use crate::error::Result;
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

trait ToHex {
    fn to_hex(self) -> String;

    fn format_hex(n: u8) -> String {
        format!(
            "{}{}",
            Self::to_hex_digit(n / 16),
            Self::to_hex_digit(n % 16)
        )
    }

    fn to_hex_digit(n: u8) -> char {
        match n {
            d if d < 10 => (d + 48) as char,
            d if d < 16 => (d + 87) as char,
            _ => '0',
        }
    }
}

/// Represents a RGBA color for the barcode foreground and background.
#[derive(Copy, Clone, Debug)]
pub struct Color {
    /// Reg, Green, Blue, Alpha value.
    pub rgba: [u8; 4],
}

impl Color {
    /// Constructor.
    #[must_use]
    pub const fn new(rgba: [u8; 4]) -> Self {
        Self { rgba }
    }

    /// Constructor for black (#000000).
    #[must_use]
    pub const fn black() -> Self {
        Self::new([0, 0, 0, 255])
    }

    /// Constructor for white (#FFFFFF).
    #[must_use]
    pub const fn white() -> Self {
        Self::new([255, 255, 255, 255])
    }

    fn to_opacity(self) -> String {
        format!("{:.*}", 2, (f64::from(self.rgba[3]) / 255.0))
    }
}

impl ToHex for Color {
    fn to_hex(self) -> String {
        self.rgba
            .iter()
            .take(3)
            .map(|&c| Self::format_hex(c))
            .collect()
    }
}

/// The SVG barcode generator type.
#[derive(Clone, Debug)]
pub struct SVG {
    /// The height of the barcode (```self.height``` pixels high for SVG).
    pub height: u32,
    /// The X dimension. Specifies the width of the "narrow" bars.
    /// For SVG, each will be ```self.xdim``` pixels wide.
    pub xdim: u32,
    /// The RGBA color for the foreground.
    pub foreground: Color,
    /// The RGBA color for the foreground.
    pub background: Color,
    /// The XML namespace
    pub xmlns: Option<String>,
}

impl SVG {
    /// Returns a new SVG with default values.
    #[must_use]
    pub const fn new(height: u32) -> Self {
        Self {
            height,
            xdim: 1,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
            xmlns: None,
        }
    }

    /// Set the xml namespace (xmlns) of the SVG
    #[must_use]
    pub fn xmlns(mut self, xmlns_uri: String) -> Self {
        self.xmlns = Some(xmlns_uri);
        self
    }

    /// Set the x dimensional bar width
    #[must_use]
    pub const fn xdim(mut self, xdim: u32) -> Self {
        self.xdim = xdim;
        self
    }

    /// Set the foreground (bar) color
    #[must_use]
    pub const fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    /// Set the background color
    #[must_use]
    pub const fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    fn rect(&self, style: u8, offset: u32, width: u32) -> String {
        let fill = match style {
            1 => self.foreground,
            _ => self.background,
        };

        let opacity = match &fill.to_opacity()[..] {
            "1.00" | "1" => String::new(),
            o => format!(" fill-opacity=\"{o}\" "),
        };

        format!(
            "<rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#{}\"{}/>",
            offset,
            width,
            self.height,
            fill.to_hex(),
            opacity
        )
    }

    /// Generates the given barcode.
    ///
    /// Returns a `Result<String, Error>` containing the SVG data or an error message.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provided barcode data is invalid or cannot
    /// be processed into a valid SVG representation.
    pub fn generate<T: AsRef<[u8]>>(&self, barcode: T) -> Result<String> {
        let barcode = barcode.as_ref();
        let width = match u32::try_from(barcode.len()) {
            Ok(len) => len * self.xdim,
            Err(_) => return Err(crate::error::Error::Length),
        };
        let rects: String = barcode
            .iter()
            .enumerate()
            .filter(|&(_, &n)| n == 1)
            .map(|(i, &n)| {
                Ok(match u32::try_from(i) {
                    Ok(offset) => self.rect(n, offset * self.xdim, self.xdim),
                    Err(_) => return Err(crate::error::Error::Conversion),
                })
            })
            .collect::<Result<String>>()?;

        let xmlns = self
            .xmlns
            .as_ref()
            .map_or_else(String::new, |xmlns| format!("xmlns=\"{xmlns}\" "));

        Ok(format!(
            "<svg version=\"1.1\" {x}viewBox=\"0 0 {w} {h}\">{s}{r}</svg>",
            x = xmlns,
            w = width,
            h = self.height,
            s = self.rect(0, 0, width),
            r = rects
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::generators::svg::*;
    use crate::sym::codabar::*;
    use crate::sym::code11::*;
    use crate::sym::code128::*;
    use crate::sym::code39::*;
    use crate::sym::code93::*;
    use crate::sym::ean13::*;
    use crate::sym::ean8::*;
    use crate::sym::ean_supp::*;
    use crate::sym::tf::*;
    #[cfg(feature = "std")]
    use std::fs::File;
    #[cfg(feature = "std")]
    use std::io::prelude::*;
    #[cfg(feature = "std")]
    use std::io::BufWriter;
    #[cfg(feature = "std")]
    use std::path::Path;

    const TEST_DATA_BASE: &str = "./target/debug";
    const WRITE_TO_FILE: bool = true;

    #[cfg(feature = "std")]
    fn write_file(data: &str, file: &'static str) {
        let path = open_file(file);
        let mut writer = BufWriter::new(path);
        writer
            .write_all(data.as_bytes())
            .expect("Failed to write data to file");
    }

    #[cfg(not(feature = "std"))]
    fn write_file(_data: &str, _file: &'static str) {}

    #[cfg(feature = "std")]
    fn open_file(name: &'static str) -> File {
        File::create(Path::new(&format!("{TEST_DATA_BASE}/{name}")[..]))
            .expect("Failed to create file")
    }

    #[test]
    fn ean_13_as_svg() {
        let ean13 = EAN13::new("750103131130").expect("Failed to create EAN13 barcode");
        let svg = SVG::new(80);
        let generated = svg
            .generate(&ean13.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "ean13.svg");
        }

        assert_eq!(generated.len(), 2890);
    }

    #[test]
    fn colored_ean_13_as_svg() {
        let ean13 = EAN13::new("750103131130").expect("Failed to create EAN13 barcode");
        let svg = SVG {
            height: 80,
            xdim: 1,
            background: Color {
                rgba: [255, 0, 0, 255],
            },
            foreground: Color {
                rgba: [0, 0, 255, 255],
            },
            xmlns: None,
        };
        let generated = svg
            .generate(&ean13.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "ean13_colored.svg");
        }

        assert_eq!(generated.len(), 2890);
    }

    #[test]
    fn colored_semi_transparent_ean_13_as_svg() {
        let ean13 = EAN13::new("750103131130").expect("Failed to create EAN13 barcode");
        let svg = SVG {
            height: 70,
            xdim: 1,
            background: Color {
                rgba: [255, 0, 0, 128],
            },
            foreground: Color {
                rgba: [0, 0, 255, 128],
            },
            xmlns: None,
        };
        let generated = svg
            .generate(&ean13.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "ean13_colored_semi_transparent.svg");
        }

        assert_eq!(generated.len(), 3940);
    }

    #[test]
    fn ean_8_as_svg() {
        let ean8 = EAN8::new("9998823").expect("Failed to create EAN8 barcode");
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg
            .generate(&ean8.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "ean8.svg");
        }

        assert_eq!(generated.len(), 1956);
    }

    #[test]
    fn code39_as_svg() {
        let code39 = Code39::new("IGOT99PROBLEMS").expect("Failed to create Code39 barcode");
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg
            .generate(&code39.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "code39.svg");
        }

        assert_eq!(generated.len(), 6574);
    }

    #[test]
    fn code93_as_svg() {
        let code93 = Code93::new("IGOT99PROBLEMS").expect("Failed to create Code93 barcode");
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg
            .generate(&code93.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "code93.svg");
        }

        assert_eq!(generated.len(), 4493);
    }

    #[test]
    fn codabar_as_svg() {
        let codabar = Codabar::new("A12----34A").expect("Failed to create Codabar barcode");
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg
            .generate(&codabar.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "codabar.svg");
        }

        assert_eq!(generated.len(), 2985);
    }

    #[test]
    fn code128_as_svg() {
        let code128 =
            Code128::new("HIĆ345678", CharacterSet::A).expect("Failed to create Code128 barcode");
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg
            .generate(&code128.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "code128.svg");
        }

        assert_eq!(generated.len(), 2758);
    }

    #[test]
    fn ean_2_as_svg() {
        let ean2 = EANSUPP::new("78").expect("Failed to create EAN2 barcode");
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg
            .generate(&ean2.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "ean2.svg");
        }

        assert_eq!(generated.len(), 760);
    }

    #[test]
    fn itf_as_svg() {
        let itf =
            TF::interleaved("1234123488993344556677118").expect("Failed to create ITF barcode");
        let svg = SVG {
            height: 80,
            xdim: 1,
            background: Color::black(),
            foreground: Color::white(),
            xmlns: None,
        };
        let generated = svg
            .generate(&itf.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "itf.svg");
        }

        assert_eq!(generated.len(), 7123);
    }

    #[test]
    fn code11_as_svg() {
        let code11 = Code11::new("9988-45643201").expect("Failed to create Code11 barcode");
        let svg = SVG {
            height: 80,
            xdim: 1,
            background: Color::black(),
            foreground: Color::white(),
            xmlns: None,
        };
        let generated = svg
            .generate(&code11.encode()[..])
            .expect("Failed to generate SVG");

        if WRITE_TO_FILE {
            write_file(&generated[..], "code11.svg");
        }

        assert_eq!(generated.len(), 4219);
    }
}
