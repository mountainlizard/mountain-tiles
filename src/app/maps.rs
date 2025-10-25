use egui::Response;

use crate::{
    app::App,
    data::{
        action::Action,
        map_edit_state::MapEditState,
        maps::{Map, MapId},
        mode::Mode,
        resources::Resources,
        stamp::{Stamp, TileLocation},
        tiles::{tile_color::UserColor, Tiles},
    },
    geom::{i32pos2::I32Pos2, u32pos2::u32pos2, u32size2::U32Size2},
    selection::{apply_default_selection, apply_default_value_per_selectable_id},
    ui::tileset_textures::TilesetTextures,
};

pub struct MapEditing<'a> {
    pub map: &'a mut Map,
    pub edit: &'a mut MapEditState,
    pub mode: Mode,
    pub stamp: &'a mut Stamp,
    pub resources: &'a mut Resources,
    pub textures: &'a TilesetTextures,
}

impl<'a> MapEditing<'a> {
    pub fn selected_layer_indices(&self) -> Vec<usize> {
        let selection = &self.edit.layer_selection;
        self.map
            .tiles
            .layers()
            .enumerate()
            .filter(|(_index, layer)| selection.is_selected(layer.id()))
            .map(|(index, _layer)| index)
            .collect()
    }

    /// Find the indices of the layers of our associated [`Map`] that are selected and visible,
    /// in increasing order (lowest to highest index).
    pub fn selected_visible_layer_indices(&self) -> Vec<usize> {
        let selection = &self.edit.layer_selection;
        self.map
            .tiles()
            .layers()
            .enumerate()
            .filter(|(_index, layer)| layer.visible() && selection.is_selected(layer.id()))
            .map(|(index, _layer)| index)
            .collect()
    }

    /// Add an empty layer named "New Layer" before first selected layer, or to the start of the list if there is no selected layer
    pub fn add_layer(&mut self) {
        let i = self.selected_layer_indices().first().copied().unwrap_or(0);
        let layer_id = self.map.tiles.insert_layer(i, "New Layer", true, None);
        self.edit.layer_selection.select_only(layer_id);
    }

    pub fn delete_selected_layers(&mut self) -> bool {
        self.edit
            .layer_selection
            .delete_selected_items(&mut self.map.tiles)
    }

    pub fn merge_selected_layers(&mut self) -> bool {
        let mut layer_indices = self.selected_layer_indices();
        if layer_indices.len() < 2 {
            false
        } else {
            let tiles = self.map.tiles_mut();
            let top_layer_index = layer_indices.remove(0);
            let top_layer_id = tiles.layer_id(top_layer_index);
            for pos in tiles.map_positions() {
                if tiles.tile(top_layer_index, pos).is_none() {
                    for layer_index in layer_indices.iter() {
                        if let Some(tile) = tiles.tile(*layer_index, pos) {
                            tiles.set_tile(top_layer_index, pos, Some(tile));
                            break;
                        }
                    }
                }
            }

            // Remove the merged layers
            for layer_index in layer_indices.iter().rev() {
                tiles.remove_layer_by_index(*layer_index)
            }

            // Select the top layer we merged into
            if let Some(id) = top_layer_id {
                self.edit.layer_selection.select_only(id);
            }

            true
        }
    }

    pub fn can_move_selected_layers_higher(&self) -> bool {
        self.selected_layer_indices()
            .first()
            .map(|i| *i > 0)
            .unwrap_or(false)
    }

    pub fn can_move_selected_layers_lower(&mut self) -> bool {
        self.selected_layer_indices()
            .last()
            .map(|i| *i < self.map.tiles.layer_count() - 1)
            .unwrap_or(false)
    }

    pub fn move_selected_layers_higher(&mut self) -> bool {
        if self.can_move_selected_layers_higher() {
            for i in self.selected_layer_indices().iter() {
                self.map.tiles.move_layer_higher(*i);
            }
            true
        } else {
            false
        }
    }

    pub fn move_selected_layers_lower(&mut self) -> bool {
        if self.can_move_selected_layers_lower() {
            for i in self.selected_layer_indices().iter().rev() {
                self.map.tiles.move_layer_lower(*i);
            }
            true
        } else {
            false
        }
    }

    /// Produce a mapping from layers of our associated [`Map`] to the
    /// layer index of the provided [`Stamp`] that would be drawn to them.
    /// This is [`None`] if there is no [`Stamp`] layer to be drawn there.
    /// Note this is essentially the inverse of [`State::stamp_layer_index_to_layer_index`]
    pub fn layer_index_to_stamp_layer_index(&self) -> Vec<Option<usize>> {
        // Default to assigning no stamp layers (so nothing is drawn)
        let mut layer_map = vec![None; self.map.tiles().layer_count()];
        let stamp_layer_count = self.stamp.layer_count();

        // Assign stamp layers to visible, selected layers in order.
        for (stamp_layer_index, layer_index) in
            self.selected_visible_layer_indices().iter().enumerate()
        {
            if stamp_layer_index < stamp_layer_count {
                if let Some(target_layer_map) = layer_map.get_mut(*layer_index) {
                    *target_layer_map = Some(stamp_layer_index);
                }
            }
        }
        layer_map
    }

