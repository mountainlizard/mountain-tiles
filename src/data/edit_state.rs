use std::mem;

use egui::ahash::HashMap;

use crate::{
    data::palette::PaletteIndex,
    data::stamp::Stamp,
    data::tiles::tile_color::TileColor,
    data::tiles::tile_selection::TileSelection,
    data::tilesets::TilesetId,
    data::{map_edit_state::MapEditState, maps::MapId, modal::ModalState, mode::Mode},
};

#[derive(Default, Clone, PartialEq)]
pub struct EditState {
    /// The [`TilesetId`] of the selected [`crate::tilesets::Tileset`],
    /// if any is selected.
    /// Note that this is not guaranteed to always correspond to an available
    /// [`crate::tilesets::Tileset`], and if it does not, this should be
    /// treated identically to a value of [`None`] (no selection).
    /// It's recommended that on each frame, this value is checked and if
    /// the id is not valid, this field is set to [`None`], or
    /// [`Some`] containing a valid [`TilesetId`].
    pub selected_tileset_id: Option<TilesetId>,

    /// A map from each [`crate::tilesets::Tileset`]s [`TilesetId`]
    /// to its current [`TileSelection`].
    /// This may contain [`TilesetId`]s that do not correspond to
    /// an available [`crate::tilesets::Tileset`], and these should
    /// be ignored.
    /// This may not contain a mapping for each available [`TilesetId`],
    /// and if a mapping is missing for a given [`TilesetId`] this should
    /// be treated as that [`crate::tilesets::Tileset`] having an empty
    /// [`TileSelection`].
    /// It's recommended that on each frame, this value is
    /// checked and any mappings from invalid [`TilesetId`]s are removed,
    /// and empty [`TileSelection`]s added for any missing valid
    /// [`TilesetId`]s.
    pub tileset_tile_selection_by_id: HashMap<TilesetId, TileSelection>,

    /// The [`MapId`] of the selected [`crate::data::map::Map`],
    /// if any is selected.
    /// Note that this is not guaranteed to always correspond to an available
    /// [`crate::data::map::Map`], and if it does not, this should be
    /// treated identically to a value of [`None`] (no selection).
    /// It's recommended that on each frame, this value is checked and if
    /// the id is not valid, this field is set to [`None`], or
    /// [`Some`] containing a valid [`MapId`].
    pub selected_map_id: Option<MapId>,

    /// A map from each [`crate::data::map::Map`]s [`MapId`]
    /// to its current [`MapEditState`].
    /// This may contain [`MapId`]s that do not correspond to
    /// an available [`crate::data::maps::Map`], and these should
    /// be ignored.
    /// This may not contain a mapping for each available [`MapId`],
    /// and if a mapping is missing for a given [`MapId`] this should
    /// be treated as that [`crate::data::maps::Map`] having a default
    /// [`MapEditState`].
    /// It's recommended that on each frame, this value is
    /// checked and any mappings from invalid [`MapId`]s are removed,
    /// and empty [`MapEditState`]s added for any missing valid
    /// [`MapId`]s.
    pub map_edit_state_by_id: HashMap<MapId, MapEditState>,

    /// The currently selected palette index.
    /// Note that this is not guaranteed to be within the index range
    /// of the current palette, and if it is not it should be treated as
    /// selection of the last valid palette index.
    /// It's recommended that on each frame, this value is
    /// checked and if the index is outside the palette range, it's
    /// set to the last valid palette index.
    selected_palette_index: PaletteIndex,

    /// The current modal state, when this is anything other than
    /// [`ModalState::None`] the UI will be displaying the relevant
    /// modal
    pub modal: ModalState,

    /// The stamp we will apply if a draw action occurs - this can be empty
    pub stamp: Stamp,

    /// The current editing [`Mode`]
    pub mode: Mode,
}

impl EditState {
    pub fn selected_tileset_tile_selection(&self) -> Option<&TileSelection> {
        self.selected_tileset_id
            .and_then(|id| self.tileset_tile_selection_by_id.get(&id))
    }

    pub fn selected_tileset_tile_selection_mut(&mut self) -> Option<&mut TileSelection> {
        self.selected_tileset_id
            .as_mut()
            .and_then(|id| self.tileset_tile_selection_by_id.get_mut(id))
    }

    pub fn selected_palette_index(&self) -> PaletteIndex {
        self.selected_palette_index
    }

    pub fn select_palette_index(&mut self, i: PaletteIndex) {
        self.selected_palette_index = i;
        self.stamp = self.stamp.with_color(TileColor::from_palette_index(i));
    }

    pub fn show_modal(&mut self, modal_state: ModalState) {
        self.modal = modal_state;
    }

    pub fn hide_modal(&mut self) -> ModalState {
        mem::take(&mut self.modal)
    }

    pub fn clear_tileset_tile_selections_and_stamp(&mut self) {
        self.clear_tileset_tile_selections();
        self.stamp = Stamp::new();
    }

    pub fn clear_tileset_tile_selections(&mut self) {
        for selection in self.tileset_tile_selection_by_id.values_mut() {
            selection.clear();
        }
    }

    /// Merge the relevant parts of the [`EditState`] from an undo/redo state with this
    /// [`EditState`]. The relevant parts are those needed to show the correct "context"
    /// for the change - i.e. make sure that the location of the change is visible in the UI.
    /// Currently, this is the selected map id (so we can see the map that's being changed),
    /// and the scene rect for the selected map (so we can see the area that's being changed)
    pub(crate) fn merge_undo_redo(&mut self, undo_redo_edit: EditState) {
        self.selected_map_id = undo_redo_edit.selected_map_id;

        // If the undo/redo state has a selected map, and an edit state for that map
        if let Some(map_id) = undo_redo_edit.selected_map_id {
            if let Some(undo_redo_map_edit_state) = undo_redo_edit.map_edit_state_by_id.get(&map_id)
            {
                // If we have an edit state for the map, copy over the undo/redo scene rect,
                // otherwise just grab a copy of the whole undo/redo edit state
                if let Some(map_edit_state) = self.map_edit_state_by_id.get_mut(&map_id) {
                    map_edit_state.scene_rect = undo_redo_map_edit_state.scene_rect;
                } else {
                    self.map_edit_state_by_id
                        .insert(map_id, undo_redo_map_edit_state.clone());
                }
            }
        }
    }
}
