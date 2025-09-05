#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct PngExportSettings {
    pub scale: u32,
    pub transparent: bool,
}

impl Default for PngExportSettings {
    fn default() -> Self {
        Self {
            scale: 4,
            transparent: false,
        }
    }
}
