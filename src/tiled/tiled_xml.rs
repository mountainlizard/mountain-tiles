use std::fmt::Display;
use std::io::BufWriter;
use std::{fs::File, io::BufReader, num::ParseIntError};

use crate::data::tilesets::TilesetMode;
use crate::{
    app::maps::MapEditing,
    data::palette::{Palette, PaletteIndex},
    data::tiled::TiledExportSettings,
    data::tiles::{layer_tiles::LayerTiles, tile_color::TileColor, Tiles},
    data::tilesets::Tileset,
    data::tilesets::Tilesets,
    geom::u32size2::u32size2,
    tiled::tiled_color::TiledColor,
    tiled::tiled_json::{Tiled, TiledLayer, TiledTileset},
    ui::file_dialog,
    utils,
};
use camino::Utf8PathBuf;
use egui::ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use eyre::{bail, eyre, Context};
use quick_xml::events::{BytesDecl, BytesStart, Event};
use quick_xml::Writer;
use serde::{Deserialize, Serialize};

pub const TSX_VERSION: &str = "1.10";
pub const TMX_VERSION: &str = "1.10";
pub const TILED_VERSION: &str = "1.11.0";
pub const INFINITE_FALSE: u32 = 0;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TilesetReferenceXml {
    #[serde(rename = "@firstgid")]
    firstgid: u32,
    #[serde(rename = "@source")]
    source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    #[serde(rename = "orthogonal")]
    Orthogonal,
    #[serde(rename = "isometric")]
    Isometric,
    #[serde(rename = "staggered")]
    Staggered,
    #[serde(rename = "hexagonal")]
    Hexagonal,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Orthogonal => write!(f, "orthogonal"),
            Self::Isometric => write!(f, "isometric"),
            Self::Staggered => write!(f, "staggered"),
            Self::Hexagonal => write!(f, "hexagonal"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum RenderOrder {
    #[serde(rename = "right-down")]
    RightDown,
    #[serde(rename = "right-up")]
    RightUp,
    #[serde(rename = "left-down")]
    LeftDown,
    #[serde(rename = "left-up")]
    LeftUp,
}

impl Display for RenderOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RightDown => write!(f, "right-down"),
            Self::RightUp => write!(f, "right-up"),
            Self::LeftDown => write!(f, "left-down"),
            Self::LeftUp => write!(f, "left-up"),
        }
    }
}

