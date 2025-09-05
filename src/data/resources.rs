use crate::{data::palette::Palette, data::tiles::Tile, data::tilesets::Tilesets};

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq)]
pub struct Resources {
    pub tilesets: Tilesets,
    pub palette: Palette,
}

impl Resources {
    pub fn tilesets(&self) -> &Tilesets {
        &self.tilesets
    }
    pub fn tilesets_mut(&mut self) -> &mut Tilesets {
        &mut self.tilesets
    }
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn is_tile_valid(&self, tile: &Tile) -> bool {
        self.palette.is_tilecolor_available(&tile.color)
            && self.tilesets.is_tile_source_available(tile.source)
    }
}

#[derive(Clone, PartialEq)]
pub struct TileResourceLocation {
    pub map_name: String,
    pub layer_name: String,
}

#[derive(Clone, PartialEq)]
pub struct TileResourceUse {
    pub locations: Vec<TileResourceLocation>,
    pub tile_count: usize,
}

impl TileResourceUse {
    pub fn locations_to_string(&self) -> String {
        self.locations
            .iter()
            .map(|l| format!("{}: {}", l.map_name, l.layer_name))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{palette::PaletteIndex, tiles::tile_color::UserColor};

    use super::*;

    #[test]
    fn defaults_make_sense() -> eyre::Result<()> {
        let resources = Resources::default();
        log::debug!("{}", serde_json::to_string_pretty(&resources)?);

        assert_eq!(resources.tilesets.len(), 0);
        assert_eq!(resources.palette.len(), 1);
        assert_eq!(
            resources.palette.color_option(PaletteIndex::new(0)),
            Some(UserColor::WHITE)
        );

        Ok(())
    }
}
