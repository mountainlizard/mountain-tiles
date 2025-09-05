use egui::{Id, Modal, Slider, Ui};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    ui::theme::DEFAULT_THEME,
};

pub fn export_png_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::ExportPng {
        ref mut settings,
        ref mut result,
    } = app.edit.modal
    {
        Modal::new(Id::new("Export PNG Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Export PNG");

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.checkbox(&mut settings.transparent, "Background transparent");

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label("Scale factor");
                ui.add(
                    Slider::new(&mut settings.scale, 1..=32).clamping(egui::SliderClamping::Always),
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
    if let Some(ModalState::ExportPng { settings, .. }) = modal_to_apply {
        app.show_export_png_file_modal(&settings);
    }
}
