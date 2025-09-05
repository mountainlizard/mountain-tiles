/// Global, persistent settings for the app
#[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Settings {
    /// If true, the tileset grid will have spacing to separate tiles visually
    pub tileset_grid_spacing_enabled: bool,
}
