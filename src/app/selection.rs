use crate::{
    app::App,
    data::{action::Action, mode::Mode},
};

impl App {
    pub fn select_layer(&mut self, layer_index: usize) {
        if let Some(map_editing) = self.selected_map_editing_mut() {
            map_editing
                .edit
                .layer_selection
                .select_only_index(layer_index, map_editing.map.tiles());
        }
    }

    pub fn cut(&mut self) {
        self.copy_optional_delete(true);
    }

    pub fn copy(&mut self) {
        self.copy_optional_delete(false);
    }

    pub fn delete_and_clear_selection(&mut self) {
        if let Some(map_id) = self.edit.selected_map_id {
            self.act(Action::EraseSelectedVisibleTiles { map_id });
            self.clear_selection();
        }
    }

    pub fn clear_selection(&mut self) {
        if let Some(map_edit_state) = self.selected_map_edit_state_mut() {
            map_edit_state.selection_mut().clear();
        }
    }

    pub fn copy_optional_delete(&mut self, delete: bool) {
        if self.edit.mode == Mode::Select {
            if let Some(new_stamp) = self
                .selected_map_editing_mut()
                .map(|me| me.selection_as_stamp())
            {
                self.edit.stamp = new_stamp;

                if !self.edit.stamp.is_empty() {
                    if delete {
                        if let Some(map) = self.selected_map() {
                            self.act(Action::EraseSelectedVisibleTiles { map_id: map.id() });
                        }
                    }
                    self.draw_mode();
                    self.tileset_selected_tiles_from_stamp();
                }
            }
        }
    }

    pub fn tileset_selected_tiles_from_stamp(&mut self) {
        // Make sure we have selections for all tilesets, and clear them
        self.apply_default_tileset_tile_selection_by_id();
        self.edit.clear_tileset_tile_selections();

        // For each tileset, add the tiles used in the stamp to the associated selection
        for tileset in self.state.resources.tilesets().iter() {
            let id = tileset.id();
            let tile_set_size = tileset.size_in_tiles;
            if let Some(selection) = self.edit.tileset_tile_selection_by_id.get_mut(&id) {
                for source in self.edit.stamp.tile_sources.iter() {
                    if source.tileset_id == id {
                        let tile_pos =
                            tile_set_size.pos_from_linear_index(source.tile_index.index());
                        selection.add_selection(tile_pos);
                    }
                }
            }
        }
    }
}
