use camino::Utf8PathBuf;

use crate::{
    app::{maps::MapEditing, App},
    data::{modal::DataLossOperation, tiled::TiledExportSettings},
    tiled::{tiled_json::Tiled, tiled_xml::TiledXml},
    ui::file_dialog,
    utils::path_with_suffix_and_extension,
};

impl App {
    pub fn check_data_loss_then_pick_tiled_file_to_import(&mut self) {
        if self.may_have_unsaved_changes() {
            self.show_data_loss_modal(DataLossOperation::ImportTiled);
        } else {
            self.pick_tiled_file_to_import();
        }
    }

    pub fn pick_tiled_file_to_import(&mut self) {
        match file_dialog::pick_file() {
            Ok(Some(path)) => match Tiled::from_path(path) {
                Ok(tiled) => self.show_import_tiled_modal(tiled),
                Err(e) => self.show_error_modal(&e.to_string()),
            },
            Ok(None) => {}
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }

    pub fn show_export_tiled_file_modal(&mut self, settings: &TiledExportSettings) {
        let self_path = self.save_path.clone();
        if let Some(me) = self.selected_map_editing_mut() {
            // Default tiled path to the same as the project itself, plus the map name, with tmx extension
            let default_path = self_path.map(|path| {
                path_with_suffix_and_extension(
                    &path,
                    "Map",
                    me.map.name().as_str(),
                    file_dialog::TMX_EXTENSION,
                )
            });
            match file_dialog::save_tmx_file(&default_path) {
                Ok(Some(path)) => {
                    if let Err(e) = Self::export_tiled(&me, path, settings) {
                        self.show_error_modal(&e.to_string());
                    }
                }
                Ok(None) => {}
                Err(e) => self.show_error_modal(&e.to_string()),
            }
        } else {
            self.show_error_modal("No map selected to export");
        }
    }

    fn export_tiled(
        me: &MapEditing<'_>,
        path: Utf8PathBuf,
        settings: &TiledExportSettings,
    ) -> eyre::Result<()> {
        let tiled = TiledXml::from_map_editing(me, path.clone(), settings)?;
        tiled.save(path, settings)?;
        Ok(())
    }
}
