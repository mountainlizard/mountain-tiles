use crate::{
    app::App,
    data::{
        action::Action,
        modal::DataLossOperation,
        palette::{Palette, PaletteIndex},
        tiles::tile_color::TileColor,
    },
    ui::file_dialog::{self, JSON_EXTENSION, JSON_NAME},
};

impl App {
    pub fn selected_palette_index(&self) -> PaletteIndex {
        self.edit
            .selected_palette_index()
            .limited_to_palette(self.state.resources.palette())
    }

    pub fn selected_palette_color(&self) -> TileColor {
        TileColor::from_palette_index(self.selected_palette_index())
    }

    pub fn select_palette_index(&mut self, i: PaletteIndex) {
        self.edit
            .select_palette_index(i.limited_to_palette(self.state.resources.palette()));
    }

    /// Replace palette without checking for affected tiles
    /// This clears any tiles that were using a palette index outside the new palette,
    /// and also clears tileset selections and stamp
    pub(super) fn replace_palette(&mut self, palette: Palette) -> bool {
        if self.state.resources.palette != palette {
            self.state.clear_tiles_outside_palette(&palette);
            self.state.resources.palette = palette;
            self.edit.clear_tileset_tile_selections_and_stamp();
            true
        } else {
            false
        }
    }

    pub fn previous_palette_index(&mut self) {
        let i = self.selected_palette_index();
        self.select_palette_index(i.previous_within_palette(self.state.resources.palette()));
    }

    pub fn next_palette_index(&mut self) {
        let i = self.selected_palette_index();
        self.select_palette_index(i.next_within_palette(self.state.resources.palette()));
    }

    /// Check whether palette has reduced count, if so prompt user to replace,
    /// otherwise just replace.
    pub fn prompt_to_replace_palette(&mut self, palette: Palette) {
        if let Some(resource_use) = self.state.find_use_of_colors_outside_palette(&palette) {
            self.show_data_loss_modal(DataLossOperation::ReplacePalette {
                palette,
                resource_use,
            });
        } else {
            self.act(Action::ReplacePalette { palette });
        }
    }

    pub fn show_import_palette_modal(&mut self) {
        match file_dialog::pick_file() {
            Ok(Some(path)) => match Palette::from_image_by_path(path) {
                Ok(palette) => self.show_palette_modal(palette),
                Err(e) => self.show_error_modal(&e.to_string()),
            },
            Ok(None) => {}
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }

    pub fn show_export_palette_modal(&mut self) {
        match file_dialog::save_png_file(&None) {
            Ok(Some(path)) => {
                if let Err(e) = self.state.resources.palette().write_to_image_by_path(path) {
                    self.show_error_modal(&e.to_string());
                }
            }
            Ok(None) => {}
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }

    pub fn show_import_palette_lospec_modal(&mut self) {
        match file_dialog::pick_file_with_extension(JSON_NAME, JSON_EXTENSION) {
            Ok(Some(path)) => match Palette::from_json_by_path(path) {
                Ok(palette) => self.show_palette_modal(palette),
                Err(e) => self.show_error_modal(&e.to_string()),
            },
            Ok(None) => {}
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }

    pub fn show_export_palette_lospec_modal(&mut self) {
        match file_dialog::save_json_file(&None) {
            Ok(Some(path)) => {
                if let Err(e) = self.state.resources.palette().write_to_json_by_path(path) {
                    self.show_error_modal(&e.to_string());
                }
            }
            Ok(None) => {}
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }
}
