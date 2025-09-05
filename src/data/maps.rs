use std::fmt::Display;
use std::slice::{Iter, IterMut};

use egui::ahash::HashSet;

use crate::data::tiles::layer_tiles::LayerTiles;
use crate::data::tiles::tile_color::UserColor;
use crate::geom::u32size2::U32Size2;
use crate::selection::{Selectable, SelectableDefault};

/// This is unique within a [`crate::data::state::State`], and persists for a
/// given [`Map`] even if the [`Map`] is edited. This can be used
/// to track a [`Map`] externally, e.g. to track which [`Map`] is selected
/// in a [`crate::data::state::State`].
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct MapId(u32);

impl MapId {
    pub const ONE: MapId = MapId(1);

    pub fn next(&self) -> MapId {
        MapId(self.0 + 1)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for MapId {
    fn default() -> Self {
        Self::ONE
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Default, PartialEq)]
pub struct Map {
    id: MapId,
    pub name: String,
    pub tiles: LayerTiles,
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Map {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn id(&self) -> MapId {
        self.id
    }

    pub(crate) fn tiles(&self) -> &LayerTiles {
        &self.tiles
    }

    pub(crate) fn tiles_mut(&mut self) -> &mut LayerTiles {
        &mut self.tiles
    }

    pub fn new(
        name: String,
        map_size: U32Size2,
        tile_size: U32Size2,
        background: UserColor,
    ) -> Self {
        let tiles = LayerTiles::new(background, tile_size, map_size);
        Self {
            id: Default::default(),
            name,
            tiles,
        }
    }

    pub(crate) fn new_with_layer(
        name: String,
        map_size: U32Size2,
        tile_size: U32Size2,
        background: UserColor,
    ) -> Self {
        let mut tiles = LayerTiles::new(background, tile_size, map_size);
        tiles.insert_layer(0, "New Layer", true, None);
        Self {
            id: Default::default(),
            name,
            tiles,
        }
    }
}

#[derive(Clone, Default, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Maps {
    maps: Vec<Map>,
    next_map_id: MapId,
}

impl Maps {
    pub fn iter(&self) -> Iter<'_, Map> {
        self.maps.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Map> {
        self.maps.iter_mut()
    }

    pub fn push_map(&mut self, mut map: Map) -> MapId {
        let id = self.next_map_id;
        self.next_map_id = id.next();
        map.id = id;
        self.maps.push(map);
        id
    }

    pub fn get_by_id(&self, id: MapId) -> Option<&Map> {
        self.maps.iter().find(|t| t.id == id)
    }

    pub fn get_by_id_mut(&mut self, id: MapId) -> Option<&mut Map> {
        self.maps.iter_mut().find(|t| t.id == id)
    }

    pub fn delete_by_id(&mut self, id: MapId) -> bool {
        let start_len = self.maps.len();
        self.maps.retain(|map| map.id != id);
        self.maps.len() != start_len
    }
}

impl Selectable<MapId> for Maps {
    fn all_ids(&self) -> HashSet<MapId> {
        self.maps.iter().map(|t| t.id()).collect()
    }
    fn contains_id(&self, id: &MapId) -> bool {
        self.maps.iter().any(|l| l.id() == *id)
    }
}

impl SelectableDefault<MapId> for Maps {
    fn default_id(&self) -> Option<MapId> {
        self.maps.first().map(|t| t.id())
    }
}
