use egui::{Id, Modal, Ui};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    ui::theme::DEFAULT_THEME,
};

pub fn export_tiled_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::ExportTiled {
        ref mut settings,
        ref mut result,
    } = app.edit.modal
    {
        Modal::new(Id::new("Export Tiled Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Export Tiled Map");

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.label("Exports the currently selected map as a .tmx file, with optional .tsx files for tilesets.");
                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.checkbox(
                    &mut settings.include_layer_data_as_properties,
                    "Include layer data as properties",
                );

                ui.checkbox(
                    &mut settings.export_tsx_files,
                    "Export tilesets as .tsx files",
                );

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Export").clicked() {
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
    if let Some(ModalState::ExportTiled { settings, .. }) = modal_to_apply {
        app.show_export_tiled_file_modal(&settings);
    }
}
