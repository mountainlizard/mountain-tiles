#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct RawExportSettings {
    pub export_project_to_rust: bool,
    pub export_combined_png_tileset: bool,
    // pub export_combined_1bit_tileset: bool,
}

impl Default for RawExportSettings {
    fn default() -> Self {
        Self {
            export_project_to_rust: true,
            export_combined_png_tileset: true,
            // export_combined_1bit_tileset: true,
        }
    }
}