/// Structure of a Tiled XML file (.tmx)
/// Note this only uses filters seen/needed for example data, may be extended in future.
/// See https://doc.mapeditor.org/en/stable/reference/tmx-map-format/
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TiledXml {
    // Attributes...
    #[serde(rename = "@version")]
    pub version: String,

    #[serde(rename = "@tiledversion")]
    pub tiledversion: String,

    #[serde(rename = "@orientation")]
    pub orientation: Orientation,

    #[serde(rename = "@renderorder")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderorder: Option<RenderOrder>,

    #[serde(rename = "@nextlayerid")]
    pub nextlayerid: u32,

    #[serde(rename = "@nextobjectid")]
    pub nextobjectid: u32,

    #[serde(rename = "@backgroundcolor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backgroundcolor: Option<TiledColor>,

    #[serde(rename = "@width")]
    pub width: u32,

    #[serde(rename = "@height")]
    pub height: u32,

    /// 0 for non-infinite (default), 1 for infinite
    #[serde(rename = "@infinite")]
    pub infinite: u32,

    #[serde(rename = "@tilewidth")]
    pub tilewidth: u32,

    #[serde(rename = "@tileheight")]
    pub tileheight: u32,

    // Contents...

    // Tilesets referenced in the layer data - note that this MUST come before
    // layers in the XML, be careful if refactoring and changing order of fields.
    #[serde(default)]
    pub tileset: Vec<TilesetReferenceXml>,

    #[serde(default)]
    pub layer: Vec<LayerXml>,

    #[serde(skip)]
    #[serde(default)]
    pub xml_tilesets: Vec<TilesetXml>,

    /// The path to the file this data was loaded
    /// from, if known.
    #[serde(skip)]
    #[serde(default)]
    pub file_path: Option<Utf8PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LayerXml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<LayerPropertiesXml>,

    pub data: LayerDataXml,

    #[serde(rename = "@height")]
    pub height: u32,

    #[serde(rename = "@width")]
    pub width: u32,

    /// Layer id, valid ids start from 1
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@opacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(rename = "@tintcolor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tintcolor: Option<TiledColor>,
    // pub r#type: String,
    #[serde(rename = "@visible")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    // pub x: u32,
    // pub y: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LayerPropertiesXml {
    #[serde(default)]
    pub property: Vec<LayerPropertyXml>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LayerPropertyXml {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LayerDataXml {
    #[serde(rename = "@encoding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,

    #[serde(rename = "@compression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,

    #[serde(rename = "$value")]
    pub contents: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TilesetXml {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@tiledversion")]
    pub tiledversion: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@tileheight")]
    pub tileheight: u32,
    #[serde(rename = "@tilewidth")]
    pub tilewidth: u32,
    #[serde(rename = "@tilecount")]
    pub tilecount: u32,
    #[serde(rename = "@columns")]
    pub columns: u32,
    #[serde(rename = "@backgroundcolor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backgroundcolor: Option<TiledColor>,
    pub image: ImageXml,

    /// The path to the tsx file this data was loaded
    /// from, if known.
    /// This is used to convert the `image.source` path
    /// from one that may be relative to the tsx file, to
    /// one that is relative to the tmx file, since
    /// in this project we use image files directly relative
    /// to the project file.
    /// This should generally be the path as it was specified
    /// in the tmx file, so may be relative to the tmx file,
    /// or can be absolute if needed.
    #[serde(skip)]
    #[serde(default)]
    pub tmx_to_tsx_file_path: Option<Utf8PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ImageXml {
    #[serde(rename = "@source")]
    pub source: String,
    #[serde(rename = "@height")]
    pub height: u32,
    #[serde(rename = "@width")]
    pub width: u32,
    #[serde(rename = "@trans")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trans: Option<TiledColor>,
}

impl TiledXml {
    pub fn into_tiled(self) -> eyre::Result<Tiled> {
        if self.orientation != Orientation::Orthogonal {
            return Err(eyre!(
                "Unsupported Tiled map orientation '{}', only {} is supported.",
                self.orientation,
                Orientation::Orthogonal
            ));
        }

        if let Some(renderorder) = self.renderorder {
            if renderorder != RenderOrder::RightDown {
                return Err(eyre!(
                    "Unsupported Tiled map render order '{}', only {} is supported.",
                    renderorder,
                    RenderOrder::RightDown
                ));
            }
        }

        let mut layers = vec![];
        for layer_xml in self.layer.iter() {
            if layer_xml.data.encoding != Some("csv".to_string()) {
                if let Some(ref encoding) = layer_xml.data.encoding {
                    return Err(eyre!(
                        "Unsupported layer data encoding '{}', only 'csv' is supported.",
                        encoding
                    ));
                } else {
                    return Err(eyre!(
                        "Unsupported layer data encoding - none specified, implying XML Tile data, only 'csv' is supported."
                    ));
                }
            };

            if let Some(ref compression) = layer_xml.data.compression {
                return Err(eyre!(
                    "Unsupported layer data compression '{}', only uncompressed data is supported.",
                    compression
                ));
            };

            let data_r: Result<Vec<u32>, ParseIntError> = layer_xml
                .data
                .contents
                .split(",")
                .map(|s| s.trim().parse::<u32>())
                .collect();
            let data = data_r?;
            let layer = TiledLayer {
                data,
                height: layer_xml.height,
                width: layer_xml.width,
                id: layer_xml.id,
                name: layer_xml.name.clone(),
                opacity: layer_xml.opacity.unwrap_or(1.0),
                tintcolor: layer_xml.tintcolor,
                visible: layer_xml.visible.unwrap_or(true),
            };
            layers.push(layer);
        }

        let mut tilesets = vec![];
        for tileset_xml in self.tileset.iter() {
            let tileset = TiledTileset {
                firstgid: tileset_xml.firstgid,
                source: tileset_xml.source.clone(),
            };
            tilesets.push(tileset);
        }

        // For some reason, Tiled uses 1/0 in XML, and true/false in JSON
        let infinite = self.infinite == 1;
        let tiled = Tiled {
            backgroundcolor: self.backgroundcolor,
            height: self.height,
            width: self.width,
            infinite,
            layers,
            tileheight: self.tileheight,
            tilewidth: self.tilewidth,
            tilesets,
            xml_tilesets: self.xml_tilesets,
            file_path: self.file_path,
        };

        Ok(tiled)
    }

    fn import_xml_tilesets(&mut self, tmx_path: Utf8PathBuf) -> eyre::Result<()> {
        let tmx_dir = utils::tmx_parent_dir(&tmx_path)?;

        let mut xml_tilesets = vec![];
        for tileset_ref in self.tileset.iter() {
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

    pub fn from_path_xml(path: Utf8PathBuf) -> eyre::Result<TiledXml> {
        let file = File::open(path.clone())?;
        let buf_reader = BufReader::new(file);
        let mut tiled: TiledXml = quick_xml::de::from_reader(buf_reader)?;
        tiled.import_xml_tilesets(path.clone())?;
        tiled.file_path = Some(path);

        Ok(tiled)
    }

    pub fn from_map_parts(
        path: Utf8PathBuf,
        settings: &TiledExportSettings,
        tiles: &LayerTiles,
        palette: &Palette,
        tilesets: &Tilesets,
    ) -> eyre::Result<TiledXml> {
        let map_size = tiles.map_size();
        let tile_size = tiles.tile_size();

        let mut firstgid = 1;
        let mut tileset_refs = vec![];
        let mut firstgids = HashMap::new();
        let mut xml_tilesets = vec![];

        let mut tsx_file_names = HashSet::new();

        // Convert tileset
        for tileset in tilesets.iter() {
            // First the reference
            let source_base = sanitize_filename::sanitize(tileset.name.clone());
            let source = format!("{}.{}", source_base, file_dialog::TSX_EXTENSION);

            if !tsx_file_names.insert(source.clone()) {
                bail!("Cannot export map as Tiled .tmx format.\nThere is more than one tileset named '{}'\n To export as Tiled, edit tileset names to be unique.\nThis is to allow each tileset to be saved as 'name.tsx'.", tileset.name)
            }

            tileset_refs.push(TilesetReferenceXml {
                firstgid,
                source: source.to_string(),
            });
            firstgids.insert(tileset.id(), firstgid);
            firstgid += tileset.size_in_tiles.area();

            // Now everything for the .tsx file
            let name = tileset.name.clone();

            let pixel_size = tileset.size_in_tiles * tileset.tile_size;
            let image = ImageXml {
                source: tileset.path.to_string(),
                height: pixel_size.h,
                width: pixel_size.w,
                trans: None,
            };

            xml_tilesets.push(TilesetXml {
                version: TSX_VERSION.to_string(),
                tiledversion: TILED_VERSION.to_string(),
                name,
                tileheight: tileset.tile_size.h,
                tilewidth: tileset.tile_size.w,
                tilecount: tileset.size_in_tiles.area(),
                columns: tileset.size_in_tiles.w,
                backgroundcolor: tileset.background.map(|c| c.into()),
                image,
                // We will put the tsx file alongside the tmx, so the path is just the name of the tsx file
                tmx_to_tsx_file_path: Some(Utf8PathBuf::from(source)),
            });
        }

        // Convert layers
        let mut layers = vec![];
        // Note we can't use our own layer ids, since we need to split layers by color
        let mut layer_id: u32 = 1;
        for layer in tiles.layers().rev() {
            // Note we go through colors in reverse, so we end up with them in the right order in Tiled.
            // This looks better in tiled, and means the palette doesn't get reversed if we reimport
            for palette_index in (0..palette.len()).rev() {
                let color = TileColor::Palette {
                    index: PaletteIndex::new(palette_index),
                };
                let tile_data: Vec<_> = layer
                    .tiles_iter()
                    .map(|tile| match tile {
                        Some(tile) => {
                            if tile.color == color {
                                if let Some(firstgid) = firstgids.get(&tile.source.tileset_id) {
                                    (firstgid + tile.source.tile_index.index())
                                        | tile.transform.as_tiled_flip_bits()
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        }
                        None => 0,
                    })
                    .map(|i| i.to_string())
                    .collect();
                let contents = tile_data.join(", ");

                let data = LayerDataXml {
                    encoding: Some("csv".to_string()),
                    compression: None,
                    contents,
                };

                let name = format!("{}-c{}", layer.name(), palette_index);

                let properties = if settings.include_layer_data_as_properties {
                    Some(LayerPropertiesXml {
                        property: vec![
                            LayerPropertyXml {
                                name: "com.mountaintiles.layer_name".to_string(),
                                value: layer.name(),
                            },
                            LayerPropertyXml {
                                name: "com.mountaintiles.palette_index".to_string(),
                                value: format!("{}", palette_index),
                            },
                        ],
                    })
                } else {
                    None
                };

                layers.push(LayerXml {
                    properties,
                    data,
                    height: map_size.h,
                    width: map_size.w,
                    id: layer_id,
                    name,
                    opacity: layer.opacity(),
                    tintcolor: palette
                        .color_option(PaletteIndex::new(palette_index))
                        .map(|c| c.into()),
                    visible: Some(layer.visible()),
                });
                layer_id += 1;
            }
        }

        Ok(TiledXml {
            version: TMX_VERSION.to_string(),
            tiledversion: TILED_VERSION.to_string(),

            orientation: Orientation::Orthogonal,
            renderorder: Some(RenderOrder::RightDown),

            nextlayerid: layer_id,

            // Note we don't use objects, so the next valid id is 1 (ids start from 1)
            nextobjectid: 1,

            backgroundcolor: Some(tiles.background().into()),
            height: map_size.h,
            width: map_size.w,
            infinite: INFINITE_FALSE,
            layer: layers,
            tileheight: tile_size.h,
            tilewidth: tile_size.w,
            tileset: tileset_refs,
            xml_tilesets,
            file_path: Some(path),
        })
    }

    pub fn from_map_editing(
        me: &MapEditing<'_>,
        path: Utf8PathBuf,
        settings: &TiledExportSettings,
    ) -> eyre::Result<TiledXml> {
        Self::from_map_parts(
            path,
            settings,
            me.map.tiles(),
            me.resources.palette(),
            me.resources.tilesets(),
        )
    }

    pub fn save_tmx(&self, path: Utf8PathBuf) -> eyre::Result<()> {
        let file = File::create(path.clone())?;
        let buf_writer = BufWriter::new(file);
        let mut writer = Writer::new(buf_writer);
        writer.write_event(Event::Decl(BytesDecl::from_start(
            BytesStart::from_content("xml version=\"1.0\" encoding=\"UTF-8\"", 3),
        )))?;
        writer.write_serializable("map", &self)?;
        Ok(())
    }

    /// Save tsx files for this [`TiledXml`], as files in the same directory
    /// as the specified path used for the `.tmx` file.
    pub fn save_tsx_files(&self, path: Utf8PathBuf) -> eyre::Result<()> {
        let mut base_dir = path.clone();
        base_dir.pop();

        for xml_tileset in self.xml_tilesets.iter() {
            let mut tileset_path = base_dir.clone();
            let relative_path = xml_tileset.tmx_to_tsx_file_path.clone().ok_or(eyre!(
                "Tileset has missing path - this should not happen, please report a bug."
            ))?;
            tileset_path.push(&relative_path);
            xml_tileset.save(tileset_path)?;
        }

        Ok(())
    }

    pub fn save(&self, path: Utf8PathBuf, settings: &TiledExportSettings) -> eyre::Result<()> {
        self.save_tmx(path.clone())?;
        if settings.export_tsx_files {
            self.save_tsx_files(path)?;
        }
        Ok(())
    }
}

impl TilesetXml {
    /// Convert to a mountaintiles tileset.
    ///
    /// Note the handling around the path in the output [`Tileset`] is slightly complicated.
    /// 1. If the [`TilesetXml`] has an absolute path for the image source in the tsx file,
    ///    then this will just be used in the output tileset.
    /// 2. If the [`TilesetXml`] has a relative path, this is relative to the tsx file it was
    ///    loaded from. If we also have the relative path from the tmx file to the tsx file as part of
    ///    the [`TilesetXml`], then we will default to the relative path tmx -> tsx -> image, since
    ///    we expect that the resulting tileset will be saved in a file in the same directory
    ///    as the tmx file, and this will then work. When importing a tmx file to make a new
    ///    mountaintiles [`crate::data::state::State`], it's best to set the `save_path` of the
    ///    state to the same dir as the tmx file so that the relative path will work before the
    ///    state is saved to a file.
    /// 3. HOWEVER - if `tmx_file_path` is not None, and the tmx to tsx path is known, then we
    ///    will prepend the `tmx_file_path`. This is expected to be absolute, and so will make
    ///    the resulting path absolute. This should be preferred when we are attempting to append
    ///    the imported Tiled map to an existing [`crate::data::state::State`] where we can't
    ///    set the `save_path` since it might break existing relative tileset paths.
    ///
    /// Note that in all cases `prefer_relative_path` will be passed through to the tileset created,
    /// so even if an absolute path is generated for the tileset, if `prefer_relative_path` is set
    /// to true the system will attempt to convert it back to a relative path when the file is saved.
    pub fn as_tileset(
        &self,
        prefer_relative_path: bool,
        tmx_file_path: Option<Utf8PathBuf>,
    ) -> eyre::Result<Tileset> {
        let columns = self.columns;
        let rows = self.tilecount.div_ceil(columns);
        let background = self.backgroundcolor.as_ref().map(|c| c.into());

        // Since image path is relative to the tsx file, we need to find the tsx file's
        // own path - default to an empty path, but if we know the tsx file path, append
        // the directory it's in.
        let mut path = Utf8PathBuf::new();
        if let Some(ref tmx_to_tsx_file_path) = self.tmx_to_tsx_file_path {
            // If we also know the path to the tmx file, we can start with this, to hopefully
            // make the final path absolute where possible.
            if let Some(tmx_file_path) = tmx_file_path {
                let tmx_dir_path = utils::tmx_parent_dir(&tmx_file_path)?;
                path.push(tmx_dir_path);
            }
            let tmx_to_tsx_dir_path = utils::tsx_parent_dir(tmx_to_tsx_file_path)?;
            path.push(tmx_to_tsx_dir_path);
        }

        let tile_size = u32size2(self.tilewidth, self.tileheight);
        let size_in_tiles = u32size2(columns, rows);

        // Push the image source - if this is absolute, it replaces the base path, otherwise
        // it is added to the base path, so we have the full path to the image
        path.push(self.image.source.clone());

        let mode = self
            .image
            .trans
            .map(|color| TilesetMode::TransparentBackground {
                background: color.into(),
            })
            .unwrap_or_default();

        let tileset = Tileset::new_with_default_id(
            self.name.clone(),
            path,
            tile_size,
            size_in_tiles,
            mode,
            None,
            background,
            prefer_relative_path,
        );

        Ok(tileset)
    }

    pub fn add_to_tilesets(
        &self,
        tilesets: &mut Tilesets,
        prefer_relative_path: bool,
        tmx_file_path: Option<Utf8PathBuf>,
    ) -> eyre::Result<()> {
        let tileset = self.as_tileset(prefer_relative_path, tmx_file_path)?;
        tilesets.push_tileset(tileset);
        Ok(())
    }

    pub fn from_path_xml(path: Utf8PathBuf) -> eyre::Result<TilesetXml> {
        let file = File::open(path.clone())
            .wrap_err_with(|| format!("Failed to open Tiled .tsx file, expected at {}", path))?;
        let buf_reader = BufReader::new(file);
        let tiled: TilesetXml = quick_xml::de::from_reader(buf_reader)?;
        Ok(tiled)
    }

    pub fn save(&self, path: Utf8PathBuf) -> eyre::Result<()> {
        let file = File::create(path.clone())?;
        let buf_writer = BufWriter::new(file);
        let mut writer = Writer::new(buf_writer);
        writer.write_event(Event::Decl(BytesDecl::from_start(
            BytesStart::from_content("xml version=\"1.0\" encoding=\"UTF-8\"", 3),
        )))?;
        writer.write_serializable("tileset", &self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_tiled_xml_matches_json() -> eyre::Result<()> {
        let tiled_xml = TiledXml::from_path_xml("test-data/tiled-zoo.tmx".into())?;
        let tiled = tiled_xml.into_tiled()?;

        let mut tiled_json = Tiled::from_path_json("test-data/tiled-zoo.tmj".into())?;

        // Cheat a little - we need to make file_path match manually
        tiled_json.file_path = Some("test-data/tiled-zoo.tmx".into());

        assert_eq!(tiled, tiled_json);
        Ok(())
    }

    #[test]
    fn import_tileset_xml() -> eyre::Result<()> {
        let tileset_xml = TilesetXml::from_path_xml("test-data/mountain-tiles.tsx".into())?;
        let expected = TilesetXml {
            version: "1.10".to_string(),
            tiledversion: "1.11.0".to_string(),
            name: "mountain-tiles".to_string(),
            tileheight: 8,
            tilewidth: 8,
            tilecount: 256,
            columns: 16,
            backgroundcolor: None,
            image: ImageXml {
                source: "mountain-tiles.png".to_string(),
                height: 128,
                width: 128,
                trans: None,
            },
            tmx_to_tsx_file_path: None,
        };
        assert_eq!(tileset_xml, expected);
        Ok(())
    }
}
