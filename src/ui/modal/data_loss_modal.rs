use egui::{Color32, Id, Modal, Stroke, Ui};

use crate::{
    app::{files::OpenContext, App},
    data::{
        action::Action,
        modal::{DataLossOperation, ModalResult, ModalState},
    },
    ui::theme::DEFAULT_THEME,
};

pub fn data_loss_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::DataLoss {
        ref operation,
        ref mut result,
    } = &mut app.edit.modal
    {
        Modal::new(Id::new("Data Loss Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                match operation {
                    DataLossOperation::DeleteTileset {
                        tileset_name,
                        tileset_use,
                        ..
                    } => {
                        ui.heading("󱂥 Tileset is in use");
                        ui.add_space(DEFAULT_THEME.modal_spacing);

                        ui.label(tileset_name);
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                        ui.label(format!(
                            "Used by {} tiles, on maps/layers:",
                            tileset_use.tile_count
                        ));
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                        ui.label(tileset_use.locations_to_string());
                    }
                    DataLossOperation::ReplacePalette {
                        palette,
                        resource_use,
                    } => {
                        ui.heading("󱂥 Deleted colors are in use");
                        ui.add_space(DEFAULT_THEME.modal_spacing);

                        ui.label(format!(
                            "The current palette has {} colors, the new one has {}.",
                            app.state.resources.palette().len(),
                            palette.len()
                        ));
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                        ui.label(format!(
                            "Deleted colors are used by {} tiles, on maps/layers:",
                            resource_use.tile_count
                        ));
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                        ui.label(resource_use.locations_to_string());
                    }
                    _ => {
                        ui.heading("󱂥 Warning - unsaved data");
                        ui.add_space(DEFAULT_THEME.modal_spacing);
                        ui.label(
                            "There is unsaved data, do you wish to \
                        continue and delete data, or cancel?",
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
                        ui.scope(|ui| {
                            ui.style_mut().visuals.widgets.inactive.bg_stroke =
                                Stroke::new(1.0, Color32::RED);
                            ui.style_mut().visuals.widgets.active.bg_stroke =
                                Stroke::new(1.0, Color32::RED);
                            ui.style_mut().visuals.widgets.hovered.bg_stroke =
                                Stroke::new(1.0, Color32::RED);

                            let text = match operation {
                                DataLossOperation::DeleteTileset { tileset_use, .. } => {
                                    format!("Delete tileset and {} tiles", tileset_use.tile_count)
                                }
                                DataLossOperation::ReplacePalette { resource_use, .. } => {
                                    format!(
                                        "Delete palette colors and {} tiles",
                                        resource_use.tile_count
                                    )
                                }
                                _ => "Continue (delete data)".to_string(),
                            };
                            if ui.button(text).clicked() {
                                *result = ModalResult::Apply;
                            };
                        });
                        if ui.button("Cancel").clicked() {
                            *result = ModalResult::Cancel;
                        };
                    },
                );
            });

        app.progress_modal_state()
    } else {
        None
    };

    if let Some(ModalState::DataLoss { operation, .. }) = modal_to_apply {
        match operation {
            DataLossOperation::New => {
                app.new_document();
            }
            DataLossOperation::Open => {
                app.show_open_document_modal();
            }
            DataLossOperation::OpenFileArgument { path } => {
                app.open_document(path, OpenContext::FileArgument);
            }
            DataLossOperation::ImportTiled => {
                app.pick_tiled_file_to_import();
            }
            DataLossOperation::DeleteTileset { tileset_id, .. } => {
                app.act(Action::DeleteTileset { id: tileset_id });
            }
            DataLossOperation::ReplacePalette { palette, .. } => {
                app.act(Action::ReplacePalette { palette });
            }
            DataLossOperation::Quit => {
                app.quit(ui.ctx());
            }
        };
    }
}
