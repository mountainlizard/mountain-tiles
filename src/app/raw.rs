use camino::Utf8PathBuf;

use crate::{app::App, data::raw::RawExportSettings, ui::file_dialog};

impl App {
    pub fn show_export_raw_file_modal(&mut self, settings: &RawExportSettings) {
        let self_path = self.save_path.clone();

        // Default dir to the same as the project itself
        match file_dialog::pick_folder_with_default(&self_path) {
            Ok(Some(path)) => {
                if let Err(e) = self.export_raw(path, settings) {
                    self.show_error_modal(&e.to_string());
                }
            }
            Ok(None) => {}
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }

    fn export_raw(&self, path: Utf8PathBuf, settings: &RawExportSettings) -> eyre::Result<()> {
        // let tiled = TiledXml::from_map_editing(me, path.clone(), settings)?;
        // tiled.save(path, settings)?;
        // todo!()
        println!("Saving to {:?} with settings {:?}", path, settings);
        Ok(())
    }
}
