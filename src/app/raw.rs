use std::fs::File;
use std::io::{BufWriter, Write};

use crate::data::config::workspace::{Project, Workspace};
use crate::data::png::PngExportSettings;
use crate::data::tiles::layer_tiles::LayerTiles;
use crate::data::tiles::tileset_stacked_tiles::TilesetStackedTiles;
use crate::render::render_tiles;
use crate::ui::file_dialog::PNG_EXTENSION;
use crate::{
    app::App,
    data::{
        raw::RawExportSettings,
        tiles::{layer_tiles::Layer, tile_color::TileColor, Tile, Tiles},
        tilesets::{TilesetId, Tilesets},
    },
    ui::file_dialog,
    utils::path_with_suffix_and_extension,
};
use camino::Utf8PathBuf;
use convert_case::ccase;
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
    pub fn show_export_selected_map_raw_file_modal(&mut self, settings: &RawExportSettings) {
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
                    if let Err(e) = Self::export_map_module_to_file(
                        &me.map.name,
                        &me.map.tiles,
                        &me.resources.tilesets,
                        path.clone(),
                    ) {
                        self.show_error_modal(&e.to_string());
                        return;
                    }
                    if settings.export_combined_png_tileset {
                        let mut png_path = path.clone();
                        png_path.set_extension(file_dialog::PNG_EXTENSION);
                        if let Err(e) = self.export_tilesets_png(png_path) {
                            self.show_error_modal(
                                format!("Error exporting png:\n{}", &e.to_string()).as_str(),
                            );
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => self.show_error_modal(&e.to_string()),
            }
        } else {
            self.show_error_modal("No map selected to export");
        }
    }

    pub fn export_from_workspace_error(&mut self) -> eyre::Result<()> {
        let self_path = self
            .save_path
            .as_ref()
            .ok_or(eyre!("Please save the project before exporting."))?;

        let project = Project::from_project_path(self_path.clone())?;

        if !project.export_has_effect() {
            let workspace_path = Workspace::workspace_path_from_project_path(self_path.clone())?;
            bail!("Export settings do not export any files\nAdd some settings to workspace file at:\n{}\nSee example data for supported settings.", workspace_path);
        }

        if let Some(module_path) = project.export.as_ref().and_then(|e| e.module_path.clone()) {
            let mut module_file = self_path.clone();
            module_file.pop();
            module_file.push(module_path);
            let mut f = BufWriter::new(
                File::create(module_file.clone())
                    .map_err(|e| eyre!("Failed to open module file '{}': {}", module_file, e))?,
            );

            // TODO: Keep map of name to count, use to append numbers on duplicate names

            for map in self.state.maps.iter() {
                Self::export_map_module(
                    &map.name,
                    &map.tiles,
                    &self.state.resources.tilesets,
                    &mut f,
                )?;
            }

            f.flush()?;
        }

        if let Some(relative_tileset_path) =
            project.export.as_ref().and_then(|e| e.tileset_path.clone())
        {
            let mut tileset_path = self_path.clone();
            tileset_path.pop();
            tileset_path.push(relative_tileset_path);

            if project.export_tileset_png() {
                let mut png_path = tileset_path.clone();
                png_path.set_extension(PNG_EXTENSION);
                self.export_tilesets_png(png_path)?;
            }
        }

        Ok(())
    }

    pub fn export_from_workspace(&mut self) {
        if let Err(e) = self.export_from_workspace_error() {
            self.show_error_modal(&e.to_string());
        }
    }

    pub fn show_export_project_raw_file_modal(&mut self, _settings: &RawExportSettings) {
        println!("TODO: Export project raw");
        // if let Some(self_path) = self.save_path.clone() {
        // } else {
        //     self.show_error_modal("Please save the project before exporting.");
        // }
        // // Default dir to the same as the project itself
        // match file_dialog::pick_folder_with_default(&self_path) {
        //     Ok(Some(path)) => {
        //         // for map in self.state.maps.iter() {
        //         //     if let Err(e) = Self::export_map_raw(
        //         //         &map.tiles,
        //         //         &self.state.resources.tilesets,
        //         //         path.clone(),
        //         //     ) {
        //         //         self.show_error_modal(
        //         //             format!("Error exporting map '{}':\n{}", map.name(), &e.to_string())
        //         //                 .as_str(),
        //         //         );
        //         //     }
        //         // }
        //         // if let Err(e) = self.export_raw(path, settings) {
        //         //     self.show_error_modal(&e.to_string());
        //         // }
        //         println!("TODO: Export project raw to {:?}", path);
        //     }
        //     Ok(None) => {}
        //     Err(e) => self.show_error_modal(&e.to_string()),
        // }
    }

    pub fn show_export_raw_file_modal(&mut self, settings: &RawExportSettings) {
        if settings.export_project_to_rust {
            self.show_export_project_raw_file_modal(settings);
        } else {
            self.show_export_selected_map_raw_file_modal(settings);
        }
    }

    fn export_tilesets_png(&self, path: Utf8PathBuf) -> eyre::Result<()> {
        let tiles = &TilesetStackedTiles::new(&self.state.resources.tilesets);
        let palette = &self.state.resources.palette;
        let tilesets = &self.state.resources.tilesets;
        let textures = &self.textures;
        let settings = &PngExportSettings {
            scale: 1,
            transparent: true,
        };
        let image = render_tiles(tiles, palette, tilesets, textures, settings)?;
        image.save(path)?;

        Ok(())
    }

    fn export_map_module_to_file(
        module_name: &str,
        tiles: &LayerTiles,
        tilesets: &Tilesets,
        path: Utf8PathBuf,
    ) -> eyre::Result<()> {
        let mut f = BufWriter::new(File::create(path)?);
        Self::export_map_module(module_name, tiles, tilesets, &mut f)?;
        Ok(())
    }

    fn export_map_module<W: Write>(
        map_name: &str,
        tiles: &LayerTiles,
        tilesets: &Tilesets,
        f: &mut W,
    ) -> eyre::Result<()> {
        // If there are no layers, there's nothing to export
        if let Some(layer_tile_count) = tiles.first_layer().map(|layer| layer.tiles_iter().len()) {
            writeln!(f, "pub mod {} {{", ccase!(snake, map_name))?;

            writeln!(f, "    pub mod layers {{")?;
            writeln!(f)?;
            writeln!(f, "        use embedded_graphics_core::prelude::Size;")?;
            writeln!(f, "        use tili::tile::{{Tile, LayerData}};")?;
            writeln!(f)?;

            for layer in tiles.layers() {
                let combined = layer_to_raw(layer, tilesets)?;

                writeln!(
                    f,
                    "        pub const {}: LayerData<{}> = LayerData {{",
                    ccase!(constant, layer.name()),
                    layer_tile_count,
                )?;

                writeln!(f, "            name: \"{}\",", layer.name())?;
                writeln!(f, "            visible: {},", layer.visible())?;
                writeln!(
                    f,
                    "            size: Size::new({}, {}),",
                    tiles.map_size().w,
                    tiles.map_size().h
                )?;

                writeln!(f, "            tiles: [")?;

                for tile in combined.iter() {
                    writeln!(f, "                Tile::raw({}),", tile)?;
                }

                writeln!(f, "            ],")?;
                writeln!(f, "            opacity: {:?},", layer.opacity())?;
                writeln!(f, "        }};")?;
            }

            writeln!(f, "    }}")?;
            writeln!(f)?;
            writeln!(f, "    use embedded_graphics_core::prelude::Size;")?;
            writeln!(f, "    use tili::tile::MapData;")?;
            writeln!(f)?;

            writeln!(
                f,
                "    pub const map: MapData<{}, {}> = MapData {{",
                tiles.layer_count(),
                layer_tile_count
            )?;
            writeln!(f, "        name: \"{}\",", map_name)?;
            writeln!(
                f,
                "        size: Size::new({}, {}),",
                tiles.map_size().w,
                tiles.map_size().h
            )?;

            writeln!(f, "        layers: [")?;
            for layer in tiles.layers() {
                writeln!(f, "            &layers::{}", ccase!(constant, layer.name()))?;
            }
            writeln!(f, "        ]")?;

            writeln!(f, "    }};")?;

            writeln!(f, "}}")?;

            // Flush to detect any errors
            f.flush()?;
        }

        Ok(())
    }
}
