use egui::Color32;

use crate::{
    data::palette::{Palette, PaletteIndex},
    tiled::tiled_color::TiledColor,
};

/// A color in the sRGB color space, with an alpha
/// component. The alpha is NOT premultiplied.
/// R, G and B components on a scale of 0-255, this
/// is equivalent to a CSS rgb/rgba color with the same components
/// specified as `<number>`s
/// Alpha as a value from 0 to 255, where 0 is fully transparent
/// and 255 is fully opaque - this corresponds to the range of
/// 0.0 to 1.0 for the alpha of a CSS rgb/rgba color.
/// We use our own type to make it very clear what the color is,
/// we could also use Color32, but that may have alpha premultiplied
/// (most of the time) or unmultiplied (if we've specifically
/// converted it). We store colors unmultiplied since we
/// want to present this to the user for editing etc, and
/// to avoid loss of precision from converting from premultiplied.
#[derive(
    Default, Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash,
)]
pub struct UserColor([u8; 4]);

impl UserColor {
    pub const BLACK: UserColor = UserColor([0, 0, 0, 255]);
    pub const WHITE: UserColor = UserColor([255, 255, 255, 255]);
    pub const TRANSPARENT: UserColor = UserColor([0, 0, 0, 0]);
    pub const PLACEHOLDER: UserColor = UserColor([255, 0, 255, 255]);

    /// A new color with specified components (see main docs, sRGB plus unmultiplied alpha 0-255)
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        UserColor([r, g, b, a])
    }

    /// A new color with specified components (see main docs, sRGB), fully opaque
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        UserColor([r, g, b, 255])
    }

    /// Convert to a [`Color32`] in the normal premultiplied form - this can be
    /// used directly by egui, for example to give the vertex colors of quads in a tile etc.
    /// This should NOT be shown to the user since it doesn't match the normal (e.g. CSS)
    /// sRGB + Alpha format with unmultiplied alpha.
    pub fn as_premultiplied_color32(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.0[0], self.0[1], self.0[2], self.0[3])
    }

    /// Convert to a [`Color32`] in the non-standard unmultipled form - this can be
    /// used with an egui color picker for example. Otherwise not a huge amount of use.
    pub fn as_unmultiplied_color32(&self) -> Color32 {
        Color32::from_rgba_premultiplied(self.0[0], self.0[1], self.0[2], self.0[3])
    }

    /// Produce a new color with an additional opacity applied - the current alpha is
    /// multiplied by the opacity and clamped to the range 0 to 255.
    /// Note that opacity is a float value from 0.0 for fully transparent to 1.0 for
    /// fully opaque.
    pub fn with_opacity(&self, opacity: f32) -> UserColor {
        // Note that cast from f32 to u8 is saturating, result will be clamped to 0-255 range, which is what we want
        let alpha = ((self.a() as f32) * opacity) as u8;
        UserColor::new(self.r(), self.g(), self.b(), alpha)
    }

    /// Produce a new color with an additional optional opacity applied.
    /// If opacity is None, the color is unchanged, if it is Some(value) the current alpha is
    /// multiplied by the value and clamped to the range 0 to 255.
    /// Note that the opacity value is a float value from 0.0 for fully transparent to 1.0 for
    /// fully opaque.
    pub fn with_optional_opacity(&self, opacity: Option<f32>) -> UserColor {
        match opacity {
            Some(opacity) => self.with_opacity(opacity),
            None => *self,
        }
    }

    pub fn slice_mut(&mut self) -> &mut [u8; 4] {
        &mut self.0
    }

    pub fn as_slice(&self) -> [u8; 4] {
        self.0
    }

    pub fn as_css_string(&self) -> String {
        if self.0[3] == 255 {
            format!("#{:02X}{:02X}{:02X}", self.0[0], self.0[1], self.0[2])
        } else {
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                self.0[0], self.0[1], self.0[2], self.0[3]
            )
        }
    }

    pub fn r_mut(&mut self) -> &mut u8 {
        &mut self.0[0]
    }

    pub fn g_mut(&mut self) -> &mut u8 {
        &mut self.0[1]
    }

    pub fn b_mut(&mut self) -> &mut u8 {
        &mut self.0[2]
    }

    pub fn a_mut(&mut self) -> &mut u8 {
        &mut self.0[3]
    }

    pub fn r(&self) -> u8 {
        self.0[0]
    }

    pub fn g(&self) -> u8 {
        self.0[1]
    }

    pub fn b(&self) -> u8 {
        self.0[2]
    }

    pub fn a(&self) -> u8 {
        self.0[3]
    }
}

