use egui::{Id, Modal, Ui};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    ui::theme::DEFAULT_THEME,
};

pub fn error_modal_ui(ui: &mut Ui, app: &mut App) {
    if let ModalState::Error {
        ref message,
        ref mut result,
    } = &mut app.edit.modal
    {
        Modal::new(Id::new("Error Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Error");
                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label(message);

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Ok").clicked() {
                            *result = ModalResult::Apply;
                        }
                    },
                );
            });

        app.progress_modal_state();
    }
}
