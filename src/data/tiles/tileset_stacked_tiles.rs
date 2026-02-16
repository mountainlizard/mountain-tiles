use crate::{
    data::{
        tiles::{
            tile_color::{TileColor, UserColor},
            Tile, TileIndex, TileSource, Tiles,
        },
        tilesets::{TilesetId, Tilesets},
    },
    geom::{
        transform::Transform,
        u32pos2::U32Pos2,
        u32size2::{u32size2, U32Size2},
    },
};

struct TilesetIndexing {
    tileset_id: TilesetId,
    firstgid: u32,
    tile_count: u32,
}

pub struct TilesetStackedTiles {
    tile_size: U32Size2,
    indexing: Vec<TilesetIndexing>,
    map_size: U32Size2,
}

impl TilesetStackedTiles {
    pub fn new(tilesets: &Tilesets) -> Self {
        // Build tileset first gids
        let mut firstgid = 1;
        let mut indexing = Vec::new();

        for tileset in tilesets.iter() {
            let tile_count = tileset.size_in_tiles.area();
            indexing.push(TilesetIndexing {
                tileset_id: tileset.id(),
                firstgid,
                tile_count,
            });
            firstgid += tile_count;
        }

        let tile_size = tilesets
            .first()
            .map(|ts| ts.tile_size)
            .unwrap_or(u32size2(8, 8));

        let total_tile_count = tilesets.iter().map(|ts| ts.size_in_tiles.area()).sum();

        Self {
            tile_size,
            indexing,
            map_size: u32size2(1, total_tile_count),
        }
    }
}

impl Tiles for TilesetStackedTiles {
    fn background(&self) -> UserColor {
        UserColor::TRANSPARENT
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
        1.0
    }

    fn gap(&self) -> U32Size2 {
        U32Size2::ZERO
    }

    fn tile(&self, layer: usize, pos: U32Pos2) -> Option<Tile> {
        if layer == 0 && pos.x == 0 {
            // We're using tiled-style index, so it should start from 1
            let flat_tile_index = pos.y + 1;

            if let Some(index) = self
                .indexing
                .iter()
                .rev()
                .find(|&index| index.firstgid <= flat_tile_index)
            {
                let tile_index = flat_tile_index - index.firstgid;
                if tile_index < index.tile_count {
                    Some(Tile {
                        source: TileSource {
                            tileset_id: index.tileset_id,
                            tile_index: TileIndex::new(tile_index),
                        },
                        color: TileColor::Default,
                        transform: Transform::None,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn set_tile(&mut self, _layer: usize, _pos: U32Pos2, _tile: Option<Tile>) -> bool {
        false
    }
}
