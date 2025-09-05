use camino::Utf8PathBuf;

use crate::{
    app::{maps::MapEditing, App},
    data::png::PngExportSettings,
    render::render_tiles,
    ui::file_dialog,
    utils::path_with_suffix_and_extension,
};

impl App {
    pub fn show_export_png_file_modal(&mut self, settings: &PngExportSettings) {
        let self_path = self.save_path.clone();
        if let Some(me) = self.selected_map_editing_mut() {
            // Default save path to the same as the project itself, plus the map name, with png extension
            let default_path = self_path.map(|path| {
                path_with_suffix_and_extension(
                    &path,
                    "Map",
                    me.map.name().as_str(),
                    file_dialog::PNG_EXTENSION,
                )
            });

            // Default png path to the same as the map itself, with ".png" extension
            match file_dialog::save_png_file(&default_path) {
                Ok(Some(path)) => {
                    if let Err(e) = Self::export_png(&me, path, settings) {
                        self.show_error_modal(&e.to_string());
                    }
                }
                Ok(None) => {}
                Err(e) => self.show_error_modal(&e.to_string()),
            }
        } else {
            self.show_error_modal("No map selected to export to PNG.");
        }
    }

    fn export_png(
        me: &MapEditing<'_>,
        path: Utf8PathBuf,
        settings: &PngExportSettings,
    ) -> eyre::Result<()> {
        let image = render_tiles(
            me.map.tiles(),
            me.resources.palette(),
            me.resources.tilesets(),
            me.textures,
            settings,
        )?;
        image.save(path)?;
        Ok(())
    }
}
