#![warn(
    clippy::all,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    rust_2018_idioms
)]

pub mod app;
pub mod instance;
mod render;
mod selection;
mod undo;
mod utils;

pub mod data {
    pub mod action;
    pub mod edit_state;
    pub mod file_format;
    pub mod map_edit_state;
    pub mod maps;
    pub mod modal;
    pub mod mode;
    pub mod palette;
    pub mod png;
    pub mod resources;
    pub mod settings;
    pub mod stamp;
    pub mod state;
    pub mod tiled;
    pub mod tiles;
    pub mod tilesets;
}

pub mod geom {
    pub mod i32pos2;
    pub mod transform;
    pub mod u32pos2;
    pub mod u32rect;
    pub mod u32size2;
}

pub mod tiled {
    pub mod tiled_color;
    pub mod tiled_json;
    pub mod tiled_xml;
}

pub mod ui {
    pub mod color_edit;
    pub mod egui_utils;
    pub mod file_dialog;
    pub mod layers;
    pub mod map;
    pub mod maps;
    pub mod menu;
    pub mod modal;
    pub mod palette;
    pub mod shortcuts;
    pub mod theme;
    pub mod tile_mesh;
    pub mod tiles;
    pub mod tileset;
    pub mod tileset_image_loader;
    pub mod tileset_textures;
    pub mod utils;
}
