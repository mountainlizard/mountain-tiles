use std::slice::Iter;

use egui::ahash::{HashSet, HashSetExt};

use crate::{
    data::palette::Palette,
    data::tiles::{tile_color::UserColor, Tile, Tiles},
    data::tilesets::TilesetId,
    geom::u32pos2::{u32pos2, U32Pos2},
    geom::u32size2::{u32size2, U32Size2},
    selection::{
        Selectable, SelectableList, SelectableListIter, SelectableListIterDeletable, Selection,
    },
};

/// This is unique within a [`LayerTiles`], and persists for a
/// given [`Layer`] even if edited (e.g. resized). This can be used
/// to track a layer externally, e.g. to store in a
/// [`std::collections::HashSet`]` to represent selected layers.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LayerId(u32);

impl LayerId {
    pub const ONE: LayerId = LayerId(1);

    fn next(&self) -> LayerId {
        LayerId(self.0 + 1)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct Layer {
    id: LayerId,
    name: String,
    visible: bool,
    size: U32Size2,
    tiles: Vec<Option<Tile>>,
    opacity: Option<f32>,
}

impl Layer {
    fn new(id: LayerId, name: &str, visible: bool, size: U32Size2, opacity: Option<f32>) -> Self {
        let tiles = vec![None; size.area() as usize];
        Self {
            id,
            name: name.to_string(),
            visible,
            size,
            tiles,
            opacity,
        }
    }

    pub fn id(&self) -> LayerId {
        self.id
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn opacity(&self) -> Option<f32> {
        self.opacity
    }

    fn tile(&self, pos: U32Pos2) -> Option<Tile> {
        if self.visible {
            pos.linear_index(self.size)
                .and_then(|i| self.tiles.get(i as usize).copied())?
        } else {
            None
        }
    }

    fn set_tile(&mut self, pos: U32Pos2, tile: Option<Tile>) {
        if self.visible {
            if let Some(i) = pos.linear_index(self.size) {
                if let Some(target_tile) = self.tiles.get_mut(i as usize) {
                    *target_tile = tile;
                }
            }
        }
    }

    fn with_new_size(&self, new_size: U32Size2) -> Layer {
        let mut new_layer = Layer::new(self.id, &self.name, self.visible, new_size, self.opacity);
        for y in 0..new_size.h {
            for x in 0..new_size.w {
                let pos = u32pos2(x, y);
                new_layer.set_tile(pos, self.tile(pos));
            }
        }
        new_layer
    }

    pub fn tiles_iter(&self) -> Iter<'_, Option<Tile>> {
        self.tiles.iter()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    fn clear_tiles_with_tileset(&mut self, tileset_id: TilesetId) -> bool {
        let mut change = false;
        for tile_option in self.tiles.iter_mut() {
            if let Some(tile) = tile_option {
                if tile.source.tileset_id == tileset_id {
                    *tile_option = None;
                    change = true;
                }
            }
        }
        change
    }

    pub fn clear_tiles_outside_palette(&mut self, palette: &Palette) {
        for tile_option in self.tiles.iter_mut() {
            if let Some(tile) = tile_option {
                if !palette.is_tilecolor_available(&tile.color) {
                    *tile_option = None;
                }
            }
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct LayerTiles {
    pub background: UserColor,
    tile_set_count: usize,
    layers: Vec<Layer>,
    tile_size: U32Size2,
    map_size: U32Size2,
    next_layer_id: LayerId,
}

impl Tiles for LayerTiles {
    fn background(&self) -> UserColor {
        self.background
    }

    fn layer_count(&self) -> usize {
        self.layers.len()
    }

    fn layer_opacity(&self, layer: usize) -> Option<f32> {
        self.layers.get(layer).and_then(|l| l.opacity)
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
        self.layers.get(layer).and_then(|layer| layer.tile(pos))
    }

    fn set_tile(&mut self, layer: usize, pos: U32Pos2, tile: Option<Tile>) -> bool {
        if let Some(layer) = self.layers.get_mut(layer) {
            if layer.tile(pos) != tile {
                layer.set_tile(pos, tile);
                return true;
            }
        }
        false
    }
}

impl Default for LayerTiles {
    fn default() -> Self {
        let map_size = u32size2(8, 8);
        let mut tiles = LayerTiles::new(UserColor::BLACK, u32size2(8, 8), map_size);
        tiles.insert_layer(0, "Layer 0", true, None);
        tiles
    }
}

impl LayerTiles {
    pub fn new(background: UserColor, tile_size: U32Size2, map_size: U32Size2) -> Self {
        Self {
            background,
            tile_set_count: 1,
            layers: vec![],
            tile_size,
            map_size,
            next_layer_id: LayerId::ONE,
        }
    }

    /// Build a list of visible layer indices, this contains an entry
    /// per layer, which is None for invisible layers, and Some(i) for the ith
    /// visible layer.
    pub fn layer_index_to_visible_layer_index(&self) -> Vec<Option<usize>> {
        let mut indices = Vec::with_capacity(self.layers.len());
        let mut visible_layer_index = 0usize;
        for layer in self.layers.iter() {
            if layer.visible {
                indices.push(Some(visible_layer_index));
                visible_layer_index += 1;
            } else {
                indices.push(None);
            }
        }
        indices
    }

    /// Build a list of layer indices, one for each visible layer
    pub fn visible_layer_index_to_layer_index(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        for (layer_index, layer) in self.layers.iter().enumerate() {
            if layer.visible {
                indices.push(layer_index);
            }
        }
        indices
    }

    /// Resize this [`LayerTiles`] to `new_size`, recreating each existing layer
    /// using [`Layer::with_new_size`]. This will preserve any existing contents
    /// that lie within the new size, and pad any additional size with empty tiles,
    /// i.e. [`None`].
    pub fn resize(&mut self, new_size: U32Size2) {
        let mut new_layers = Vec::with_capacity(self.layers.len());
        for layer in self.layers.iter() {
            new_layers.push(layer.with_new_size(new_size));
        }
        self.layers = new_layers;
        self.map_size = new_size;
    }

    pub fn set_tile_size(&mut self, new_size: U32Size2) {
        self.tile_size = new_size;
    }

    pub fn insert_layer(
        &mut self,
        i: usize,
        name: &str,
        visible: bool,
        opacity: Option<f32>,
    ) -> LayerId {
        let id = self.next_layer_id;
        self.next_layer_id = id.next();
        self.layers
            .insert(i, Layer::new(id, name, visible, self.map_size, opacity));
        id
    }

    pub fn remove_layer(&mut self, i: usize) {
        self.layers.remove(i);
    }

    pub fn move_layer_higher(&mut self, i: usize) {
        if i > 0 {
            let to_move = self.layers.remove(i);
            self.layers.insert(i - 1, to_move);
        }
    }

    pub fn move_layer_lower(&mut self, i: usize) {
        if i < self.layers.len() - 1 {
            let to_move = self.layers.remove(i);
            self.layers.insert(i + 1, to_move);
        }
    }

    pub fn layer_name_mut(&mut self, i: usize) -> Option<&mut String> {
        self.layers.get_mut(i).map(|layer| &mut layer.name)
    }

    pub fn layer_name(&self, i: usize) -> Option<&String> {
        self.layers.get(i).map(|layer| &layer.name)
    }

    pub fn layer_id(&self, i: usize) -> Option<LayerId> {
        self.layers.get(i).map(|layer| layer.id())
    }

    pub fn layer_visible_mut(&mut self, i: usize) -> Option<&mut bool> {
        self.layers.get_mut(i).map(|layer| &mut layer.visible)
    }

    pub fn layer_visible(&self, i: usize) -> Option<bool> {
        self.layers.get(i).map(|layer| layer.visible)
    }

    pub fn rename_layer(&mut self, layer_index: usize, name: &str) {
        if let Some(layer) = self.layers.get_mut(layer_index) {
            layer.name = name.to_string();
        }
    }

    pub fn layer_index_for_id(&self, id: LayerId) -> Option<usize> {
        self.layers.iter().position(|layer| layer.id == id)
    }

    pub fn next_layer_id(&self) -> LayerId {
        self.next_layer_id
    }

    /// Return the [`LayerId`]s for all layers between layer "a" and "b",
    /// where a and b are located using the specified [`LayerId`]s.
    /// Layer a and b can be in either order, note that the layer ids between
    /// will always be returned sorted by position in our layer list, by increasing index.
    /// The returned vec may be empty if one or both of the [`LayerId`]s don't match
    /// any layer.
    pub fn layer_id_range(&self, a: LayerId, b: LayerId) -> Vec<LayerId> {
        let mut layer_ids = vec![];
        if let (Some(index_a), Some(index_b)) =
            (self.layer_index_for_id(a), self.layer_index_for_id(b))
        {
            for i in index_a.min(index_b)..=index_a.max(index_b) {
                if let Some(id) = self.layer_id(i) {
                    layer_ids.push(id);
                }
            }
        }
        layer_ids
    }

    pub fn all_layer_ids(&self) -> HashSet<LayerId> {
        let mut layer_ids = HashSet::new();
        for layer in self.layers.iter() {
            layer_ids.insert(layer.id);
        }
        layer_ids
    }

    pub fn layers(&self) -> core::slice::Iter<'_, Layer> {
        self.layers.iter()
    }

    pub fn last_layer(&self) -> Option<&Layer> {
        self.layers.last()
    }

    pub fn first_layer(&self) -> Option<&Layer> {
        self.layers.first()
    }

    pub fn remove_layer_by_index(&mut self, index: usize) {
        self.layers.remove(index);
    }

    pub fn remove_layers_by_selection(&mut self, selection: &Selection<LayerId>) -> bool {
        let start_len = self.layers.len();
        self.layers.retain(|layer| !selection.is_selected(layer.id));
        self.layers.len() != start_len
    }

    pub fn clear_tiles_with_tileset(&mut self, tileset_id: TilesetId) -> bool {
        let mut change = false;
        for layer in self.layers.iter_mut() {
            change |= layer.clear_tiles_with_tileset(tileset_id);
        }
        change
    }

    pub fn clear_tiles_outside_palette(&mut self, palette: &Palette) {
        for layer in self.layers.iter_mut() {
            layer.clear_tiles_outside_palette(palette);
        }
    }

    pub fn set_layer_visible(&mut self, id: LayerId, visible: bool) -> bool {
        let mut change = false;
        for layer in self.layers.iter_mut() {
            if layer.id() == id && layer.visible != visible {
                layer.visible = visible;
                change = true;
            }
        }
        change
    }

    pub fn set_layer_name(&mut self, id: LayerId, name: String) -> bool {
        let mut change = false;
        for layer in self.layers.iter_mut() {
            if layer.id() == id && layer.name != name {
                layer.name = name.clone();
                change = true;
            }
        }
        change
    }

    pub fn set_layer_opacity(&mut self, id: LayerId, opacity: Option<f32>) -> bool {
        let mut change = false;
        for layer in self.layers.iter_mut() {
            if layer.id() == id && layer.opacity != opacity {
                layer.opacity = opacity;
                change = true;
            }
        }
        change
    }

    pub fn layer_name_by_id(&self, id: LayerId) -> Option<String> {
        for layer in self.layers.iter() {
            if layer.id() == id {
                return Some(layer.name.clone());
            }
        }
        None
    }

    pub fn layer_opacity_by_id(&self, id: LayerId) -> Option<f32> {
        for layer in self.layers.iter() {
            if layer.id() == id {
                return layer.opacity;
            }
        }
        None
    }
}

impl Selectable<LayerId> for LayerTiles {
    fn all_ids(&self) -> HashSet<LayerId> {
        self.all_layer_ids()
    }

    fn contains_id(&self, id: &LayerId) -> bool {
        self.layers().any(|l| l.id == *id)
    }
}

impl SelectableList<LayerId> for LayerTiles {
    fn id_range(&self, a: LayerId, b: LayerId) -> Vec<LayerId> {
        self.layer_id_range(a, b)
    }
}

impl SelectableListIter<LayerId> for LayerTiles {
    fn get_id(&self, i: usize) -> Option<LayerId> {
        self.layer_id(i)
    }

    fn id_iter(&self) -> impl Iterator<Item = LayerId> {
        self.layers.iter().map(|l| l.id())
    }

    fn id_len(&self) -> usize {
        self.layers.len()
    }
}

impl SelectableListIterDeletable<LayerId> for LayerTiles {
    fn delete_by_selection(&mut self, selection: &Selection<LayerId>) -> bool {
        self.remove_layers_by_selection(selection)
    }
}
