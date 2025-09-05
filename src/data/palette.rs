use std::slice::{Iter, IterMut};

use camino::Utf8PathBuf;
use egui::ahash::{HashSet, HashSetExt};
use image::{ImageReader, RgbaImage};

use crate::data::tiles::tile_color::{TileColor, UserColor};

#[derive(
    Debug, serde::Deserialize, serde::Serialize, Clone, Copy, Hash, PartialEq, Eq, Default,
)]
pub struct PaletteIndex(u32);

impl PaletteIndex {
    #[inline(always)]
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub fn index(&self) -> u32 {
        self.0
    }

    pub fn limited_to_palette(&self, palette: &Palette) -> Self {
        if self.index() >= palette.len() {
            PaletteIndex::new(palette.len() - 1)
        } else {
            *self
        }
    }

    pub fn limit_to_palette(&mut self, palette: &Palette) {
        if self.0 >= palette.len() {
            self.0 = palette.len() - 1;
        }
    }

    pub fn previous_within_palette(&self, palette: &Palette) -> Self {
        if self.0 > 0 {
            PaletteIndex::new(self.0 - 1)
        } else {
            PaletteIndex::new(palette.len() - 1)
        }
    }

    pub fn next_within_palette(&self, palette: &Palette) -> Self {
        if self.0 < palette.len() - 1 {
            PaletteIndex::new(self.0 + 1)
        } else {
            PaletteIndex::new(0)
        }
    }
}

#[inline(always)]
pub fn palette_index(index: u32) -> PaletteIndex {
    PaletteIndex(index)
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct Palette {
    colors: Vec<UserColor>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: vec![UserColor::WHITE],
        }
    }
}

impl Palette {
    /// Load an image from given path, and use the colors in it as a palette.
    /// The pixels of the image are used in order from x = 0 to width then y = 0 to height
    /// Exact duplicate colors are ignored, only the first instance is used (note that
    /// if colors are even one bit different they will be considered to be different)
    /// The simplest format for the image is a single row of pixels with one pixel of each color, in order.
    /// It's also possible to use multiple rows, and/or to use larger squares or rectangles of the colors.
    /// Be careful not to scale or process the image so that it has blurred edges with
    /// additional unwanted colors.
    pub fn from_image_by_path(path: Utf8PathBuf) -> eyre::Result<Palette> {
        let mut colors = vec![];
        let mut imported = HashSet::new();
        let dynamic_image = ImageReader::open(path)?.decode()?;
        let image = dynamic_image.into_rgba8();
        for pixel in image.pixels() {
            let color = UserColor::new(pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]);
            if imported.insert(color) {
                colors.push(color);
            }
        }
        Ok(Self::new(colors))
    }

    pub fn write_to_image_by_path(&self, path: Utf8PathBuf) -> eyre::Result<()> {
        let mut image = RgbaImage::new(self.len(), 1);
        for (x, color) in self.colors().enumerate() {
            let rgba: image::Rgba<u8> = color.into();
            image.put_pixel(x as u32, 0, rgba);
        }
        image.save(path)?;
        Ok(())
    }

    pub fn new(colors: Vec<UserColor>) -> Self {
        Self { colors }
    }

    /// Return the color at given index, or [`None`] if index is outside palette
    pub fn color_option(&self, i: PaletteIndex) -> Option<UserColor> {
        self.colors.get(i.index() as usize).copied()
    }

    pub fn len(&self) -> u32 {
        self.colors.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn colors(&self) -> Iter<'_, UserColor> {
        self.colors.iter()
    }

    pub fn colors_mut(&mut self) -> IterMut<'_, UserColor> {
        self.colors.iter_mut()
    }

    pub fn insert_color(&mut self, index: PaletteIndex, color: UserColor) -> PaletteIndex {
        let insert_index = (index.index() + 1).min(self.len());
        self.colors.insert(insert_index as usize, color);
        palette_index(insert_index)
    }

    pub fn delete_color(&mut self, index: PaletteIndex) -> PaletteIndex {
        let delete_index = index.index().min(self.len() - 1);
        self.colors.remove(delete_index as usize);
        palette_index(delete_index.min(self.len() - 1))
    }

    pub fn can_move_color_previous(&self, index: PaletteIndex) -> bool {
        index.index() > 0 && index.index() < self.len()
    }

    pub fn can_move_color_next(&self, index: PaletteIndex) -> bool {
        index.index() < self.len() - 1
    }

    pub fn move_color_previous(&mut self, index: PaletteIndex) -> PaletteIndex {
        if self.can_move_color_previous(index) {
            self.colors
                .swap((index.index() - 1) as usize, index.index() as usize);
            PaletteIndex::new(index.index() - 1)
        } else {
            index
        }
    }

    pub fn move_color_next(&mut self, index: PaletteIndex) -> PaletteIndex {
        if self.can_move_color_next(index) {
            self.colors
                .swap(index.index() as usize, (index.index() + 1) as usize);
            PaletteIndex::new(index.index() + 1)
        } else {
            index
        }
    }

    pub fn is_tilecolor_available(&self, color: &TileColor) -> bool {
        match color {
            TileColor::Default => true,
            TileColor::Palette { index } => index.index() < self.len(),
            TileColor::UserColor(_user_color) => true,
        }
    }
}