impl From<UserColor> for image::Rgba<u8> {
    fn from(value: UserColor) -> Self {
        image::Rgba(value.0)
    }
}

impl From<&UserColor> for image::Rgba<u8> {
    fn from(value: &UserColor) -> Self {
        image::Rgba(value.0)
    }
}

impl From<TiledColor> for UserColor {
    fn from(value: TiledColor) -> Self {
        UserColor([value.r, value.g, value.b, value.a])
    }
}

impl From<&TiledColor> for UserColor {
    fn from(value: &TiledColor) -> Self {
        UserColor([value.r, value.g, value.b, value.a])
    }
}

impl From<UserColor> for TiledColor {
    fn from(value: UserColor) -> Self {
        TiledColor {
            r: value.r(),
            g: value.g(),
            b: value.b(),
            a: value.a(),
        }
    }
}

impl From<&UserColor> for TiledColor {
    fn from(value: &UserColor) -> Self {
        TiledColor {
            r: value.r(),
            g: value.g(),
            b: value.b(),
            a: value.a(),
        }
    }
}

impl From<csscolorparser::Color> for UserColor {
    fn from(value: csscolorparser::Color) -> Self {
        UserColor(value.to_rgba8())
    }
}

/// Specifies the color of a tile.
/// This is used to process the underlying tile image by multiplying red, green and blue
/// components of each pixel of the image by the tile color components. In addition, the
/// image pixel alpha values are multiplied by the tile color alpha.
#[derive(Default, Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum TileColor {
    /// The default color for a tile, this should be the tileset image with
    /// no tint or processing. This should be equivalent to using a tile
    /// color of opaque white (full value for each of red, green, blue
    /// and alpha components)
    #[default]
    Default,
    /// The tile color is taken from the relevant palette for the tile,
    /// using the given index.
    Palette { index: PaletteIndex },
    /// The tile color is taken from the relevant palette for the tile,
    /// using an index for the foreground (`fg`) and the background (`bg`).
    /// If this tile has foreground and background areas, then they are colored
    /// according to the specified palette colors, if the tile does not have
    /// such areas then the entire tile uses the foreground color, making this
    /// equivalent to [`TileColor::Palette`] with `index` equal to `fg`.
    PaletteFgBg { fg: PaletteIndex, bg: PaletteIndex },
    /// The tile color is specified directly by a [`UserColor`]
    UserColor(UserColor),
}

impl TileColor {
    pub fn from_palette_index(index: PaletteIndex) -> Self {
        Self::Palette { index }
    }

    pub fn as_foreground_color32_premultiplied(&self, palette: &Palette) -> Color32 {
        self.as_foreground_user_color(palette)
            .as_premultiplied_color32()
    }

    pub fn as_foreground_user_color(&self, palette: &Palette) -> UserColor {
        match self {
            TileColor::Default => UserColor::WHITE,
            TileColor::Palette { index } => palette
                .color_option(*index)
                .unwrap_or(UserColor::PLACEHOLDER),
            TileColor::PaletteFgBg { fg, bg: _ } => {
                palette.color_option(*fg).unwrap_or(UserColor::PLACEHOLDER)
            }
            TileColor::UserColor(color) => *color,
        }
    }

    pub fn as_foreground_slice(&self, palette: &Palette) -> [u8; 4] {
        self.as_foreground_user_color(palette).0
    }
}
