use crate::{
    data::tiles::{
        tile_color::{TileColor, UserColor},
        Tile, TileIndex, TileSource, Tiles,
    },
    data::tilesets::TilesetId,
    geom::transform::Transform,
    geom::u32pos2::U32Pos2,
    geom::u32size2::U32Size2,
};

// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TilesetTiles {
    pub foreground: UserColor,
    pub background: UserColor,
    pub tileset_id: TilesetId,
    pub tile_size: U32Size2,
    pub map_size: U32Size2,
    pub scale: f32,
    pub gap: U32Size2,
}

impl Tiles for TilesetTiles {
    fn background(&self) -> UserColor {
        self.background
    }

    fn layer_count(&self) -> usize {
        1
    }

    fn layer_opacity(&self, _layer: usize) -> Option<f32> {
        None
    }

    fn tile_size(&self) -> U32Size2 {
        self.tile_size
    }

    fn map_size(&self) -> U32Size2 {
        self.map_size
    }

    fn scale(&self) -> f32 {
        self.scale
    }

    fn gap(&self) -> U32Size2 {
        self.gap
    }

    fn tile(&self, layer: usize, pos: U32Pos2) -> Option<Tile> {
        if layer == 0 {
            pos.linear_index(self.map_size).map(|tile_index| Tile {
                source: TileSource {
                    tileset_id: self.tileset_id,
                    tile_index: TileIndex::new(tile_index),
                },
                color: TileColor::UserColor(self.foreground),
                transform: Transform::None,
            })
        } else {
            None
        }
    }

    fn set_tile(&mut self, _layer: usize, _pos: U32Pos2, _tile: Option<Tile>) -> bool {
        false
    }
}
