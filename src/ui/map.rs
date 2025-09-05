use egui::{DragPanButtons, Sense, Ui};

use crate::{
    app::App,
    data::tiles::{stamp_tiles::StampTiles, tile_selection::TileSelection, Tiles},
    data::{action::Action, mode::Mode},
    ui::tiles::{tiles_no_sense, Overlay},
};

pub fn map_ui(ui: &mut Ui, app: &mut App) {
    // Only sense drag, to avoid delay in starting drag,
    // and only allow built-in drag panning with right mouse button
    // Note that we don't sense anything in the tiles ui's, because
    // it's easier and better to just let interactions drop through to
    // the scene to handle them there. This means we can detect hover and
    // drag outside the map area, and we don't have drags that start inside
    // the tiles stopping when they reach the edge, which is interferes with
    // drawing.
    let scene = egui::Scene::new()
        .zoom_range(0.1..=6.0)
        .sense(Sense::drag())
        .drag_pan_buttons(DragPanButtons::SECONDARY);

    let mode = app.edit.mode;

    // Note that we produce an action from rendering, so we can carry it out later
    // when app is no longer borrowed
    let action = if let Some(mut me) = app.selected_map_editing_mut() {
        let mut scene_rect = me.edit.scene_rect;
        let map_hovered = me.edit.map_hovered;
        let map_scene_hovered = me.edit.map_scene_hovered;
        let layer_to_stamp_layer = me.layer_index_to_stamp_layer_index();
        let selection = me.edit.selection_mut();

        let double_response = scene.show(ui, &mut scene_rect, |ui| {
            let palette = me.resources.palette();
            let tilesets = me.resources.tilesets();
            let textures = me.textures;
            let response = match mode {
                Mode::Select => ui.add(tiles_no_sense(
                    me.map.tiles(),
                    palette,
                    tilesets,
                    textures,
                    Some(selection),
                    Overlay::None,
                )),
                Mode::Draw => {
                    if let Some(offset) = map_scene_hovered {
                        let render_tiles = StampTiles {
                            stamp: me.stamp,
                            offset,
                            layer_to_stamp_layer: &layer_to_stamp_layer,
                            inner_tiles: me.map.tiles(),
                        };
                        ui.add(tiles_no_sense(
                            &render_tiles,
                            palette,
                            tilesets,
                            textures,
                            None,
                            Overlay::None,
                        ))
                    } else {
                        ui.add(tiles_no_sense(
                            me.map.tiles(),
                            palette,
                            tilesets,
                            textures,
                            None,
                            Overlay::None,
                        ))
                    }
                }
                Mode::Erase => ui.add(tiles_no_sense(
                    me.map.tiles(),
                    palette,
                    tilesets,
                    textures,
                    Some(&TileSelection::erase(map_hovered)),
                    Overlay::None,
                )),
            };
            response
        });

        let tiles_response = double_response.inner;
        let scene_response = double_response.response;

        let action = if let Some(event) = me.map.tiles().scene_event(&scene_response) {
            match mode {
                Mode::Select => {
                    let shift = ui.input(|i| i.modifiers.shift);
                    let command = ui.input(|i| i.modifiers.command);
                    let tile_event = event.as_tile_event(me.map.tiles().map_size());

                    selection.apply_tile_event(&tile_event, shift, command);
                    None
                }

                Mode::Draw => {
                    let pos = event.pos();
                    let complete = event.complete();
                    Some(Action::Draw {
                        map_id: me.map.id(),
                        pos,
                        complete,
                    })
                }
                Mode::Erase => {
                    let pos = event.pos();
                    let complete = event.complete();
                    Some(Action::Erase {
                        map_id: me.map.id(),
                        pos,
                        complete,
                    })
                }
            }
        } else {
            None
        };

        // Remember where map and scene were hovered, to draw next frame
        me.update_map_hover(&tiles_response, &scene_response);

        // Remember any pan/zoom applied by user
        me.edit.scene_rect = scene_rect;

        action
    } else {
        None
    };

    // Perform any action needed for editing
    if let Some(action) = action {
        app.act(action);
    }
}
