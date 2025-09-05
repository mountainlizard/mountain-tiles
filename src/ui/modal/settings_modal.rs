use egui::{Id, Modal, Ui};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    ui::theme::DEFAULT_THEME,
};

pub fn settings_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::Settings {
        ref mut settings,
        ref mut result,
    } = app.edit.modal
    {
        Modal::new(Id::new("Application Settings Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Application Settings");

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.checkbox(
                    &mut settings.tileset_grid_spacing_enabled,
                    "Show gridlines in tilesets",
                );

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Apply").clicked() {
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

    // Note, to avoid issues calling app methods while still
    // holding mutable ref, we produce an optional setting, and apply it here
    if let Some(ModalState::Settings { settings, .. }) = modal_to_apply {
        app.settings = settings;
    }
}
