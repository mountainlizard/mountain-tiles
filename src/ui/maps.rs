use egui::Ui;

use crate::{app::App, ui::egui_utils::square_button, ui::theme::DEFAULT_THEME};

pub fn maps_ui(ui: &mut Ui, app: &mut App) {
    ui.horizontal(|ui| {
        if app.selected_map().is_some() {
            if square_button(ui, "󰐕").clicked() {
                app.show_new_map_modal();
            }

            if square_button(ui, "󰍴").clicked() {
                app.delete_selected_map();
            }

            if square_button(ui, "󰏫").clicked() {
                app.show_edit_selected_map_modal();
            }

            let selected_text = app
                .selected_map()
                .map(|t| format!("{}", t))
                .unwrap_or("".to_string());

            if let Some(selected_id) = app.edit.selected_map_id.as_mut() {
                // Make combo box match button size - it uses interact size for height
                ui.style_mut().spacing.interact_size.y = DEFAULT_THEME.control_height;

                egui::ComboBox::from_id_salt("select_map")
                    .selected_text(selected_text)
                    .truncate()
                    .width(ui.available_width())
                    .height(DEFAULT_THEME.control_height)
                    .show_ui(ui, |ui| {
                        for map in app.state.maps.iter() {
                            ui.selectable_value(selected_id, map.id(), format!("{map}"));
                        }
                    });
            }
        } else if ui
            .add_sized(
                [ui.available_width(), DEFAULT_THEME.control_height],
                egui::Button::new("󰐕 Add map..."),
            )
            .clicked()
        {
            app.show_new_map_modal();
        };
    });
}
