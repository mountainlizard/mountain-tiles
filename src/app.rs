use std::collections::VecDeque;

use camino::Utf8PathBuf;

use crate::data::edit_state::EditState;
use crate::data::mode::Mode;
use crate::data::settings::Settings;
use crate::data::state::State;
use crate::instance::IpcListener;
use crate::ui::tileset_textures::TilesetTextures;
use crate::undo::{RevisionIndex, Undo};

mod actions;
mod eframe_app;
pub mod files;
mod init;
mod ipc;
mod layers;
pub mod maps;
mod modals;
mod palette;
mod png;
mod selection;
mod stamp;
mod tiled;
mod tilesets;
mod undoredo;

pub const UNIQUE_ID: &str = "com.mountainlizard.mountain-tiles";
pub const USE_STORAGE: bool = true;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    /// Store the path the current data was most recently loaded from/saved to,
    /// to allow File->Save to save to the same file. This also allows us to
    /// attempt to re-open the file when starting the application.
    /// This is the only state that is persisted when app is closed.
    pub save_path: Option<Utf8PathBuf>,

    /// Store the most recent n paths data was loaded from/saved to,
    /// to allow display of recent documents list
    /// These are in order of last access, with the most recently
    /// loaded/saved earliest in the list.
    pub recent_paths: VecDeque<Utf8PathBuf>,

    /// Global, persistent settings for the app
    pub settings: Settings,

    /// True when user has requested to quit, which overrides the normal check for
    /// unsaved data.
    #[serde(skip)]
    quit_requested: bool,

    #[serde(skip)]
    pub state: State,

    #[serde(skip)]
    pub edit: EditState,

    #[serde(skip)]
    pub undo: Undo<State, EditState>,

    #[serde(skip)]
    pub saved_revision: Option<RevisionIndex>,

    /// Provide textures to draw tilesets - this is updated with the base dir
    /// where data is saved, to allow resolving relative image paths
    #[serde(skip)]
    pub textures: TilesetTextures,

    /// Allows listening for IPC messages, e.g. to open files
    #[serde(skip)]
    pub ipc_listener: Option<IpcListener>,
}

impl App {
    pub fn update_texture_base_dir_from_file_path(&mut self, path: Option<Utf8PathBuf>) {
        self.textures.update_base_dir_from_file_path(path);
    }

    fn apply_invariants(&mut self) {
        self.apply_default_tileset_selection();
        self.apply_default_tileset_tile_selection_by_id();
        self.apply_default_map_selection();
        self.apply_default_map_edit_state_by_id();
    }

    pub fn select_mode(&mut self) {
        self.edit.mode = Mode::Select;
    }
    pub fn erase_mode(&mut self) {
        self.edit.mode = Mode::Erase;
    }
    pub fn draw_mode(&mut self) {
        self.edit.mode = Mode::Draw;
    }
}
