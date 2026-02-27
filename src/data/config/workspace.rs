use std::{collections::HashMap, fs};

use camino::Utf8PathBuf;
use eyre::eyre;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum Endianness {
    #[default]
    Big,
    Little,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Project {
    pub export: Option<Export>,
}

impl Project {
    /// True if the export settings have any effect (i.e. they specify some data to be exported)
    pub fn export_has_effect(&self) -> bool {
        self.export.as_ref().is_some_and(Export::has_effect)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Export {
    /// If specified, all maps in the project are exported to a set of Rust modules in this file
    /// Note the path may be relative - when a workspace is loaded from a file, paths should be
    /// taken to be relative to that file.
    #[serde(rename = "module-path")]
    pub module_path: Option<Utf8PathBuf>,

    /// Export tilesets as a single png image to the specified file path.
    /// Note the path may be relative - when a workspace is loaded from a file, paths should be
    /// taken to be relative to that file.
    /// The image contains the tiles from all tilesets, ordered
    /// by tileset then tile indec within the tileset.
    /// The image consists of a single column of tiles.
    /// This is compatible with use in Bevy as an `ImageArrayLayout`, for
    /// example as the `tileset` for a `TilemapChunk`.
    #[serde(rename = "tileset-png-path")]
    pub tileset_png_path: Option<Utf8PathBuf>,

    /// Export tilesets as a single 1bit raw image.
    /// Note the path may be relative - when a workspace is loaded from a file, paths should be
    /// taken to be relative to that file.
    /// The 1bit raw data consists of just image data, with no header.
    /// Data is output in the normal order from top left, row by row.
    /// Each row consists of 1 bit per pixel, nominally this is 1 for white
    /// and 0 for black. The bits are packed into successive bytes, in
    /// BigEndian order, and the last byte is padded with zeroes if needed.
    /// This should be compatible with Imagemagick `convert` using a depth of 1.
    /// The image contains the tiles from all tilesets, ordered
    /// by tileset then tile indec within the tileset.
    /// The image consists of a single column of tiles.
    /// This is compatible with use in Bevy as an `ImageArrayLayout`, for
    /// example as the `tileset` for a `TilemapChunk`.
    #[serde(rename = "tileset-1bit-path")]
    pub tileset_1bit_path: Option<Utf8PathBuf>,

    /// If specified, sets the endianness of the 1bit raw image export.
    /// If not specified, defaults to little endian
    #[serde(rename = "tileset-1bit-endianness")]
    pub tileset_1bit_endianness: Option<Endianness>,

    /// Export palette as a png image to the specified file path.
    /// Note the path may be relative - when a workspace is loaded from a file, paths should be
    /// taken to be relative to that file.
    #[serde(rename = "palette-image-path")]
    pub palette_image_path: Option<Utf8PathBuf>,

    /// Export palette as lospec-compatible JSON to the specified file path.
    /// Note the path may be relative - when a workspace is loaded from a file, paths should be
    /// taken to be relative to that file.
    #[serde(rename = "palette-json-path")]
    pub palette_json_path: Option<Utf8PathBuf>,

    /// If this is specified, then the export will skip any map
    /// whose name starts with the given prefix, e.g. "_skip-this-map"
    #[serde(rename = "skip-maps-with-prefix")]
    pub skip_maps_with_prefix: Option<String>,
}

impl Export {
    /// True if the export settings have any effect (i.e. they specify some data to be exported)
    pub fn has_effect(&self) -> bool {
        self.module_path.is_some() || self.exports_tileset() || self.exports_palette()
    }

    /// True if tileset is exported in any format
    pub fn exports_tileset(&self) -> bool {
        self.tileset_png_path.is_some() || self.tileset_1bit_path.is_some()
    }

    /// True if palette is exported in any format
    pub fn exports_palette(&self) -> bool {
        self.palette_image_path.is_some() || self.palette_json_path.is_some()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Workspace {
    pub default: Option<Project>,
    pub project: Option<HashMap<String, Project>>,
}

impl Workspace {
    pub const FILENAME: &str = "mountain-tiles-workspace.toml";

    pub fn workspace_path_from_project_path(path: Utf8PathBuf) -> eyre::Result<Utf8PathBuf> {
        let dir = path
            .parent()
            .ok_or(eyre!("Project path {} has no parent directory", path))?;

        let mut workspace_path = Utf8PathBuf::from(dir);
        workspace_path.push(Self::FILENAME);
        Ok(workspace_path)
    }

    pub fn from_file(path: Utf8PathBuf) -> eyre::Result<Workspace> {
        let workspace_toml = fs::read_to_string(path.clone()).map_err(|e| {
            eyre!(
                "Can't open workspace TOML file.\n\nExpected at:\n{}\n\nError:\n{}\n\nExample data contains a template.",
                path,
                e
            )
        })?;
        let workspace: Workspace = toml::from_str(&workspace_toml).map_err(|e| {
            eyre!(
                "Can't parse workspace TOML file.\n\nFile is at:\n{}:\n\nError:\n{}\n\nExample data contains a template.",
                path,
                e
            )
        })?;
        Ok(workspace)
    }

    pub fn from_project_path(path: Utf8PathBuf) -> eyre::Result<Workspace> {
        let workspace_path = Self::workspace_path_from_project_path(path)?;
        Self::from_file(workspace_path)
    }

    fn specific_project_by_name(&self, project_name: &str) -> Option<Project> {
        if let Some(projects) = &self.project {
            if let Some(project) = projects.get(project_name) {
                return Some(project.clone());
            }
        }
        None
    }

    pub fn project_by_name(&self, project_name: &str) -> Option<Project> {
        self.specific_project_by_name(project_name)
            .or(self.default.clone())
    }
}

impl Project {
    pub fn from_project_path(path: Utf8PathBuf) -> eyre::Result<Project> {
        let project_name = path
            .file_stem()
            .ok_or(eyre!("Project path has no filename"))?;

        let workspace_path = Workspace::workspace_path_from_project_path(path.clone())?;

        let workspace = Workspace::from_file(workspace_path.clone())?;

        workspace
            .project_by_name(project_name)
            .ok_or(eyre!("No workspace settings found.\n\nCreate a file at:\n{}\n\nExample data contains a template.", workspace_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_full() -> eyre::Result<()> {
        let workspace = Workspace {
            default: Some(Project {
                export: Some(Export {
                    module_path: Some("../src/maps.rs".into()),
                    tileset_png_path: Some("../assets/tilesets/tileset.png".into()),
                    tileset_1bit_path: None,
                    tileset_1bit_endianness: Some(Endianness::Little),
                    palette_image_path: Some("../assets/palette/palette.png".into()),
                    palette_json_path: None,
                    skip_maps_with_prefix: Some("skip-".to_string()),
                }),
            }),
            project: Some(HashMap::from([(
                "example".to_string(),
                Project {
                    export: Some(Export {
                        module_path: Some("example/example.rs".into()),
                        ..Default::default()
                    }),
                },
            )])),
        };
        let toml = toml::to_string(&workspace)?;
        let expected = r#"[default.export]
module-path = "../src/maps.rs"
tileset-png-path = "../assets/tilesets/tileset.png"
tileset-1bit-endianness = "Little"
palette-image-path = "../assets/palette/palette.png"
skip-maps-with-prefix = "skip-"

[project.example.export]
module-path = "example/example.rs"
"#;
        assert_eq!(toml, expected);
        Ok(())
    }

    #[test]
    fn serialize_default_only() -> eyre::Result<()> {
        let workspace = Workspace {
            default: Some(Project {
                export: Some(Export {
                    module_path: Some("../src/maps.rs".into()),
                    tileset_png_path: Some("../assets/tilesets/tileset.png".into()),
                    tileset_1bit_path: None,
                    tileset_1bit_endianness: None,
                    palette_image_path: Some("../assets/palette/palette.png".into()),
                    palette_json_path: None,
                    skip_maps_with_prefix: None,
                }),
            }),
            ..Default::default()
        };
        let toml = toml::to_string(&workspace)?;
        let expected = r#"[default.export]
module-path = "../src/maps.rs"
tileset-png-path = "../assets/tilesets/tileset.png"
palette-image-path = "../assets/palette/palette.png"
"#;
        assert_eq!(toml, expected);
        Ok(())
    }

    #[test]
    fn serialize_empty() -> eyre::Result<()> {
        let workspace = Workspace::default();
        let toml = toml::to_string(&workspace)?;
        assert_eq!(toml, "");
        Ok(())
    }
}
