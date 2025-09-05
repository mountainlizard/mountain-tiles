use egui::Rect;

use crate::{
    data::{
        maps::Map,
        tiles::{layer_tiles::LayerId, tile_selection::TileSelection},
    },
    geom::i32pos2::I32Pos2,
    geom::u32pos2::U32Pos2,
    selection::Selection,
};

/// State associated with editing a [`crate::data::maps::Map`]
#[derive(Debug, Clone, PartialEq)]
pub struct MapEditState {
    pub layer_selection: Selection<LayerId>,
    pub map_hovered: Option<U32Pos2>,
    pub map_scene_hovered: Option<I32Pos2>,
    selection: TileSelection,
    /// Tracks the position where the map is displayed in a [`egui::Scene`]
    pub scene_rect: Rect,
}

impl Default for MapEditState {
    fn default() -> Self {
        Self {
            layer_selection: Selection::new(),
            map_hovered: None,
            map_scene_hovered: None,
            selection: Default::default(),
            scene_rect: Rect::ZERO,
        }
    }
}

impl MapEditState {
    pub fn selection(&self) -> &TileSelection {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut TileSelection {
        &mut self.selection
    }

    pub fn no_layers_selected(&self, map: &Map) -> bool {
        !map.tiles()
            .layers()
            .any(|layer| self.layer_selection.is_selected(layer.id()))
    }

    /// Apply invariants for this edit state, against specified [`Map`]
    pub(crate) fn apply_invariants(&mut self, map: &Map) {
        // If layer selection is empty, select the first (topmost) layer
        if self.no_layers_selected(map) {
            if let Some(layer) = map.tiles().first_layer() {
                self.layer_selection.select_only(layer.id());
            }
        }
    }
}