    /// Produce a mapping from layers of the provided [`Stamp`] to the
    /// layer index where that stamp layer would be drawn in our asssociated
    /// [`Map`].
    /// Note that this may be smaller than the number of layers in the
    /// [`Stamp`] - in this case the additional [`Stamp`] layers will be
    /// ignored (e.g. if we have more [`Stamp`] layers than suitable
    /// target layers)
    /// Note this is essentially the inverse of [`MapEditState::layer_index_to_stamp_layer_index`]
    pub fn stamp_layer_index_to_layer_index(&self) -> Vec<usize> {
        // Assign stamp layers to visible, selected layers, in order.
        let stamp_layer_count = self.stamp.layer_count();
        let mut indices = self.selected_visible_layer_indices();
        indices.truncate(stamp_layer_count);
        indices
    }

    /// Update our hover state according to the response from drawing the map
    pub fn update_map_hover(&mut self, tiles_response: &Response, scene_response: &Response) {
        // The logic here is complex because:
        // 1. We only get a hover position for either the map (tiles), or the scene, not both
        //    but we want to produce a combined position.
        // 2. There seems to be a bug where when the scene is zoomed in so that the map entirely
        //    covers it (i.e. none of the maps edges are visible, just part of the map), we start
        //    getting hover positions for the scene again, rather than the tiles.
        // Therefore we need to check for positions using both, and combine them as necessary

        // Start from preferred options, each hover position taken from expected source
        let mut map_hovered = self.map.tiles().hovered(tiles_response);
        let mut scene_hovered = self.map.tiles().scene_hovered(scene_response);

        // If we're zoomed in too far, we may have a scene_hovered that's actually inside the
        // map, in which case use it
        // NOTE: This may become unnecessary if related scene issue is resolved
        if let Some(scene_hovered) = scene_hovered {
            if map_hovered.is_none() && self.map.tiles().map_size().contains(scene_hovered) {
                map_hovered = Some(u32pos2(scene_hovered.x as u32, scene_hovered.y as u32))
            }
        }

        // We want to report a scene_hovered even when we are inside the map - in this case we
        // use map_hovered, converted to a signed position
        scene_hovered = scene_hovered.or(map_hovered.map(|p| p.into()));

        self.edit.map_hovered = map_hovered;
        self.edit.map_scene_hovered = scene_hovered;
    }

    pub fn erase_selected_visible_tiles(&mut self) -> bool {
        let mut changed = false;
        for layer_index in self.selected_visible_layer_indices() {
            for pos in self.edit.selection().iter() {
                if self.map.tiles.set_tile(layer_index, *pos, None) {
                    changed = true;
                }
            }
        }
        changed
    }

    pub fn selection_as_stamp(&self) -> Stamp {
        let mut stamp = Stamp::new();
        let selection = self.edit.selection();
        let layer_tiles = self.map.tiles();

        // If any cells are selected, we have a selection rect, otherwise we can skip to returning
        // empty result
        if let Some(selection_rect) = selection.range_rect() {
            let center: I32Pos2 = selection_rect.center().into();

            for (stamp_layer_index, layer_index) in
                self.selected_visible_layer_indices().iter().enumerate()
            {
                for position in selection.iter() {
                    if let Some(tile) = layer_tiles.tile(*layer_index, *position) {
                        let ipos: I32Pos2 = position.into();
                        stamp.insert(
                            TileLocation {
                                stamp_layer_index,
                                position: ipos - center,
                            },
                            tile,
                        );
                    }
                }
            }
        }
        stamp
    }

    fn reset_zoom(&mut self) {
        self.edit.scene_rect = self.map.tiles.screen_rect();
    }

    pub fn can_delete_selected_layers(&self) -> bool {
        !self.selected_layer_indices().is_empty()
    }

    pub(crate) fn can_merge_selected_layers(&self) -> bool {
        self.selected_layer_indices().len() > 1
    }
}

impl App {
    pub fn add_map(&mut self, map: Map) {
        let id = self.state.maps.push_map(map);
        self.edit.selected_map_id = Some(id);
    }

    /// Call [`apply_default_selection`] on our map selection.
    pub(super) fn apply_default_map_selection(&mut self) {
        apply_default_selection(&mut self.edit.selected_map_id, &self.state.maps);
    }

