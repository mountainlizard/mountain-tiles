use egui::{Id, Modal, Slider, Ui};

use crate::{
    app::App,
    data::{
        action::Action,
        maps::Map,
        modal::{MapOperation, ModalResult, ModalState},
    },
    geom::u32size2::u32size2,
    ui::egui_utils::user_color_edit_button,
    ui::theme::DEFAULT_THEME,
};

pub fn map_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::Map {
        ref mut name,
        ref mut width,
        ref mut height,
        ref mut tile_width,
        ref mut tile_height,
        ref mut background_color,
        ref mut background_color_as_text,
        ref operation,
        ref mut result,
    } = app.edit.modal
    {
        Modal::new(Id::new("Map Settings Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                match operation {
                    MapOperation::NewMap => ui.heading("New Map"),
                    MapOperation::UpdateExistingMap(..) => ui.heading("Map Settings"),
                };

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label("Map name");
                ui.text_edit_singleline(name);

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label("Map width (in tiles)");
                ui.add(Slider::new(width, 1..=256).clamping(egui::SliderClamping::Never));

                ui.label("Map height (in tiles)");
                ui.add(Slider::new(height, 1..=256).clamping(egui::SliderClamping::Never));

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label("Tile width (in pixels)");
                ui.add(Slider::new(tile_width, 4..=32).clamping(egui::SliderClamping::Never));

                ui.label("Tile height (in pixels)");
                ui.add(Slider::new(tile_height, 4..=32).clamping(egui::SliderClamping::Never));

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label("Background");
                user_color_edit_button(ui, background_color, background_color_as_text);

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        let confirm_text = match operation {
                            MapOperation::NewMap => "Create",
                            MapOperation::UpdateExistingMap(..) => "Apply",
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

    if let Some(ModalState::Map {
        name,
        width,
        height,
        tile_width,
        tile_height,
        background_color,
        operation,
        ..
    }) = modal_to_apply
    {
        let map_size = u32size2(width, height);
        let tile_size = u32size2(tile_width, tile_height);

        match operation {
            MapOperation::NewMap => {
                let map = Map::new_with_layer(name, map_size, tile_size, background_color);
                app.act(Action::AddMap { map });
            }
            MapOperation::UpdateExistingMap(map_id) => {
                app.act(Action::UpdateMap {
                    map_id,
                    name,
                    map_size,
                    tile_size,
                    background_color,
                });
            }
        };
    };
}
