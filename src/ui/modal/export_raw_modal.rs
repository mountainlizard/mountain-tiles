use egui::{Id, Modal, Ui};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    ui::theme::DEFAULT_THEME,
};

pub fn export_raw_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::ExportRaw {
        ref mut settings,
        ref mut result,
    } = app.edit.modal
    {
        Modal::new(Id::new("Export Raw Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Export Raw Data");

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.label("Exports all maps as selected raw formats, with optional image files for combined tilesets.");
                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.checkbox(
                    &mut settings.export_combined_1bit_tileset,
                    "Export combined tilesets as stacked 1bit raw image",
                );

                ui.checkbox(
                    &mut settings.export_combined_png_tileset,
                    "Export combined tilesets as stacked png image",
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
    if let Some(ModalState::ExportRaw { settings, .. }) = modal_to_apply {
        app.show_export_raw_file_modal(&settings);
    }
}
