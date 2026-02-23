use std::env;

use camino::Utf8PathBuf;
use egui::{vec2, ThemePreference};
use egui_notify::Anchor;

use crate::{
    app::{
        files::{OpenContext, StateSource},
        App, UNIQUE_ID, USE_STORAGE,
    },
    instance::create_ipc_listener,
    ui::egui_utils,
    ui::theme::{self, DEFAULT_THEME},
};

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let is_macos = cfg!(target_os = "macos");

        // Create a listener for IPC messages - we do this early
        // since it will start listening and buffering messages immediately,
        // reducing the chances of missing a message from an instance started
        // immediately after this one
        // This is not required on macOS
        let ipc_listener = if is_macos {
            None
        } else {
            match create_ipc_listener(UNIQUE_ID, cc.egui_ctx.clone()) {
                Ok(ipc_listener) => Some(ipc_listener),
                Err(e) => {
                    log::error!("Failed to create IPC listener: {}", e);
                    None
                }
            }
        };

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // We also install our own image loaders - this works similarly to the `image`
        // loader feature in egui_extras, but uses a `tileset://` protocol and includes
        // a `TilesetMode` allowing us to handle transparent color setting etc.
        egui_utils::install_image_loaders(&cc.egui_ctx);

        // Restore data from eframe storage
        let mut data: App = match (USE_STORAGE, cc.storage) {
            (true, Some(storage)) => {
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
            }
            _ => Default::default(),
        };

        // Store ipc_listener to be polled on updates
        data.ipc_listener = ipc_listener;

        // Customise toasts
        data.toasts = data
            .toasts
            .with_anchor(Anchor::BottomRight)
            .with_margin(vec2(32.0, 32.0))
            .with_padding(vec2(16.0, 16.0));

        data.update_texture_base_dir_from_file_path(data.save_path.clone());

        let args: Vec<String> = env::args().collect();
        let filename = args.get(1);

        // Try to open initial file, if we have one
        let opened = if let Some(filename) = filename {
            // If we were called with a filename, try to open it
            data.open_document(Utf8PathBuf::from(filename), OpenContext::FileArgument)
        } else if let Some(save_path) = data.save_path.clone() {
            // If we have a save path from stored application state, try to load the project
            // Note this shows error modal if load fails, which is fine
            data.open_document(save_path, OpenContext::AppStartup)
        } else {
            false
        };

        // If we didn't open a file, make sure we use a default map in the same
        // way as if it was produced by File->New, but skipping the modal. This sets
        // up undo, etc.
        if !opened {
            data.use_state(Default::default(), StateSource::NewOnAppStartup);
        }

        // Note, loading data from storage also loads and applies egui theme settings,
        // so we need to set the custom theme afterwards - there may be a neater
        // way of doing this
        theme::apply_theme(&cc.egui_ctx, DEFAULT_THEME);
        cc.egui_ctx.set_theme(ThemePreference::Dark);

        egui_utils::replace_fonts(&cc.egui_ctx);

        data
    }
}
