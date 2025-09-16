#![warn(
    clippy::all,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    rust_2018_idioms
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use mountain_tiles::instance::instance_startup;
use std::process::exit;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use mountain_tiles::app::{App, UNIQUE_ID};

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    if let Err(e) = color_eyre::install() {
        log::error!("Failed to install eyre error reporting: {}", e);
        exit(1);
    }

    // Handle instance startup - if this returns true we should
    // exit without error
    if instance_startup(UNIQUE_ID) {
        println!("Application is already running - will exit.");
        exit(0);
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([400.0, 300.0])
        .with_min_inner_size([300.0, 220.0])
        // Set app id for wayland so it matches the desktop file, which is named after
        // our binary, which is named after our crate.
        // At some point it would be nice to use `com.mountainlizard.mountain_tiles` since
        // this follows the freedesktop.org spec, but this would require a change to
        // cargo-packager, and looks like it might interact with other naming (e.g. in AppImage),
        // so for now we just assume that `mountain-tiles` is unusual enough that it's
        // unlikely to be used as the app id of an unrelated application.
        .with_app_id("mountain-tiles")
        .with_title("MountainTiles");

    match eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..]) {
        Ok(icon) => viewport = viewport.with_icon(icon),
        Err(e) => log::warn!("Failed to load icon {}", e),
    }

    let is_macos = cfg!(target_os = "macos");
    if is_macos {
        viewport = viewport
            .with_title_shown(false)
            .with_titlebar_shown(false)
            .with_fullsize_content_view(true);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        // Use the app id for app name, just in case it does end up getting used by wayland,
        // see `.with_app_id` call above for why we use this value.
        "mountain-tiles",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(mountain_tiles::MainApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
