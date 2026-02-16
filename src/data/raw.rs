#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct RawExportSettings {
    // pub export_selected_map_only: bool,
    // pub export_first_layer_only: bool,
    pub export_combined_png_tileset: bool,
    // pub export_combined_1bit_tileset: bool,
}

impl Default for RawExportSettings {
    fn default() -> Self {
        Self {
            // export_selected_map_only: true,
            // export_first_layer_only: true,
            export_combined_png_tileset: true,
            // export_combined_1bit_tileset: true,
        }
    }
}
