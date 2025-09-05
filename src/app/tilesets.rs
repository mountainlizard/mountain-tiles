use egui::Context;

use crate::{
    app::App,
    data::tilesets::{Tileset, TilesetId},
    data::{
        action::Action,
        modal::{DataLossOperation, ModalState, TilesetOperation},
    },
    geom::i32pos2::I32Pos2,
    geom::u32size2::u32size2,
    selection::{apply_default_selection, apply_default_value_per_selectable_id},
};

impl App {
    pub(super) fn add_tileset(&mut self, tileset: Tileset) {
        let id = self.state.resources.tilesets_mut().push_tileset(tileset);
        self.edit.selected_tileset_id = Some(id);
    }

    pub(super) fn delete_tileset_by_id(&mut self, tileset_id: TilesetId) -> bool {
        let mut change = false;
        change |= self.state.clear_tiles_with_tileset(tileset_id);
        change |= self.state.resources.tilesets_mut().delete_by_id(tileset_id);
        if change {
            self.edit.clear_tileset_tile_selections_and_stamp();
        }
        change
    }

    pub(super) fn update_tileset(&mut self, id: TilesetId, tileset: Tileset) -> bool {
        if self
            .state
            .resources
            .tilesets_mut()
            .update_tileset(id, tileset)
        {
            self.edit.clear_tileset_tile_selections_and_stamp();
            true
        } else {
            false
        }
    }

    pub fn selected_tileset(&self) -> Option<&Tileset> {
        self.edit
            .selected_tileset_id
            .and_then(|id| self.state.resources.tilesets.get_by_id(id))
    }

    /// Call [`apply_default_selection`] on our tileset selection.
    pub(super) fn apply_default_tileset_selection(&mut self) {
        apply_default_selection(
            &mut self.edit.selected_tileset_id,
            self.state.resources.tilesets(),
        );
    }

    /// Call [`apply_default_selection_per_selectable_id`] on our tilesets
    pub(super) fn apply_default_tileset_tile_selection_by_id(&mut self) {
        apply_default_value_per_selectable_id(
            &mut self.edit.tileset_tile_selection_by_id,
            self.state.resources.tilesets(),
        );
    }

    pub fn refresh_selected_tileset(&self, ctx: &Context) {
        if let Some(tileset) = self.selected_tileset() {
            self.textures.refresh_tileset(ctx, tileset);
        }
    }

    pub fn show_new_tileset_modal(&mut self) {
        let mut tileset = Tileset::default();
        // TODO: Use selected map's tile size as default, if there is one
        tileset.tile_size = u32size2(8, 8);

        self.edit
            .show_modal(ModalState::tileset(tileset, TilesetOperation::NewTileset));
    }

    pub fn show_edit_tileset_modal(&mut self) {
        if let Some(tileset) = self.selected_tileset() {
            self.edit.show_modal(ModalState::tileset(
                tileset.clone(),
                TilesetOperation::UpdateExistingTileset(tileset.id()),
            ));
        }
    }

    pub fn delete_selected_tileset(&mut self) {
        if let Some(tileset) = self.selected_tileset() {
            if let Some(tileset_use) = self.state.find_use_of_tileset(tileset.id()) {
                self.show_data_loss_modal(DataLossOperation::DeleteTileset {
                    tileset_id: tileset.id(),
                    tileset_name: tileset.path.to_string(),
                    tileset_use,
                });
            } else {
                self.act(Action::DeleteTileset { id: tileset.id() });
            }
        }
    }

    pub fn shift_tileset_selection(&mut self, shift: I32Pos2) {
        self.edit
            .stamp
            .shift(self.state.resources.tilesets(), shift);
        self.tileset_selected_tiles_from_stamp();
    }

    pub fn previous_tileset(&mut self) {
        if let Some(id) = self.edit.selected_tileset_id {
            if let Some(previous_tileset) = self.state.resources.tilesets().previous_by_id(id) {
                self.edit.selected_tileset_id = Some(previous_tileset.id());
            }
        }
    }

    pub fn next_tileset(&mut self) {
        if let Some(id) = self.edit.selected_tileset_id {
            if let Some(next_tileset) = self.state.resources.tilesets().next_by_id(id) {
                self.edit.selected_tileset_id = Some(next_tileset.id());
            }
        }
    }
}
