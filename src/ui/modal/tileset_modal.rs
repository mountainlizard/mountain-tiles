use egui::{Color32, Id, Label, Modal, Slider, Ui};

use crate::{
    app::App,
    data::{
        action::Action,
        modal::{ModalResult, ModalState, TilesetOperation},
        tiles::{tile_color::UserColor, tileset_tiles::TilesetTiles, Tiles},
        tilesets::TilesetMode,
    },
    geom::u32size2::U32Size2,
    ui::{
        file_dialog,
        theme::DEFAULT_THEME,
        tiles::{tiles, Overlay},
        tileset::tileset_message,
        utils::{optional_color_ui, user_color_edit_button},
    },
};

const PREVIEW_SIZE: f32 = 256.0;
const OVERLAY_COLOR: Color32 = DEFAULT_THEME.selected_fill;
const OVERLAY: Overlay = Overlay::Checkerboard {
    color: OVERLAY_COLOR,
};

pub fn tileset_settings_ui(ui: &mut Ui, app: &mut App) {
    if let ModalState::Tileset {
        ref mut tilesets,
        ref mut default_foreground,
        ref mut default_foreground_as_text,
        ref mut default_background,
        ref mut default_background_as_text,
        ref mut default_transparent,
        ref mut default_transparent_as_text,
        ..
    } = app.edit.modal
    {
        if let Some(tileset) = tilesets.last_mut() {
            ui.add_space(DEFAULT_THEME.modal_spacing - 8.0);
            let path_text = if tileset.path.components().next().is_some() {
                tileset.path.to_string()
            } else {
                "No file selected...".to_string()
            };
            ui.add(Label::new(path_text).truncate());

            ui.add_space(DEFAULT_THEME.modal_spacing);

            ui.horizontal(|ui| {
                if ui.button("Browse...").clicked() {
                    if let Ok(Some(path)) = file_dialog::pick_file() {
                        if let Some(file_stem) = path.file_stem() {
                            tileset.name = file_stem.to_string();
                        }
                        tileset.path = path;
                        app.textures.refresh_tileset(ui.ctx(), tileset);
                    }
                }
                ui.add_space(6.0);

                if ui.button("󰑐 Reload image").clicked() {
                    app.textures.refresh_tileset(ui.ctx(), tileset);
                }
            });

            ui.add_space(DEFAULT_THEME.modal_spacing);

            ui.label("Name");
            ui.text_edit_singleline(&mut tileset.name);

            ui.add_space(DEFAULT_THEME.modal_spacing);

            let selected_text = tileset.mode.description();
            let direct = TilesetMode::Direct;
            let transparent = TilesetMode::TransparentBackground {
                background: *default_transparent,
            };
            let foreground_background = TilesetMode::ForegroundBackground {
                background: *default_transparent,
            };
            egui::ComboBox::from_id_salt("tileset_mode")
                .selected_text(selected_text)
                .truncate()
                .width(ui.available_width())
                .height(DEFAULT_THEME.control_height)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut tileset.mode, direct, direct.description());
                    ui.selectable_value(&mut tileset.mode, transparent, transparent.description());
                    ui.selectable_value(
                        &mut tileset.mode,
                        foreground_background,
                        foreground_background.description(),
                    );
                });

            match tileset.mode {
                TilesetMode::Direct => {}
                TilesetMode::TransparentBackground { ref mut background } => {
                    user_color_edit_button(ui, background, default_transparent_as_text);
                    *default_transparent = *background;
                }
                TilesetMode::ForegroundBackground { ref mut background } => {
                    user_color_edit_button(ui, background, default_transparent_as_text);
                    *default_transparent = *background;
                }
            }

            ui.add_space(DEFAULT_THEME.modal_spacing);

            ui.label("Tile width (in pixels)");
            ui.add(
                Slider::new(&mut tileset.tile_size.w, 4..=32).clamping(egui::SliderClamping::Never),
            );

            ui.label("Tile height (in pixels)");
            ui.add(
                Slider::new(&mut tileset.tile_size.h, 4..=32).clamping(egui::SliderClamping::Never),
            );

            ui.add_space(DEFAULT_THEME.modal_spacing);
            optional_color_ui(
                ui,
                &mut tileset.foreground,
                "Foreground",
                default_foreground,
                default_foreground_as_text,
            );

            ui.add_space(DEFAULT_THEME.modal_spacing);

            optional_color_ui(
                ui,
                &mut tileset.background,
                "Background",
                default_background,
                default_background_as_text,
            );
        }
    }
}

