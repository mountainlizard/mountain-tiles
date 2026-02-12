use crate::{
    app::App,
    data::palette::Palette,
    data::tiles::Tiles,
    data::{
        maps::MapId,
        modal::{DataLossOperation, ModalResult, ModalState},
    },
    tiled::tiled_json::Tiled,
};

impl App {
    pub fn show_new_map_modal(&mut self) {
        self.edit.show_modal(ModalState::new_map());
    }

    pub fn show_help_modal(&mut self) {
        self.edit.show_modal(ModalState::help());
    }

    pub fn show_application_settings_modal(&mut self) {
        self.edit
            .show_modal(ModalState::settings(self.settings.clone()));
    }

    pub fn show_layer_modal(&mut self, map_id: MapId, layer_index: usize) {
        if let Some(map) = self.state.maps.get_by_id(map_id) {
            if let (Some(layer_id), Some(name), opacity) = (
                map.tiles().layer_id(layer_index),
                map.tiles().layer_name(layer_index),
                map.tiles().layer_opacity(layer_index),
            ) {
                self.edit.show_modal(ModalState::Layer {
                    map_id: map.id(),
                    layer_id,
                    name: name.clone(),
                    opacity,
                    result: ModalResult::Init,
                });
            }
        }
    }

    pub fn show_edit_selected_map_modal(&mut self) {
        if let Some(map) = self.selected_map() {
            self.edit.show_modal(ModalState::edit_map(map));
        }
    }

    pub fn show_data_loss_modal(&mut self, operation: DataLossOperation) {
        self.edit.show_modal(ModalState::dataloss(operation));
    }

    pub fn show_current_palette_modal(&mut self) {
        self.show_palette_modal(self.state.resources.palette().clone());
    }

    pub fn show_palette_modal(&mut self, palette: Palette) {
        self.edit.show_modal(ModalState::palette(palette));
    }

    pub fn show_error_modal(&mut self, message: &str) {
        self.edit.show_modal(ModalState::error(message));
    }

    pub fn show_import_tiled_modal(&mut self, tiled: Tiled) {
        self.edit.show_modal(ModalState::tiled(tiled));
    }

    pub fn show_export_tiled_modal(&mut self) {
        self.edit.show_modal(ModalState::export_tiled());
    }

    pub fn show_export_raw_modal(&mut self) {
        self.edit.show_modal(ModalState::export_raw());
    }

    pub fn show_export_png_modal(&mut self) {
        self.edit.show_modal(ModalState::export_png());
    }

    /// Progress the state of a modal, based on it's [`ModalState::result`]:
    ///  - [`ModalResult::Init`]: Move the result on to [`ModalResult::Active`],
    ///    return [`None`]
    ///  - [`ModalResult::Active`]: Nothing to do, return [`None`]
    ///  - [`ModalResult::Apply`]: hide the modal and return the [`Some<ModalState>`]
    ///    with the old modal state so it can be executed.
    ///  - [`ModalResult::Cancel`] hide the modal and return [`None`]
    pub fn progress_modal_state(&mut self) -> Option<ModalState> {
        self.edit.modal.result().and_then(|r| match r {
            ModalResult::Init => {
                self.edit.modal.make_active();
                None
            }
            ModalResult::Active => None,
            ModalResult::Apply => Some(self.edit.hide_modal()),
            ModalResult::Cancel => {
                self.edit.hide_modal();
                None
            }
        })
    }
}
