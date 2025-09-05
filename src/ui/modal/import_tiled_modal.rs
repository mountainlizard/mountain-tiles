use egui::{Id, Modal, Ui};

use crate::{
    app::App,
    data::{
        action::Action,
        modal::{ModalResult, ModalState},
    },
    ui::theme::DEFAULT_THEME,
};

pub fn import_tiled_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::ImportTiled {
        tiled,
        ref mut squash_layers,
        ref mut prefer_relative_path,
        ref mut result,
    } = &mut app.edit.modal
    {
        Modal::new(Id::new("Import Tiled Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Import Tiled");
                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.label("Imports a Tiled map in XML (.tmx, .xml) or JSON (.tmj, .json) format. This map and any required tilesets and palette entries are added to the current project.");
                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label(format!("Tiled, {} layers", tiled.layers.len()));

                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.checkbox(squash_layers, "Squash layers");

                ui.checkbox(prefer_relative_path, "Use relative paths for tilesets");

                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Import").clicked() {
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

    if let Some(ModalState::ImportTiled {
        tiled,
        squash_layers,
        prefer_relative_path,
        ..
    }) = modal_to_apply
    {
        app.act(Action::AppendTiledMap {
            tiled,
            squash_layers,
            prefer_relative_path,
        });
    }
}
