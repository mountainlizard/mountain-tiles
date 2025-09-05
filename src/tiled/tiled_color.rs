use std::fmt;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TiledColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl TiledColor {
    pub fn from_hex_rgb(hex: &str) -> Option<TiledColor> {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(TiledColor { r, g, b, a: 255 })
        } else {
            None
        }
    }

    pub fn from_hex_rgba(hex: &str) -> Option<TiledColor> {
        if hex.len() == 8 {
            let a = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let r = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let g = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let b = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(TiledColor { r, g, b, a })
        } else {
            None
        }
    }

    fn as_hex_string(&self) -> String {
        if self.a < 255 {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.a, self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }
    }
}

impl Serialize for TiledColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_hex_string())
    }
}

impl<'de> Deserialize<'de> for TiledColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TiledColorVisitor)
    }
}

struct TiledColorVisitor;

impl Visitor<'_> for TiledColorVisitor {
    type Value = TiledColor;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a valid Tiled color, in format '#RRGGBB' or '#AARRGGBB' where capital letters are hexadecimal values of Red, Green, Blue and Alpha, and '#' is an optional hash character")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let s = v.trim();
        // Leading # appears to be optional - it's not used for
        // `trans` attribute of `<image>` in .tsx files, but is
        // used for `<map>` `backgroundcolor` and `<layer>` `tintcolor`
        let hex = s.strip_prefix("#").unwrap_or(s);

        match hex.len() {
            6 => TiledColor::from_hex_rgb(hex).ok_or(serde::de::Error::custom(
                "Tiled color contains invalid hex for RRGGBB format",
            )),
            8 => TiledColor::from_hex_rgba(hex).ok_or(serde::de::Error::custom(
                "Tiled color contains invalid hex for AARRGGBB format",
            )),
            _ => Err(serde::de::Error::custom(
                "Tiled color has wrong number of hex digits, must be 6 or 8",
            )),
        }
    }
}
