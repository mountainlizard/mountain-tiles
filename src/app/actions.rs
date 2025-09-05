use crate::{
    app::App,
    data::{
        action::{Action, ActionResult},
        mode::Mode,
    },
};

impl App {
    pub fn act(&mut self, action: Action) {
        let result = match action {
            Action::Draw {
                map_id,
                pos,
                complete,
            } => {
                let change = self.draw(map_id, pos);
                ActionResult::new(change, complete)
            }
            Action::Erase {
                map_id,
                pos,
                complete,
            } => {
                let change = self.erase(map_id, pos);
                ActionResult::new(change, complete)
            }
            Action::EraseSelectedVisibleTiles { map_id } => {
                if let Some(mut me) = self.map_editing_mut(map_id) {
                    // TODO: Remove this mode check or move it to somewhere more suitable, like
                    // shortcuts?
                    if me.mode == Mode::Select {
                        let change = me.erase_selected_visible_tiles();
                        ActionResult::new(change, true)
                    } else {
                        ActionResult::NONE
                    }
                } else {
                    ActionResult::NONE
                }
            }
            Action::UpdateTileset { id, tileset } => {
                let change = self.update_tileset(id, tileset);
                ActionResult::new(change, true)
            }
            Action::AddTileset { tileset } => {
                // Always succeeds
                self.add_tileset(tileset);
                ActionResult::CHANGE_AND_REVISION
            }
            Action::DeleteTileset { id } => {
                let change = self.delete_tileset_by_id(id);
                ActionResult::new(change, true)
            }
            Action::ReplacePalette { palette } => {
                let change = self.replace_palette(palette);
                ActionResult::new(change, true)
            }
            Action::AddMap { map } => {
                // Always succeeds
                self.add_map(map);
                ActionResult::CHANGE_AND_REVISION
            }
            Action::AppendTiledMap {
                tiled,
                squash_layers,
                prefer_relative_path,
            } => {
                // We'll append to a clone, since if the process fails it may partially modify the state
                let mut new_state = self.state.clone();
                match tiled.append_to_state(&mut new_state, squash_layers, prefer_relative_path) {
                    Ok(map_id) => {
                        self.state = new_state;
                        self.edit.selected_map_id = Some(map_id);
                        ActionResult::CHANGE_AND_REVISION
                    }
                    Err(e) => {
                        self.show_error_modal(&e.to_string());
                        ActionResult::NONE
                    }
                }
            }
            Action::UpdateMap {
                map_id,
                name,
                map_size,
                tile_size,
                background_color,
            } => {
                let change = self.update_map(map_id, name, map_size, tile_size, background_color);
                ActionResult::new(change, true)
            }
            Action::DeleteMap { id } => {
                let change = self.delete_map(id);
                ActionResult::new(change, true)
            }
            Action::SetLayerVisible {
                map_id,
                layer_id,
                visible,
            } => {
                if let Some(map) = self.state.maps.get_by_id_mut(map_id) {
                    let change = map.tiles.set_layer_visible(layer_id, visible);
                    ActionResult::new(change, true)
                } else {
                    ActionResult::NONE
                }
            }
            Action::SetLayerName {
                map_id,
                layer_id,
                name,
            } => {
                if let Some(map) = self.state.maps.get_by_id_mut(map_id) {
                    let change = map.tiles.set_layer_name(layer_id, name);
                    ActionResult::new(change, true)
                } else {
                    ActionResult::NONE
                }
            }
            Action::EditLayer {
                map_id,
                layer_id,
                name,
                opacity,
            } => {
                if let Some(map) = self.state.maps.get_by_id_mut(map_id) {
                    let mut change = map.tiles.set_layer_name(layer_id, name);
                    change |= map.tiles.set_layer_opacity(layer_id, opacity);
                    ActionResult::new(change, true)
                } else {
                    ActionResult::NONE
                }
            }

            Action::AddLayer { map_id } => {
                if let Some(mut me) = self.map_editing_mut(map_id) {
                    // Always succeeds
                    me.add_layer();
                    ActionResult::CHANGE_AND_REVISION
                } else {
                    ActionResult::NONE
                }
            }
            Action::DeleteSelectedLayers { map_id } => {
                if let Some(mut me) = self.map_editing_mut(map_id) {
                    let change = me.delete_selected_layers();
                    ActionResult::new(change, true)
                } else {
                    ActionResult::NONE
                }
            }

            Action::MergeSelectedLayers { map_id } => {
                if let Some(mut me) = self.map_editing_mut(map_id) {
                    let change = me.merge_selected_layers();
                    ActionResult::new(change, true)
                } else {
                    ActionResult::NONE
                }
            }

            Action::MoveSelectedLayersHigher { map_id } => {
                if let Some(mut me) = self.map_editing_mut(map_id) {
                    let change = me.move_selected_layers_higher();
                    ActionResult::new(change, true)
                } else {
                    ActionResult::NONE
                }
            }
            Action::MoveSelectedLayersLower { map_id } => {
                if let Some(mut me) = self.map_editing_mut(map_id) {
                    let change = me.move_selected_layers_lower();
                    ActionResult::new(change, true)
                } else {
                    ActionResult::NONE
                }
            }

            Action::OnSave { path } => {
                self.state.on_save(path);
                // Don't create a new revision for this, it isn't relevant for undo
                // Note that undoing across this action may set relative paths back
                // to absolute, but this will be set to relative as needed on next save.
                ActionResult::NONE
            }
        };

        // Update state to reflect result of action - this includes updating
        // the state's revision as needed
        self.state.add_action_result(&result);
    }
}
