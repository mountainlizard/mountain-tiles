use std::{fs::File, io::BufReader};

use camino::Utf8PathBuf;
use egui::ahash::{HashMap, HashMapExt};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    data::palette::{palette_index, Palette},
    data::tiles::{
        layer_tiles::LayerTiles,
        tile_color::{TileColor, UserColor},
        Tile, TileIndex, TileSource, Tiles,
    },
    data::{
        maps::{Map, MapId},
        state::State,
    },
    geom::{transform::Transform, u32pos2::u32pos2, u32size2::u32size2},
    tiled::{
        tiled_color::TiledColor,
        tiled_xml::{TiledXml, TilesetXml},
    },
    utils,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TiledTileset {
    pub firstgid: u32,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tiled {
    pub backgroundcolor: Option<TiledColor>,
    // pub compressionlevel: i32,
    pub height: u32,
    pub width: u32,
    pub infinite: bool,
    pub layers: Vec<TiledLayer>,
    pub tileheight: u32,
    pub tilewidth: u32,
    pub tilesets: Vec<TiledTileset>,

    #[serde(skip)]
    #[serde(default)]
    pub xml_tilesets: Vec<TilesetXml>,
    /// The path to the file this data was loaded
    /// from, if known.
    #[serde(skip)]
    #[serde(default)]
    pub file_path: Option<Utf8PathBuf>,
    // TODO: Add remaining fields, and validate on input - error if
    // we get a value we can't import / don't understand
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TiledLayer {
    pub data: Vec<u32>,
    pub height: u32,
    pub width: u32,
    pub id: u32,
    pub name: String,
    pub opacity: f32,
    pub tintcolor: Option<TiledColor>,
    // TODO: We should parse this, and ignore layers that aren't tiles
    // pub r#type: String,
    pub visible: bool,
    // pub x: u32,
    // pub y: u32,
}

struct ColorCache {
    color_to_index: HashMap<UserColor, u32>,
    colors: Vec<UserColor>,
}

impl ColorCache {
    fn new() -> Self {
        Self {
            color_to_index: HashMap::new(),
            colors: vec![],
        }
    }
    fn insert_tintcolor(&mut self, tintcolor: &TiledColor) -> TileColor {
        let user_color: UserColor = tintcolor.into();
        self.insert_user_color(user_color)
    }

    fn insert_user_color(&mut self, user_color: UserColor) -> TileColor {
        if let Some(index) = self.color_to_index.get(&user_color) {
            TileColor::from_palette_index(palette_index(*index))
        } else {
            let new_index = self.colors.len() as u32;
            self.colors.push(user_color);
            self.color_to_index.insert(user_color, new_index);
            TileColor::from_palette_index(palette_index(new_index))
        }
    }

    fn from_palette(palette: &Palette) -> Self {
        let mut cache = Self::new();
        for user_color in palette.colors() {
            cache.insert_user_color(*user_color);
        }
        cache
    }
}

fn convert_background_color(color: &Option<TiledColor>) -> UserColor {
    if let Some(color) = color {
        color.into()
    } else {
        UserColor::TRANSPARENT
    }
}

impl Tiled {
    // Tiled uses an offset for tiles in each tileset
    // To support multiple tilesets, we need to find which tileset Tiled index
    // is in by finding greatest `tileset.firstgid` that is <= tiled_tile_index,
    // and subtracting that firstgid
    fn tiled_tile_index_to_tileset_index(
        &self,
        tiled_tile_index: u32,
    ) -> eyre::Result<(usize, u32)> {
        if let Some((index, tiled_tileset)) = self
            .tilesets
            .iter()
            .enumerate()
            .rev()
            .find(|&(_index, tiled_tileset)| tiled_tileset.firstgid <= tiled_tile_index)
        {
            Ok((index, tiled_tile_index - tiled_tileset.firstgid))
        } else {
            eyre::bail!(
                "Can't find tileset for Tiled tile index {}",
                tiled_tile_index
            )
        }
    }

    /// Append Tiled data to a [`State`], reusing matching resources (tilesets and palette colors)
    /// where possible, adding them where they are not already present.
    /// See https://doc.mapeditor.org/en/stable/reference/global-tile-ids/
    pub fn append_to_state(
        &self,
        state: &mut State,
        squash_layers: bool,
        prefer_relative_path: bool,
    ) -> eyre::Result<MapId> {
        let background = convert_background_color(&self.backgroundcolor);
        let tile_size = u32size2(self.tilewidth, self.tileheight);
        let map_size = u32size2(self.width, self.height);
        let mut tiles = LayerTiles::new(background, tile_size, map_size);

        // Track the tint colors we've seen, as user colors, starting
        // from the initial palette
        let mut color_cache = ColorCache::from_palette(state.resources.palette());

        // Convert xml tilesets to our format, and find/add to tilesets, keeping a map from
        // tiled tileset index to tileset id.
        let tilesets = &mut state.resources.tilesets;
        let mut tiled_tileset_index_to_tileset_id = HashMap::new();
        for (index, xml_tileset) in self.xml_tilesets.iter().enumerate() {
            let tileset = xml_tileset.as_tileset(prefer_relative_path, self.file_path.clone())?;
            let tileset_id = tilesets.find_or_insert_matching_tileset(tileset);
            tiled_tileset_index_to_tileset_id.insert(index, tileset_id);
        }

        for layer in self.layers.iter().rev() {
            let layer_index = tiles.layer_count();
            if layer.width != map_size.w || layer.height != map_size.h {
                return Err(eyre!(
                    "Tiled maps must have all layers the same size as map"
                ));
            }
            let opacity = if layer.opacity >= 1.0 {
                None
            } else {
                Some(layer.opacity.clamp(0.0, 1.0))
            };
            tiles.insert_layer(layer_index, &layer.name, layer.visible, opacity);

            let color = if let Some(ref tintcolor) = layer.tintcolor {
                color_cache.insert_tintcolor(tintcolor)
            } else {
                // There's no tint color, so use default tile color (no recoloring of tiles)
                TileColor::Default
            };

            for y in 0..map_size.h {
                for x in 0..map_size.w {
                    let pos = u32pos2(x, y);
                    let tiled_tile = *layer
                        .data
                        .get((x + y * layer.width) as usize)
                        .ok_or(eyre!("Missing data in tile array"))?;

                    // Ignore the top 4 bits (flip bits) to get in Tiled version of tile index
                    let tiled_tile_index = tiled_tile & 0xFFFFFFF;

                    // In Tiled, 0 indicates an empty tile, so just ignore
                    if tiled_tile_index != 0 {
                        // Find the index of the tiled tileset we're using, and
                        // the 0-based tile index within that tileset
                        let (tiled_tileset_index, tile_index) =
                            self.tiled_tile_index_to_tileset_index(tiled_tile_index)?;

                        let tileset_id = tiled_tileset_index_to_tileset_id
                            .get(&tiled_tileset_index)
                            .ok_or(eyre!(
                                "Cannot find tileset for Tiled index {}",
                                tiled_tileset_index
                            ))?;
                        let source = TileSource {
                            tileset_id: *tileset_id,
                            tile_index: TileIndex::new(tile_index),
                        };

                        // The top 3 bits are used for our transform
                        let transform = Transform::from_tiled_flip_bits(tiled_tile);

                        let tile = Tile {
                            source,
                            color,
                            transform,
                        };
                        tiles.set_tile(layer_index, pos, Some(tile));
                    }
                }
            }
        }

        if squash_layers {
            for y in 0..map_size.h {
                for x in 0..map_size.w {
                    let pos = u32pos2(x, y);
                    for layer_index in 0..tiles.layer_count() {
                        if let Some(tile) = tiles.tile(layer_index, pos) {
                            tiles.set_tile(0, pos, Some(tile));
                            break;
                        }
                    }
                }
            }
            for _layer_index in 1..tiles.layer_count() {
                tiles.remove_layer(1);
            }
        }

        let palette = Palette::new(color_cache.colors);

        state.resources.palette = palette;
        let mut map = Map::default();
        // TODO: Better name?
        map.name = "Tiled import".to_string();
        map.tiles = tiles;

        let map_id = state.maps.push_map(map);

        Ok(map_id)
    }

    /// Convert Tiled data to [`State`], with map tiles and palette
    /// See https://doc.mapeditor.org/en/stable/reference/global-tile-ids/
    pub fn to_state(&self, squash_layers: bool, prefer_relative_path: bool) -> eyre::Result<State> {
        let mut state = State::default();
        self.append_to_state(&mut state, squash_layers, prefer_relative_path)?;
        Ok(state)
    }

    fn import_xml_tilesets(&mut self, tmx_path: Utf8PathBuf) -> eyre::Result<()> {
        let tmx_dir = utils::tmx_parent_dir(&tmx_path)?;

        let mut xml_tilesets = vec![];
        for tileset_ref in self.tilesets.iter() {
            let tmx_to_tsx_file_path = Utf8PathBuf::from(tileset_ref.source.clone());

            // Make tileset path relative to tmx dir (if needed - if tileset path is relative,
            // it will be appended to base path, if it's absolute it will replace it)
            let mut tileset_path = Utf8PathBuf::new();
            tileset_path.push(tmx_dir);
            tileset_path.push(tmx_to_tsx_file_path.clone());

            let mut tileset_xml = TilesetXml::from_path_xml(tileset_path)?;
            tileset_xml.tmx_to_tsx_file_path = Some(tmx_to_tsx_file_path);
            xml_tilesets.push(tileset_xml);
        }

        self.xml_tilesets = xml_tilesets;

        Ok(())
    }

    pub fn from_path_json(path: Utf8PathBuf) -> eyre::Result<Tiled> {
        let file = File::open(path.clone())?;
        let buf_reader = BufReader::new(file);
        let mut tiled: Tiled = serde_json::from_reader(buf_reader)?;

        tiled.import_xml_tilesets(path.clone())?;
        tiled.file_path = Some(path);

        Ok(tiled)
    }

    pub fn from_path_xml(path: Utf8PathBuf) -> eyre::Result<Tiled> {
        let tiled_xml = TiledXml::from_path_xml(path)?;
        tiled_xml.into_tiled()
    }

    pub fn from_path(path: Utf8PathBuf) -> eyre::Result<Tiled> {
        match path.extension() {
            Some(extension) => match extension.to_ascii_lowercase().as_str() {
                "tmx" | "xml" => Self::from_path_xml(path),
                "tmj" | "json" => Self::from_path_json(path),
                extension => eyre::bail!(
                    "Tiled extension '.{}' unknown (.tmx, .xml, .tmj and .json supported)",
                    extension
                ),
            },
            None => {
                eyre::bail!("Tiled file has no extension (.tmx, .xml, .tmj and .json supported)")
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn import_json() -> eyre::Result<()> {
        let _tiled = Tiled::from_path_json("test-data/tiled-zoo.tmj".into())?;
        Ok(())
    }

    #[test]
    fn relative_paths_should_work_as_expected() -> eyre::Result<()> {
        use camino::Utf8PathBuf;

        // Original location of the tmx file
        let tmx_path = Utf8PathBuf::from("/path1/path2/tiled_map.tmx");
        assert!(tmx_path.is_absolute());

        // Dir containing tmx file - just pop the file off the end
        let mut tmx_dir_path = tmx_path.clone();
        tmx_dir_path.pop();
        assert!(tmx_dir_path.is_absolute());

        // Relative path from tmx file to tsx file (points to `/path1/tiled_tileset.tsx`)
        let tsx_relative_path = Utf8PathBuf::from("../tiled_tileset.tsx");
        assert!(tsx_relative_path.is_relative());

        // Relative path from tsx file to image file (points to `/path1/path3/image.png`)
        let image_relative_path = Utf8PathBuf::from("path3/image.png");
        assert!(image_relative_path.is_relative());

        // Produce a relative path from tmx file to image file
        let mut tmx_to_image_relative_path = tsx_relative_path.clone();
        // Pop off the tsx filename
        tmx_to_image_relative_path.pop();
        assert!(tmx_to_image_relative_path.is_relative());
        // Add on the image relative path
        tmx_to_image_relative_path.push(image_relative_path);
        assert!(tmx_to_image_relative_path.is_relative());

        assert_eq!(
            Utf8PathBuf::from("../path3/image.png"),
            tmx_to_image_relative_path
        );

        // To load the image, we'd start from the tmx_path
        let mut image_absolute_path = tmx_dir_path.clone();
        image_absolute_path.push(tmx_to_image_relative_path);
        assert!(image_absolute_path.is_absolute());
        assert_eq!(
            Utf8PathBuf::from("/path1/path2/../path3/image.png"),
            image_absolute_path
        );

        Ok(())
    }
}
