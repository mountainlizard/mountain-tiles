use crate::{
    data::{png::PngExportSettings, tiled::TiledExportSettings, tiles::Tiles},
    tiled::tiled_json::Tiled,
};
use camino::Utf8PathBuf;

use crate::{
    data::palette::{palette_index, Palette, PaletteIndex},
    data::settings::Settings,
    data::tiles::{layer_tiles::LayerId, tile_color::UserColor},
    data::tilesets::{Tileset, TilesetId, Tilesets},
    data::{
        maps::{Map, MapId},
        resources::TileResourceUse,
    },
};

#[derive(Clone, PartialEq, Default)]
pub enum ModalResult {
    #[default]
    Init,
    Active,
    Apply,
    Cancel,
}

#[derive(Clone, PartialEq)]
pub enum MapOperation {
    NewMap,
    UpdateExistingMap(MapId),
}

#[derive(Clone, PartialEq)]
pub enum TilesetOperation {
    NewTileset,
    UpdateExistingTileset(TilesetId),
}

#[derive(Clone, PartialEq)]
pub enum DataLossOperation {
    New,
    Open,
    OpenFileArgument {
        path: Utf8PathBuf,
    },
    ImportTiled,
    DeleteTileset {
        tileset_id: TilesetId,
        tileset_name: String,
        tileset_use: TileResourceUse,
    },
    ReplacePalette {
        palette: Palette,
        resource_use: TileResourceUse,
    },
    Quit,
}

#[derive(Clone, PartialEq, Default)]
pub enum ModalState {
    #[default]
    None,
    Map {
        name: String,
        width: u32,
        height: u32,
        tile_width: u32,
        tile_height: u32,
        background_color: UserColor,
        background_color_as_text: String,
        operation: MapOperation,
        result: ModalResult,
    },
    Tileset {
        // Note we use a `Tilesets` instance, but only the last `Tileset` is used.
        // It's recommended to have a single tileset in the `Tilesets`.
        // If the `Tilesets` instance contains no tilesets, then operations will have
        // no effect, and the UI for the modal will not display any contents.
        tilesets: Tilesets,
        operation: TilesetOperation,
        default_foreground: UserColor,
        default_foreground_as_text: String,
        default_background: UserColor,
        default_background_as_text: String,
        default_transparent: UserColor,
        default_transparent_as_text: String,
        result: ModalResult,
    },
    Layer {
        map_id: MapId,
        layer_id: LayerId,
        name: String,
        opacity: Option<f32>,
        result: ModalResult,
    },
    ImportTiled {
        tiled: Tiled,
        squash_layers: bool,
        prefer_relative_path: bool,
        result: ModalResult,
    },
    Error {
        message: String,
        result: ModalResult,
    },
    DataLoss {
        operation: DataLossOperation,
        result: ModalResult,
    },
    Palette {
        selected_index: PaletteIndex,
        palette: Palette,
        scroll_to_row: Option<usize>,
        edit_color_as_text: String,
        result: ModalResult,
    },
    Settings {
        settings: Settings,
        result: ModalResult,
    },
    ExportPng {
        settings: PngExportSettings,
        result: ModalResult,
    },
    ExportTiled {
        settings: TiledExportSettings,
        result: ModalResult,
    },
    Help {
        result: ModalResult,
    },
}

impl ModalState {
    pub fn result(&self) -> Option<ModalResult> {
        match self {
            ModalState::None => None,
            ModalState::Map { result, .. } => Some(result.clone()),
            ModalState::Tileset { result, .. } => Some(result.clone()),
            ModalState::Layer { result, .. } => Some(result.clone()),
            ModalState::ImportTiled { result, .. } => Some(result.clone()),
            ModalState::Error { result, .. } => Some(result.clone()),
            ModalState::DataLoss { result, .. } => Some(result.clone()),
            ModalState::Palette { result, .. } => Some(result.clone()),
            ModalState::Settings { result, .. } => Some(result.clone()),
            ModalState::ExportPng { result, .. } => Some(result.clone()),
            ModalState::ExportTiled { result, .. } => Some(result.clone()),
            ModalState::Help { result, .. } => Some(result.clone()),
        }
    }

    pub fn make_active(&mut self) {
        match self {
            ModalState::None => {}
            ModalState::Map { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::Tileset { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::Layer { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::ImportTiled { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::Error { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::DataLoss { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::Palette { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::Settings { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::ExportPng { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::ExportTiled { ref mut result, .. } => *result = ModalResult::Active,
            ModalState::Help { ref mut result, .. } => *result = ModalResult::Active,
        }
    }

    pub fn tileset(tileset: Tileset, operation: TilesetOperation) -> ModalState {
        let transparent = tileset.mode.default_transparent_background();
        let mut tilesets = Tilesets::new();
        tilesets.push_tileset(tileset);

        ModalState::Tileset {
            tilesets,
            operation,
            default_foreground: UserColor::WHITE,
            default_foreground_as_text: String::new(),
            default_background: UserColor::BLACK,
            default_background_as_text: String::new(),
            default_transparent: transparent,
            default_transparent_as_text: transparent.as_css_string(),
            result: ModalResult::Init,
        }
    }

    pub fn dataloss(operation: DataLossOperation) -> ModalState {
        ModalState::DataLoss {
            operation,
            result: ModalResult::Init,
        }
    }

    pub fn palette(palette: Palette) -> ModalState {
        ModalState::Palette {
            selected_index: palette_index(0),
            palette,
            scroll_to_row: Some(0),
            edit_color_as_text: String::new(),
            result: ModalResult::Init,
        }
    }

    pub fn new_map() -> ModalState {
        ModalState::Map {
            name: "New Map".to_string(),
            width: 32,
            height: 32,
            tile_width: 8,
            tile_height: 8,
            background_color: UserColor::BLACK,
            background_color_as_text: String::new(),
            operation: MapOperation::NewMap,
            result: Default::default(),
        }
    }

    pub(crate) fn edit_map(map: &Map) -> ModalState {
        let size = map.tiles().map_size();
        let tile_size = map.tiles().tile_size();
        ModalState::Map {
            name: map.name(),
            width: size.w,
            height: size.h,
            tile_width: tile_size.w,
            tile_height: tile_size.h,
            background_color: map.tiles().background,
            background_color_as_text: String::new(),
            operation: MapOperation::UpdateExistingMap(map.id()),
            result: Default::default(),
        }
    }

    pub(crate) fn help() -> ModalState {
        ModalState::Help {
            result: ModalResult::Init,
        }
    }

    pub(crate) fn settings(settings: Settings) -> ModalState {
        ModalState::Settings {
            settings,
            result: Default::default(),
        }
    }

    pub(crate) fn error(message: &str) -> ModalState {
        ModalState::Error {
            message: message.to_string(),
            result: Default::default(),
        }
    }

    pub(crate) fn tiled(tiled: Tiled) -> ModalState {
        ModalState::ImportTiled {
            tiled,
            squash_layers: false,
            prefer_relative_path: true,
            result: Default::default(),
        }
    }

    pub(crate) fn export_tiled() -> ModalState {
        ModalState::ExportTiled {
            settings: Default::default(),
            result: Default::default(),
        }
    }

    pub(crate) fn export_png() -> ModalState {
        ModalState::ExportPng {
            settings: Default::default(),
            result: Default::default(),
        }
    }
}
