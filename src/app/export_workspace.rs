use std::fs::{self, File};
use std::io::{BufWriter, Write};

use crate::data::config::workspace::{Export, Project, Workspace};
use crate::data::png::PngExportSettings;
use crate::data::tiles::layer_tiles::LayerTiles;
use crate::data::tiles::tileset_stacked_tiles::TilesetStackedTiles;
use crate::render::render_tiles;
use crate::{
    app::App,
    data::{
        tiles::{layer_tiles::Layer, tile_color::TileColor, Tile, Tiles},
        tilesets::{TilesetId, Tilesets},
    },
};
use bitbuffer::{BigEndian, BitWriteStream};
use camino::Utf8PathBuf;
use convert_case::ccase;
use egui::ahash::{HashMap, HashMapExt};
use eyre::{bail, eyre};
use image::{ImageBuffer, Rgba};

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

// Convert an image to 1-bit (i.e. binary, black and white).
// This uses 1 bit per pixel, packed into bytes in a big endian order. Pixels are
// encoded in the normal "reading" order starting at top-right and running along rows.
// At the end of each row, we pad any partial byte with zeroes to align to the next byte,
// so rows always start at the start of a byte.
// This should match with the format expected by ImageMagick using:
// convert -depth 1 -size WxH+0 gray:in.raw out.png
// Where "W" and "H" are the width and height of the image as integers.
// Output pixel bits are set to 1 if the input pixel has any color channel above 128, and
// alpha above 128. Most inputs are expected to be either pure black and white with alpha
// 255 everywhere, or white with alpha 255, and "black" pixels with alpha 0 and arbitrary color.
fn image_to_raw_1bit(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> eyre::Result<Vec<u8>> {
    let mut write_bytes = vec![];
    let mut write_stream = BitWriteStream::new(&mut write_bytes, BigEndian);

    for y in 0..image.height() {
        for x in 0..image.width() {
            let p = image.get_pixel(x, y).0;
            let bit = (p[0] > 128 || p[1] > 128 || p[2] > 128) && p[3] > 128;
            write_stream.write_bool(bit)?;
        }
        write_stream.align();
    }

    Ok(write_bytes)
}

fn save_image_as_raw_1bit(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    path: Utf8PathBuf,
) -> eyre::Result<()> {
    let raw_1bit = image_to_raw_1bit(image)
        .map_err(|e| eyre!("Failed to convert image to raw 1bit: {}", e))?;
    fs::write(path.clone(), raw_1bit).map_err(|e| {
        eyre!(
            "Failed to write raw 1bit image to:\n\n{}\n\nError:\n{}",
            path,
            e
        )
    })?;
    Ok(())
}

fn save_image_as_png(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    path: Utf8PathBuf,
) -> eyre::Result<()> {
    image.save(path.clone()).map_err(|e| {
                    eyre!(
                        "Failed to save tileset png to:\n\n{}:\n\nError:\n{}\n\nPlease check the relevant directory exists.",
                        path,
                        e
                    )
                })?;
    Ok(())
}

impl App {
    fn export_tileset(&self, self_dir: &Utf8PathBuf, export: &Export) -> eyre::Result<()> {
        if export.exports_tileset() {
            let tileset_image = self.tilesets_to_image()?;

            if let Some(rel_path) = &export.tileset_1bit_path {
                let mut path = self_dir.clone();
                path.push(rel_path);
                save_image_as_raw_1bit(&tileset_image, path)?;
            }

            if let Some(rel_path) = &export.tileset_png_path {
                let mut path = self_dir.clone();
                path.push(rel_path);
                save_image_as_png(&tileset_image, path)?;
            }
        }

        Ok(())
    }

    fn export_palette(&self, self_dir: &Utf8PathBuf, export: &Export) -> eyre::Result<()> {
        if let Some(rel_path) = &export.palette_json_path {
            let mut path = self_dir.clone();
            path.push(rel_path);

            self.state
                .resources
                .palette()
                .write_to_json_by_path(path.clone())
                .map_err(|e| {
                    eyre!(
                        "Failed to write JSON palette data to:\n\n{}\n\nError:\n{}",
                        path,
                        e
                    )
                })?;
        }

        if let Some(rel_path) = &export.palette_image_path {
            let mut path = self_dir.clone();
            path.push(rel_path);

            self.state
                .resources
                .palette()
                .write_to_image_by_path(path.clone())
                .map_err(|e| {
                    eyre!(
                        "Failed to write palette as image to:\n\n{}\n\nError:\n{}",
                        path,
                        e
                    )
                })?;
        }

        Ok(())
    }

    fn export_module(&self, self_dir: &Utf8PathBuf, export: &Export) -> eyre::Result<()> {
        if let Some(module_path) = &export.module_path {
            let mut path = self_dir.clone();
            path.push(module_path);

            let mut f = BufWriter::new(
                File::create(path.clone())
                    .map_err(|e| eyre!("Failed to open module file '{}': {}", path, e))?,
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
        Ok(())
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

        if let Some(export) = project.export.as_ref() {
            let mut self_dir = self_path.clone();
            self_dir.pop();

            self.export_tileset(&self_dir, export)?;
            self.export_palette(&self_dir, export)?;
            self.export_module(&self_dir, export)?;
        }

        Ok(())
    }

    pub fn export_from_workspace(&mut self) {
        if let Err(e) = self.export_from_workspace_error() {
            self.show_error_modal(&e.to_string());
        } else {
            self.success("Exported workspace");
        }
    }

    fn tilesets_to_image(&self) -> eyre::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let tiles = &TilesetStackedTiles::new(&self.state.resources.tilesets);
        let palette = &self.state.resources.palette;
        let tilesets = &self.state.resources.tilesets;
        let textures = &self.textures;
        let settings = &PngExportSettings {
            scale: 1,
            transparent: true,
        };
        let image = render_tiles(tiles, palette, tilesets, textures, settings)?;
        Ok(image)
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
                "    pub const MAP: MapData<{}, {}> = MapData {{",
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
