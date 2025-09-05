#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct TiledExportSettings {
    pub include_layer_data_as_properties: bool,
    pub export_tsx_files: bool,
}

impl Default for TiledExportSettings {
    fn default() -> Self {
        Self {
            include_layer_data_as_properties: true,
            export_tsx_files: true,
        }
    }
}
