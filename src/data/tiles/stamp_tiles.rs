use crate::{
    data::stamp::{Stamp, TileLocation},
    data::tiles::{tile_color::UserColor, Tile, Tiles},
    geom::i32pos2::I32Pos2,
    geom::u32pos2::U32Pos2,
    geom::u32size2::U32Size2,
};

pub struct StampTiles<'a, T: Tiles> {
    pub stamp: &'a Stamp,
    pub offset: I32Pos2,
    pub layer_to_stamp_layer: &'a [Option<usize>],
    pub inner_tiles: &'a T,
}

impl<T: Tiles> Tiles for StampTiles<'_, T> {
    fn background(&self) -> UserColor {
        self.inner_tiles.background()
    }

    fn layer_count(&self) -> usize {
        self.inner_tiles.layer_count()
    }

    fn layer_opacity(&self, layer: usize) -> Option<f32> {
        self.inner_tiles.layer_opacity(layer)
    }

    fn tile_size(&self) -> U32Size2 {
        self.inner_tiles.tile_size()
    }

    fn map_size(&self) -> U32Size2 {
        self.inner_tiles.map_size()
    }

    fn scale(&self) -> f32 {
        self.inner_tiles.scale()
    }

    fn gap(&self) -> U32Size2 {
        self.inner_tiles.gap()
    }

    fn tile(&self, layer_index: usize, pos: U32Pos2) -> Option<Tile> {
        let ipos: I32Pos2 = pos.into();
        let position = ipos - self.offset;

        // Outer `Some` is for whether we are in bounds of Vec, and inner `Some` is whether
        // this is a visible layer
        if let Some(Some(visible_layer_index)) = self.layer_to_stamp_layer.get(layer_index) {
            if let Some(tile) = self.stamp.tiles.get(&TileLocation {
                stamp_layer_index: *visible_layer_index,
                position,
            }) {
                return Some(*tile);
            }
        }
        self.inner_tiles.tile(layer_index, pos)
    }

    fn set_tile(&mut self, _layer: usize, _pos: U32Pos2, _tile: Option<Tile>) -> bool {
        false
    }
}