pub fn tileset_preview_ui(ui: &mut Ui, app: &mut App) {
    if let ModalState::Tileset {
        ref mut tilesets, ..
    } = app.edit.modal
    {
        if let Some(tileset) = tilesets.last_mut() {
            match app.textures.texture_for_tileset(ui.ctx(), tileset) {
                Ok(texture_poll) => match texture_poll.size() {
                    Some(image_size) => {
                        let image_size_pixels = U32Size2::lossy_from_vec2(&image_size);
                        let size_in_tiles = image_size_pixels / tileset.tile_size;
                        tileset.size_in_tiles = size_in_tiles;

                        let palette = app.state.resources.palette();
                        let textures = &app.textures;

                        let mut tileset_tiles = TilesetTiles {
                            foreground: tileset.foreground.unwrap_or(UserColor::WHITE),
                            background: tileset.background.unwrap_or(UserColor::BLACK),
                            tileset_id: tileset.id(),
                            tile_size: tileset.tile_size,
                            map_size: tileset.size_in_tiles,
                            scale: 1.0,
                            gap: U32Size2::ZERO,
                        };

                        let scale = tileset_tiles.scale_for_square_size(PREVIEW_SIZE);
                        tileset_tiles.scale = scale;

                        ui.add(tiles(
                            &tileset_tiles,
                            palette,
                            tilesets,
                            textures,
                            None,
                            OVERLAY,
                        ));
                        ui.label(format!(
                            "Preview ({}x{} px 󰁔 {}x{} tiles)",
                            image_size_pixels.w,
                            image_size_pixels.h,
                            size_in_tiles.w,
                            size_in_tiles.h
                        ));
                    }
                    None => {
                        tileset_message(ui, "Loading image...", PREVIEW_SIZE);
                        ui.label("Preview");
                    }
                },
                Err(_) => {
                    tileset_message(
                        ui,
                        "Select a valid image file (e.g. png) above",
                        PREVIEW_SIZE,
                    );
                    ui.label("Preview");
                }
            }
        }
    }
}

pub fn tileset_buttons_ui(ui: &mut Ui, app: &mut App) {
    if let ModalState::Tileset {
        ref operation,
        ref mut result,
        ..
    } = app.edit.modal
    {
        egui::Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                let confirm_text = match operation {
                    TilesetOperation::NewTileset => "Create",
                    TilesetOperation::UpdateExistingTileset(_) => "Apply",
                };
                if ui.button(confirm_text).clicked() {
                    *result = ModalResult::Apply;
                }
                if ui.button("Cancel").clicked() {
                    *result = ModalResult::Cancel;
                }
            },
        );
    }
}

pub fn tileset_header_ui(ui: &mut Ui, app: &mut App) {
    if let ModalState::Tileset { ref operation, .. } = app.edit.modal {
        match operation {
            TilesetOperation::NewTileset => ui.heading("New Tileset"),
            TilesetOperation::UpdateExistingTileset(_) => ui.heading("Tileset Settings"),
        };
    }
}

pub fn tileset_modal_ui(ui: &mut Ui, app: &mut App) {
    if !matches!(app.edit.modal, ModalState::Tileset { .. }) {
        return;
    }

    Modal::new(Id::new("Tileset Modal"))
        .frame(DEFAULT_THEME.modal_frame())
        .show(ui.ctx(), |ui| {
            ui.set_height(430.0);

            egui::TopBottomPanel::top("top_panel")
                .resizable(false)
                .default_height(48.0)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    tileset_header_ui(ui, app);
                });

            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.add_space(DEFAULT_THEME.modal_spacing);
                    tileset_buttons_ui(ui, app);
                });

            egui::SidePanel::left("left_panel")
                .resizable(false)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                        tileset_settings_ui(ui, app);
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(DEFAULT_THEME.modal_spacing);
                    tileset_preview_ui(ui, app);
                    ui.add_space(DEFAULT_THEME.modal_spacing);
                });
            });

            // ui.set_width(250.0);

            // tileset_header_ui(ui, app);

            // ui.add_space(DEFAULT_THEME.modal_spacing);

            // ui.horizontal_top(|ui| {
            //     ui.vertical(|ui| {
            //         tileset_settings_ui(ui, app);
            //     });
            //     ui.vertical_centered_justified(|ui| {
            //         tileset_preview_ui(ui, app);
            //     });
            // });

            // ui.add_space(DEFAULT_THEME.modal_spacing);
            // ui.separator();
            // ui.add_space(DEFAULT_THEME.modal_spacing);

            // tileset_buttons_ui(ui, app);
        });

    if let Some(ModalState::Tileset {
        mut tilesets,
        operation,
        ..
    }) = app.progress_modal_state()
    {
        if let Some(tileset) = tilesets.pop() {
            match operation {
                TilesetOperation::NewTileset => {
                    app.act(Action::AddTileset { tileset });
                }
                TilesetOperation::UpdateExistingTileset(id) => {
                    app.act(Action::UpdateTileset { id, tileset });
                }
            }
        }
    }
}