    /// Call [`apply_default_value_per_selectable_id`] on our map edit states,
    /// then apply invariants on each map edit state
    pub(super) fn apply_default_map_edit_state_by_id(&mut self) {
        apply_default_value_per_selectable_id(
            &mut self.edit.map_edit_state_by_id,
            &self.state.maps,
        );

        // TODO: Use `MapEditing`? flatmap all map ids to MapEditing then run on that
        for (map_id, map_edit_state) in self.edit.map_edit_state_by_id.iter_mut() {
            if let Some(map) = self.state.maps.get_by_id(*map_id) {
                map_edit_state.apply_invariants(map);
            }
        }
    }

    pub fn selected_map(&self) -> Option<&Map> {
        self.edit
            .selected_map_id
            .and_then(|id| self.state.maps.get_by_id(id))
    }

    pub fn selected_map_edit_state(&self) -> Option<&MapEditState> {
        self.edit
            .selected_map_id
            .and_then(|id| self.edit.map_edit_state_by_id.get(&id))
    }

    pub fn selected_map_edit_state_mut(&mut self) -> Option<&mut MapEditState> {
        self.edit
            .selected_map_id
            .and_then(|id| self.edit.map_edit_state_by_id.get_mut(&id))
    }

    pub fn reset_selected_map_zoom(&mut self) {
        if let Some(mut me) = self.selected_map_editing_mut() {
            me.reset_zoom();
        }
    }

    pub(super) fn draw(&mut self, map_id: MapId, pos: I32Pos2) -> bool {
        let mut change = false;
        // If map doesn't exist or has no edit state, there's nothing to be done.
        // Note that missing edit state is interpreted as the default edit state, with no selection
        if let Some(me) = self.map_editing_mut(map_id) {
            let stamp_layer_index_to_layer_index = me.stamp_layer_index_to_layer_index();

            for (location, tile) in me.stamp.tiles.iter() {
                if let Ok(upos) = (location.position + pos).try_into() {
                    // See if we have a layer to draw this stamp layer to - if not, ignore it (e.g.
                    // if we have more stamp layers than suitable map layers)
                    if let Some(layer_index) =
                        stamp_layer_index_to_layer_index.get(location.stamp_layer_index)
                    {
                        if me.resources.is_tile_valid(tile)
                            && me.map.tiles.set_tile(*layer_index, upos, Some(*tile))
                        {
                            change = true;
                        }
                    }
                }
            }
        }
        change
    }

    pub(super) fn erase(&mut self, map_id: MapId, pos: I32Pos2) -> bool {
        let mut changed = false;
        // If map doesn't exist or has no edit state, there's nothing to be done.
        // Note that missing edit state is interpreted as the default edit state, with no selection
        if let Some(me) = self.map_editing_mut(map_id) {
            for layer_index in me.selected_visible_layer_indices() {
                if me.map.tiles.set_tile_i(layer_index, pos, None) {
                    changed = true;
                }
            }
        }
        changed
    }

    pub(super) fn update_map(
        &mut self,
        map_id: MapId,
        name: String,
        map_size: U32Size2,
        tile_size: U32Size2,
        background_color: UserColor,
    ) -> bool {
        if let Some(map) = self.state.maps.get_by_id_mut(map_id) {
            if map.name() != name
                || map.tiles.map_size() != map_size
                || map.tiles.tile_size() != tile_size
                || map.tiles.background() != background_color
            {
                map.name = name;
                map.tiles.resize(map_size);
                map.tiles.set_tile_size(tile_size);
                map.tiles.background = background_color;

                if let Some(map_edit_state) = self.edit.map_edit_state_by_id.get_mut(&map_id) {
                    map_edit_state.selection_mut().clear();
                }

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub(super) fn delete_map(&mut self, id: MapId) -> bool {
        self.state.maps.delete_by_id(id)
    }

    pub fn delete_selected_map(&mut self) {
        if let Some(id) = self.edit.selected_map_id {
            self.act(Action::DeleteMap { id });
        }
    }

    pub fn map_editing_mut(&mut self, map_id: MapId) -> Option<MapEditing<'_>> {
        let map = self.state.maps.get_by_id_mut(map_id);
        let edit = self.edit.map_edit_state_by_id.get_mut(&map_id);

        if let (Some(map), Some(edit)) = (map, edit) {
            Some(MapEditing {
                map,
                edit,
                mode: self.edit.mode,
                stamp: &mut self.edit.stamp,
                resources: &mut self.state.resources,
                textures: &mut self.textures,
            })
        } else {
            None
        }
    }

    pub fn selected_map_editing_mut(&mut self) -> Option<MapEditing<'_>> {
        let map = self
            .edit
            .selected_map_id
            .and_then(|id| self.state.maps.get_by_id_mut(id));

        let edit = self
            .edit
            .selected_map_id
            .and_then(|id| self.edit.map_edit_state_by_id.get_mut(&id));

        if let (Some(map), Some(edit)) = (map, edit) {
            Some(MapEditing {
                map,
                edit,
                mode: self.edit.mode,
                stamp: &mut self.edit.stamp,
                resources: &mut self.state.resources,
                textures: &mut self.textures,
            })
        } else {
            None
        }
    }
}
