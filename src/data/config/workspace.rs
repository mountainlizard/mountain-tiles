use std::collections::HashMap;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Project {
    pub export: Option<Export>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Export {
    #[serde(rename = "module-path")]
    pub module_path: Option<Utf8PathBuf>,
    #[serde(rename = "tileset-path")]
    pub tileset_path: Option<Utf8PathBuf>,
    #[serde(rename = "tileset-png")]
    pub tileset_png: Option<bool>,
    #[serde(rename = "tileset-1bit")]
    pub tileset_1bit: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub default: Option<Project>,
    pub project: Option<HashMap<String, Project>>,
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
                    tileset_path: Some("../assets/tilesets/".into()),
                    tileset_png: Some(true),
                    tileset_1bit: None,
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
tileset-path = "../assets/tilesets/"
tileset-png = true

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
                    tileset_path: Some("../assets/tilesets/".into()),
                    tileset_png: Some(true),
                    tileset_1bit: None,
                }),
            }),
            ..Default::default()
        };
        let toml = toml::to_string(&workspace)?;
        let expected = r#"[default.export]
module-path = "../src/maps.rs"
tileset-path = "../assets/tilesets/"
tileset-png = true
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
