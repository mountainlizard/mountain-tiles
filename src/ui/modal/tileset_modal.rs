use egui::{Color32, Id, Modal, Slider, Ui};

use crate::{
    app::App,
    data::tiles::{tile_color::UserColor, tileset_tiles::TilesetTiles, Tiles},
    data::{
        action::Action,
        modal::{ModalResult, ModalState, TilesetOperation},
    },
    geom::u32size2::U32Size2,
    ui::file_dialog,
    ui::theme::DEFAULT_THEME,
    ui::tiles::{tiles, Overlay},
    ui::{tileset::tileset_message, utils::optional_color_ui},
};

const PREVIEW_SIZE: f32 = 256.0;
const OVERLAY_COLOR: Color32 = DEFAULT_THEME.selected_fill;
const OVERLAY: Overlay = Overlay::Checkerboard {
    color: OVERLAY_COLOR,
};

pub fn tileset_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::Tileset {
        ref mut tilesets,
        ref operation,
        ref mut default_foreground,
        ref mut default_foreground_as_text,
        ref mut default_background,
        ref mut default_background_as_text,
        ref mut result,
    } = app.edit.modal
    {
        Modal::new(Id::new("Tileset Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                match operation {
                    TilesetOperation::NewTileset => ui.heading("New Tileset"),
                    TilesetOperation::UpdateExistingTileset(_) => ui.heading("Tileset Settings"),
                };

                ui.add_space(DEFAULT_THEME.modal_spacing);

                if let Some(tileset) = tilesets.last_mut() {
                    let path_text = if tileset.path.components().next().is_some() {
                        tileset.path.to_string()
                    } else {
                        "No file selected...".to_string()
                    };
                    ui.label(path_text);

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

                    ui.label("Tile width (in pixels)");
                    ui.add(
                        Slider::new(&mut tileset.tile_size.w, 4..=32)
                            .clamping(egui::SliderClamping::Never),
                    );

                    ui.label("Tile height (in pixels)");
                    ui.add(
                        Slider::new(&mut tileset.tile_size.h, 4..=32)
                            .clamping(egui::SliderClamping::Never),
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

                    ui.add_space(DEFAULT_THEME.modal_spacing);

                    match app.textures.texture_for_tileset(ui.ctx(), tileset) {
                        Ok(texture_poll) => match texture_poll.size() {
                            Some(image_size) => {
                                let image_size_pixels = U32Size2::lossy_from_vec2(&image_size);
                                let size_in_tiles = image_size_pixels / tileset.tile_size;
                                tileset.size_in_tiles = size_in_tiles;

                                ui.label(format!(
                                    "Preview ({}x{} px 󰁔 {}x{} tiles)",
                                    image_size_pixels.w,
                                    image_size_pixels.h,
                                    size_in_tiles.w,
                                    size_in_tiles.h
                                ));

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
                            }
                            None => {
                                ui.label("Preview");

                                tileset_message(ui, "Loading image...", PREVIEW_SIZE);
                            }
                        },
                        Err(_) => {
                            ui.label("Preview");

                            tileset_message(
                                ui,
                                "Select a valid image file (e.g. png) above",
                                PREVIEW_SIZE,
                            );
                        }
                    }
                }

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

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
            });

        app.progress_modal_state()
    } else {
        None
    };

    if let Some(ModalState::Tileset {
        mut tilesets,
        operation,
        ..
    }) = modal_to_apply
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
