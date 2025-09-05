// use egui::{pos2, vec2, DragPanButtons, Rect, Sense, Ui};
use egui::{vec2, Layout, Ui, WidgetText};

use crate::{
    app::App,
    data::tiles::{tile_color::UserColor, tileset_tiles::TilesetTiles, Tiles},
    geom::u32size2::U32Size2,
    ui::egui_utils::square_button,
    ui::theme::DEFAULT_THEME,
    ui::tiles::{tiles, Overlay},
};

fn tileset_tiles_ui(ui: &mut Ui, app: &mut App) {
    if let Some(tileset) = app.selected_tileset() {
        match app.textures.texture_for_tileset(ui.ctx(), tileset) {
            Ok(_) => {
                let map_size = tileset.size_in_tiles;
                let tile_size = tileset.tile_size;
                let scene_size = ui.available_width();
                let gap = if app.settings.tileset_grid_spacing_enabled {
                    U32Size2::ONE
                } else {
                    U32Size2::ZERO
                };

                let mut tileset_tiles = TilesetTiles {
                    foreground: tileset.foreground.unwrap_or(UserColor::WHITE),
                    background: tileset.background.unwrap_or(UserColor::BLACK),
                    tileset_id: tileset.id(),
                    tile_size,
                    map_size,
                    scale: 1.0,
                    gap,
                };

                let scale = tileset_tiles.scale_for_square_size(scene_size);
                tileset_tiles.scale = scale;

                ui.allocate_ui(vec2(scene_size, scene_size), |ui| {
                    // scene.show(ui, &mut scene_rect, |ui| {
                    let palette = &app.state.resources.palette();
                    let tilesets = &app.state.resources.tilesets();
                    let textures = &app.textures;
                    let tile_selection_response = ui.add(tiles(
                        &tileset_tiles,
                        palette,
                        tilesets,
                        textures,
                        app.edit.selected_tileset_tile_selection(),
                        Overlay::None,
                    ));

                    if let (Some(event), Some(selection)) = (
                        tileset_tiles.event(&tile_selection_response),
                        app.edit.selected_tileset_tile_selection_mut(),
                    ) {
                        let shift = ui.input(|i| i.modifiers.shift);
                        let command = ui.input(|i| i.modifiers.command);

                        let updated = selection.apply_tile_event(&event, shift, command);
                        if updated {
                            app.stamp_from_tileset(&tileset_tiles);
                            app.draw_mode();
                        }
                    }
                });
            }
            Err(e) => {
                tileset_message(ui, format!("󰩋 {}", e), ui.available_width());
            }
        }
    } else if !app.state.resources.tilesets().is_empty() {
        tileset_message(ui, "No tilesets - add one below", ui.available_width());
    } else {
        tileset_message(ui, "No tileset selected", ui.available_width());
    }
}

pub fn tileset_message(ui: &mut Ui, text: impl Into<WidgetText>, size: f32) {
    DEFAULT_THEME.base_200_frame(0).show(ui, |ui| {
        ui.add_sized(vec2(size, size), tileset_label(text));
    });
}

pub fn tileset_label(text: impl Into<WidgetText>) -> impl egui::Widget {
    move |ui: &mut egui::Ui| {
        ui.with_layout(
            Layout::centered_and_justified(eframe::egui::Direction::TopDown)
                .with_cross_justify(false),
            |ui| {
                ui.label(text);
            },
        )
        .response
    }
}

pub fn tileset_ui(ui: &mut Ui, app: &mut App) {
    tileset_tiles_ui(ui, app);

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if app.selected_tileset().is_some() {
            if square_button(ui, "󰐕").clicked() {
                app.show_new_tileset_modal();
            }

            if square_button(ui, "󰍴").clicked() {
                app.delete_selected_tileset();
            }

            if square_button(ui, "󰑐").clicked() {
                app.refresh_selected_tileset(ui.ctx());
            }

            if square_button(ui, "󰏫").clicked() {
                app.show_edit_tileset_modal();
            }

            let selected_text = app
                .selected_tileset()
                .map(|t| format!("{}", t))
                .unwrap_or("".to_string());

            if let Some(selected_id) = app.edit.selected_tileset_id.as_mut() {
                // Make combo box match button size - it uses interact size for height
                ui.style_mut().spacing.interact_size.y = DEFAULT_THEME.control_height;

                egui::ComboBox::from_id_salt("select_tileset")
                    .selected_text(selected_text)
                    .truncate()
                    .width(ui.available_width())
                    .height(DEFAULT_THEME.control_height)
                    .show_ui(ui, |ui| {
                        for tileset in app.state.resources.tilesets().iter() {
                            ui.selectable_value(selected_id, tileset.id(), format!("{tileset}"));
                        }
                    });
            }
        } else if ui
            .add_sized(
                [ui.available_width(), DEFAULT_THEME.control_height],
                egui::Button::new("󰐕 Add tileset..."),
            )
            .clicked()
        {
            app.show_new_tileset_modal();
        };
    });
}
