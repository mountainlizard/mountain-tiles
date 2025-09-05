use egui::ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

use crate::{
    data::tiles::{
        tile_color::TileColor, tile_selection::TileSelection, Tile, TileIndex, TileSource, Tiles,
    },
    data::tilesets::Tilesets,
    geom::i32pos2::I32Pos2,
    geom::transform::Transform,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileLocation {
    /// The stamp layer doesn't necessarily map directly to the
    /// layer in any [`Tiles`] that are drawn into.
    /// For example, the `stamp_layer_index` may be taken as
    /// 0 for the first visible layer, 1 for the second, and so on.
    /// This allows for copying from one set of visible layers, then
    /// changing which layers are visible to draw into those, allowing
    /// us to move tiles between layers.
    pub stamp_layer_index: usize,
    pub position: I32Pos2,
}

impl TileLocation {
    pub fn new(stamp_layer_index: usize, position: I32Pos2) -> Self {
        Self {
            stamp_layer_index,
            position,
        }
    }
    pub fn with_transform(&self, transform: Transform) -> TileLocation {
        TileLocation {
            stamp_layer_index: self.stamp_layer_index,
            position: self.position.with_transform(transform),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stamp {
    pub tiles: HashMap<TileLocation, Tile>,
    pub tile_sources: HashSet<TileSource>,
    /// The number of layers the stamp contains.
    /// Note that not all layers may contain any actual tiles
    layer_count: usize,
    /// Track the transformation applied relative to the original
    /// arrangement when this stamp was created
    /// Note that this transformation is already baked into the tiles
    /// in the stamp - the transform is just used to display to user
    /// and allow for clearing applied transform, etc.
    pub transform: Transform,
}

impl Stamp {
    pub fn new() -> Self {
        Stamp {
            tiles: HashMap::new(),
            tile_sources: HashSet::new(),
            layer_count: 0,
            transform: Transform::None,
        }
    }

    pub fn insert(&mut self, tile_location: TileLocation, tile: Tile) {
        if tile_location.stamp_layer_index >= self.layer_count {
            self.layer_count = tile_location.stamp_layer_index + 1;
        }
        self.tiles.insert(tile_location, tile);
        self.tile_sources.insert(tile.source);
    }

    pub fn from_selected_tiles<T: Tiles>(
        tiles: &T,
        selection: &TileSelection,
        override_color: Option<TileColor>,
    ) -> Self {
        let mut stamp = Stamp::new();

        // If any cells are selected, we have a selection rect, otherwise we can skip to returning
        // empty result
        if let Some(selection_rect) = selection.range_rect() {
            let center: I32Pos2 = selection_rect.center().into();
            for layer_index in 0..tiles.layer_count() {
                for position in selection.iter() {
                    if let Some(tile) = tiles.tile(layer_index, *position) {
                        let ipos: I32Pos2 = position.into();
                        stamp.insert(
                            TileLocation {
                                stamp_layer_index: layer_index,
                                position: ipos - center,
                            },
                            tile.with_override_color(override_color),
                        );
                    }
                }
            }
        }
        stamp
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    pub fn with_transform(&self, transform: Transform) -> Stamp {
        let mut stamp = Stamp::new();

        for (location, tile) in self.tiles.iter() {
            let new_location = location.with_transform(transform);
            let new_tile = tile.with_transform(transform);
            stamp.insert(new_location, new_tile);
        }
        stamp.transform = self.transform.and_then(transform);
        stamp
    }

    pub fn with_color(&self, color: TileColor) -> Stamp {
        let mut stamp = Stamp::new();
        for (location, tile) in self.tiles.iter() {
            let new_tile = tile.with_color(color);
            stamp.insert(location.clone(), new_tile);
        }
        stamp
    }

    pub fn layer_count(&self) -> usize {
        self.layer_count
    }

    /// For each tile in the stamp, move the tileset tile we are using by the specified shift.
    pub fn shift(&mut self, tilesets: &Tilesets, shift: I32Pos2) {
        for tile in self.tiles.values_mut() {
            if let Some(tileset) = tilesets.get_by_id(tile.source.tileset_id) {
                let tileset_pos = tileset
                    .size_in_tiles
                    .pos_from_linear_index(tile.source.tile_index.index());
                let new_pos = tileset
                    .size_in_tiles
                    .u32pos_shifted_and_wrapped(tileset_pos, shift);
                if let Some(new_index) = new_pos.linear_index(tileset.size_in_tiles) {
                    tile.source.tile_index = TileIndex::new(new_index);
                }
            }
        }
        self.tile_sources.clear();
        for tile in self.tiles.values() {
            self.tile_sources.insert(tile.source);
        }
    }
}
