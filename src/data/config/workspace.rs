use std::{collections::HashMap, fs};

use camino::Utf8PathBuf;
use eyre::eyre;
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "module-path")]
    pub module_path: Option<Utf8PathBuf>,

    #[serde(rename = "tileset-png-path")]
    pub tileset_png_path: Option<Utf8PathBuf>,

    #[serde(rename = "tileset-1bit-path")]
    pub tileset_1bit_path: Option<Utf8PathBuf>,

    #[serde(rename = "palette-image-path")]
    pub palette_image_path: Option<Utf8PathBuf>,

    #[serde(rename = "palette-json-path")]
    pub palette_json_path: Option<Utf8PathBuf>,
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
                    palette_image_path: Some("../assets/palette/palette.png".into()),
                    palette_json_path: None,
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
palette-image-path = "../assets/palette/palette.png"

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
                    palette_image_path: Some("../assets/palette/palette.png".into()),
                    palette_json_path: None,
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
