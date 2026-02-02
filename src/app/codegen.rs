use std::{
    fs::File,
    io::{BufWriter, Write},
};

use camino::Utf8PathBuf;
use eyre::bail;

use crate::{
    app::{maps::MapEditing, App},
    data::tiles::{tile_color::TileColor, Tile, Tiles},
    ui::file_dialog,
    utils::path_with_suffix_and_extension,
};

impl App {
    pub fn show_export_codegen_file_modal(&mut self) {
        let self_path = self.save_path.clone();
        if let Some(me) = self.selected_map_editing_mut() {
            // Default codegen path to the same as the project itself, plus the map name, with rs extension
            let default_path = self_path.map(|path| {
                path_with_suffix_and_extension(
                    &path,
                    "map",
                    me.map.name().as_str(),
                    file_dialog::RS_EXTENSION,
                )
            });
            match file_dialog::save_rs_file(&default_path) {
                Ok(Some(path)) => {
                    if let Err(e) = Self::export_codegen(&me, path) {
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

    fn tile_option_to_u32(tile: &Option<Tile>) -> eyre::Result<u32> {
        if let Some(tile) = tile {
            let palette_index = match tile.color {
                TileColor::Default => 0,
                TileColor::Palette { index } if index.index() <= 255 => index.index() & 0xff,
                TileColor::Palette { .. } => {
                    bail!("Export to rust codegen only supports palette indices <= 255")
                }
                TileColor::UserColor(_user_color) => {
                    bail!("Export to rust codegen only supports default and palette colors")
                }
            };

            // Use a Tiled-style tile index, with a fixed first gid of 1 so we
            // collapse all tilesets into one. This leaves 0 free for empty tiles.
            let tiled_tile_index = tile.source.tile_index.index() + 1;

            // We will include the palette index using bits 16 to 24 (0-based) so
            // we can only support tiled_tile_index <= 65535, from an internal
            // tile index <= 65534
            if tiled_tile_index > 65535 {
                bail!("Export to rust codegen only supports tile indices <= 65534")
            }

            // Use a u32 format based on Tiled, but with the palette index included
            // TODO: we collapse all tilesets into one, and use first gid of 1,
            // we could support multiple tilesets
            let tile_and_palette_index = tiled_tile_index
                | ((palette_index as u32) << 16)
                | tile.transform.as_tiled_flip_bits();

            Ok(tile_and_palette_index)
        } else {
            // Empty tile is 0 as in Tiled
            Ok(0)
        }
    }

    fn export_codegen(me: &MapEditing<'_>, path: Utf8PathBuf) -> eyre::Result<()> {
        let tiles = &me.map.tiles;

        if tiles.layer_count() != 1 {
            bail!("Export to rust codegen only supports a single layer");
        }

        if let Some(layer) = tiles.first_layer() {
            let combined = layer
                .tiles_iter()
                .map(Self::tile_option_to_u32)
                .collect::<Result<Vec<_>, _>>()?;

            let len = combined.len();
            let width = tiles.map_size().w;

            let mut f = BufWriter::new(File::create(path)?);
            writeln!(f, "use crate::tile::Tile;")?;
            writeln!(f)?;
            writeln!(f, "pub const TILES_WIDTH: u32 = {};", width)?;
            writeln!(f, "pub const TILES: [Tile; {}] = [", len)?;

            for tile in combined.iter() {
                writeln!(f, "\tTile::raw({}),", tile)?;
            }

            writeln!(f, "];")?;
            writeln!(f)?;

            // Flush to detect any errors
            f.flush()?;

            Ok(())
        } else {
            bail!("Export to rust codegen requires a layer");
        }
    }
}
