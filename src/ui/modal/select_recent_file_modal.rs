use egui::{Id, Modal, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    ui::{egui_utils::unselectable_label, theme::DEFAULT_THEME},
};

pub fn select_recent_file_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::SelectRecentFile {
        ref mut selected_index,
        ref mut result,
    } = app.edit.modal
    {
        Modal::new(Id::new("Select Recent File Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Select Recent File");

                ui.add_space(DEFAULT_THEME.modal_spacing);

                if app.recent_paths.is_empty() {
                    unselectable_label(ui, "No recent files");
                } else {
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::remainder().clip(true))
                        .min_scrolled_height(0.0)
                        .max_scroll_height(99999999.0)
                        .sense(egui::Sense::click());

                    table.body(|mut body| {
                        // Display layers reversed so that the layer with the highest index, which is
                        // drawn over all other layers, is on first row of table, i.e. on the "top"
                        for (index, path) in app.recent_paths.iter().enumerate() {
                            let selected = *selected_index == index;
                            body.row(DEFAULT_THEME.row_height, |mut row| {
                                row.set_selected(selected);
                                row.col(|ui| {
                                    unselectable_label(
                                        ui,
                                        format!(
                                            "{}. {}",
                                            index + 1,
                                            path.file_name().unwrap_or(path.as_str())
                                        ),
                                    );
                                });
                                if row.response().clicked() {
                                    *selected_index = index;
                                }
                                if row.response().double_clicked() {
                                    *selected_index = index;
                                    *result = ModalResult::Apply;
                                }
                            });
                        }
                    });

                    if let Some(path) = app.recent_paths.get(*selected_index) {
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                        ui.label(
                            RichText::new(format!("{}", path)).color(DEFAULT_THEME.base_subcontent),
                        );
                    }
                }

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Select").clicked() {
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
    // holding mutable ref, we progress the modal state above, and respond to it here
    if let Some(ModalState::SelectRecentFile { selected_index, .. }) = modal_to_apply
        && let Some(path) = app.recent_paths.get(selected_index)
    {
        app.check_data_loss_then_open_document_from_file_argument(path.clone());
    }
}
