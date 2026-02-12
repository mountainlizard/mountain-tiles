use std::fs::File;
use std::io::{BufWriter, Write};

use crate::{
    app::{maps::MapEditing, App},
    data::{
        raw::RawExportSettings,
        tiles::{layer_tiles::Layer, tile_color::TileColor, Tile, Tiles},
        tilesets::{TilesetId, Tilesets},
    },
    ui::file_dialog,
    utils::path_with_suffix_and_extension,
};
use camino::Utf8PathBuf;
use egui::ahash::{HashMap, HashMapExt};
use eyre::{bail, eyre};

fn tile_option_to_u32(
    tile: &Option<Tile>,
    firstgids: &HashMap<TilesetId, u32>,
) -> eyre::Result<u32> {
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

        let firstgid = firstgids
            .get(&tile.source.tileset_id)
            .ok_or(eyre!("Missing gid for tileset id"))?;

        // Flatten the tile index, to work as one tileset containing tiles from all
        // input tilesets, starting from index 1 so we can use 0 for an empty tile,
        // as in Tiled
        let flat_tile_index = tile.source.tile_index.index() + *firstgid;

        // We will include the palette index using bits 16 to 24 (0-based) so
        // we can only support flat_tile_index <= 65535, from an internal
        // tile index <= 65534
        if flat_tile_index > 65535 {
            bail!("Export to rust codegen only supports tile indices <= 65534")
        }

        // Use a u32 format based on Tiled, but with the palette index included
        let tile_and_palette_index =
            flat_tile_index | ((palette_index as u32) << 16) | tile.transform.as_tiled_flip_bits();

        Ok(tile_and_palette_index)
    } else {
        // Empty tile is 0 as in Tiled
        Ok(0)
    }
}

fn layer_to_raw(layer: &Layer, tilesets: &Tilesets) -> eyre::Result<Vec<u32>> {
    // Build tileset first gids
    let mut firstgid = 1;
    let mut firstgids = HashMap::new();

    for tileset in tilesets.iter() {
        firstgids.insert(tileset.id(), firstgid);
        firstgid += tileset.size_in_tiles.area();
    }

    let combined = layer
        .tiles_iter()
        .map(|t| tile_option_to_u32(t, &firstgids))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(combined)
}

impl App {
    pub fn show_export_raw_file_modal(&mut self, settings: &RawExportSettings) {
        // let self_path = self.save_path.clone();
        // // Default dir to the same as the project itself
        // match file_dialog::pick_folder_with_default(&self_path) {
        //     Ok(Some(path)) => {
        //         if let Err(e) = self.export_raw(path, settings) {
        //             self.show_error_modal(&e.to_string());
        //         }
        //     }
        //     Ok(None) => {}
        //     Err(e) => self.show_error_modal(&e.to_string()),
        // }

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
                    if let Err(e) = Self::export_map_raw(&me, path, settings) {
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

    fn export_map_raw(
        me: &MapEditing<'_>,
        path: Utf8PathBuf,
        _settings: &RawExportSettings,
    ) -> eyre::Result<()> {
        let tiles = &me.map.tiles;
        let tilesets = &me.resources.tilesets;

        if tiles.layer_count() != 1 {
            bail!("Export to rust codegen only supports a single layer");
        }

        if let Some(layer) = tiles.first_layer() {
            let combined = layer_to_raw(layer, tilesets)?;

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
            bail!("Export to raw requires a single layer");
        }
    }
}
