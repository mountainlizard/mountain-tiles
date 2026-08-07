use std::sync::mpsc::Receiver;

use muda::Menu;

pub struct NativeMenu {
    /// Muda menu
    pub menu: Menu,

    /// Receiver for menu events
    pub rx: Receiver<NativeMenuEvent>,

    /// Receiver for menu raw events
    pub rx_raw: Receiver<NativeMenuRawEvent>,
}

/// Events fired by clicking a menu item, and handled by
/// application code during app update.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NativeMenuEvent {
    Settings,
    New,
    Open,
    Save,
    SaveAs,
    ImportPaletteImage,
    ExportPaletteImage,
    ImportPaletteLospec,
    ExportPaletteLospec,
    ImportTiled,
    ExportTiled,
    ExportPng,
    ExportFromWorkspace,
    ResetZoom,
    Help,
    Quit,
}

/// Events fired by clicking a menu item, and handled by
/// converting them into raw input events for egui.
#[derive(Debug, Clone, PartialEq)]
pub enum NativeMenuRawEvent {
    Cut,
    Copy,
    Paste(String),
    Undo,
    Redo,
    SelectAll,
}

#[cfg(target_os = "macos")]
pub fn create_for_macos(ctx: egui::Context) -> muda::Result<NativeMenu> {
    use muda::{
        MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
        accelerator::{Accelerator, CMD_OR_CTRL, Code, Modifiers},
    };

    let menu = Menu::new();

    // When egui/winit support catching macOS messages via a delegate,
    // we may be able to move back to predefined menu items for
    // cut/copy/paste, undo/redo and quit. At the moment they can't
    // be used since we can't respond to the messages they produce.
    // Therefore we use normal menu items with the correct text and
    // accelerators as stand-ins, and we can implement their behaviour
    // ourselves.
    // This does mean we don't get the relevant icons,
    // although I think they are going in macOS27 anyway...

    // App menu (first menu with app name)
    let app_menu = Submenu::new("App", true);

    let settings_item = MenuItem::new(
        "Settings...",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::Comma)),
    );
    let settings_id = settings_item.id().clone();

    app_menu.append(&PredefinedMenuItem::about(None, None))?;
    app_menu.append(&PredefinedMenuItem::separator())?;
    app_menu.append(&settings_item)?;
    app_menu.append(&PredefinedMenuItem::separator())?;
    app_menu.append(&PredefinedMenuItem::services(None))?;
    app_menu.append(&PredefinedMenuItem::separator())?;
    app_menu.append(&PredefinedMenuItem::hide(None))?;
    app_menu.append(&PredefinedMenuItem::hide_others(None))?;
    app_menu.append(&PredefinedMenuItem::show_all(None))?;
    app_menu.append(&PredefinedMenuItem::separator())?;

    // Use predefined item just for the text, which includes
    // the application name.
    let predefined_quit_item = PredefinedMenuItem::quit(None);
    let quit_item = MenuItem::new(
        predefined_quit_item.text(),
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyQ)),
    );
    let quit_id = quit_item.id().clone();
    app_menu.append(&quit_item)?;

    menu.append(&app_menu)?;

    // File menu
    let file_menu = Submenu::new("File", true);
    let new_item = MenuItem::new(
        "New",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyN)),
    );
    let new_id = new_item.id().clone();

    let open_item = MenuItem::new(
        "Open",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyO)),
    );
    let open_id = open_item.id().clone();

    let save_item = MenuItem::new(
        "Save",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyS)),
    );
    let save_id = save_item.id().clone();

    let save_as_item = MenuItem::new(
        "Save As...",
        true,
        Some(Accelerator::new(
            Some(Modifiers::SHIFT | CMD_OR_CTRL),
            Code::KeyS,
        )),
    );
    let save_as_id = save_as_item.id().clone();

    let import_palette_image_item = MenuItem::new("Import Palette (image)...", true, None);
    let import_palette_image_id = import_palette_image_item.id().clone();

    let export_palette_image_item = MenuItem::new("Export Palette (image)...", true, None);
    let export_palette_image_id = export_palette_image_item.id().clone();

    let import_palette_lospec_item = MenuItem::new("Import Palette (lospec JSON)...", true, None);
    let import_palette_lospec_id = import_palette_lospec_item.id().clone();

    let export_palette_lospec_item = MenuItem::new("Export Palette (lospec JSON)...", true, None);
    let export_palette_lospec_id = export_palette_lospec_item.id().clone();

    let import_tiled_item = MenuItem::new("Import Tiled...", true, None);
    let import_tiled_id = import_tiled_item.id().clone();

    let export_tiled_item = MenuItem::new("Export Tiled...", true, None);
    let export_tiled_id = export_tiled_item.id().clone();

    let export_png_item = MenuItem::new("Export PNG...", true, None);
    let export_png_id = export_png_item.id().clone();

    let export_from_workspace_item = MenuItem::new("Export from workspace...", true, None);
    let export_from_workspace_id = export_from_workspace_item.id().clone();

    file_menu.append(&new_item)?;
    file_menu.append(&open_item)?;
    file_menu.append(&save_item)?;
    file_menu.append(&save_as_item)?;
    file_menu.append(&PredefinedMenuItem::separator())?;
    file_menu.append(&import_palette_image_item)?;
    file_menu.append(&export_palette_image_item)?;
    file_menu.append(&import_palette_lospec_item)?;
    file_menu.append(&export_palette_lospec_item)?;
    file_menu.append(&import_tiled_item)?;
    file_menu.append(&export_tiled_item)?;
    file_menu.append(&export_png_item)?;
    file_menu.append(&export_from_workspace_item)?;
    file_menu.append(&PredefinedMenuItem::separator())?;
    file_menu.append(&PredefinedMenuItem::close_window(None))?;
    menu.append(&file_menu)?;

    // Edit menu
    let edit_menu = Submenu::new("Edit", true);
    let undo_item = MenuItem::new(
        "Undo",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyZ)),
    );
    let undo_id = undo_item.id().clone();

    let redo_item = MenuItem::new(
        "Redo",
        true,
        Some(Accelerator::new(
            Some(Modifiers::SHIFT | CMD_OR_CTRL),
            Code::KeyZ,
        )),
    );
    let redo_id = redo_item.id().clone();

    let cut_item = MenuItem::new(
        "Cut",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyX)),
    );
    let cut_id = cut_item.id().clone();

    let copy_item = MenuItem::new(
        "Copy",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyC)),
    );
    let copy_id = copy_item.id().clone();

    let paste_item = MenuItem::new(
        "Paste",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyV)),
    );
    let paste_id = paste_item.id().clone();

    edit_menu.append(&undo_item)?;
    edit_menu.append(&redo_item)?;
    edit_menu.append(&PredefinedMenuItem::separator())?;
    edit_menu.append(&cut_item)?;
    edit_menu.append(&copy_item)?;
    edit_menu.append(&paste_item)?;

    let select_all_item = MenuItem::new(
        "Select All",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyA)),
    );
    let select_all_id = select_all_item.id().clone();

    edit_menu.append(&select_all_item)?;
    menu.append(&edit_menu)?;

    // View menu
    let view_menu = Submenu::new("View", true);
    let reset_zoom_item = MenuItem::new(
        "Reset Zoom",
        true,
        Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyR)),
    );
    let reset_zoom_id = reset_zoom_item.id().clone();
    view_menu.append(&reset_zoom_item)?;
    menu.append(&view_menu)?;

    // Window menu
    let window_menu = Submenu::new("Window", true);
    window_menu.append(&PredefinedMenuItem::minimize(None))?;
    window_menu.append(&PredefinedMenuItem::maximize(None))?;
    window_menu.append(&PredefinedMenuItem::separator())?;
    window_menu.append(&PredefinedMenuItem::fullscreen(None))?;
    menu.append(&window_menu)?;

    // Help menu
    let help_menu = Submenu::new("Help", true);
    let help_item = MenuItem::new("MountainTiles Help", true, None);
    let help_id = help_item.id().clone();
    help_menu.append(&help_item)?;
    menu.append(&help_menu)?;

    // Set up menu event channels
    let (tx, rx) = std::sync::mpsc::channel();
    let (tx_raw, rx_raw) = std::sync::mpsc::channel();

    // Handle events by sending on to the
    // event channels, and triggering an egui repaint
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let native_menu_event = if *event.id() == quit_id {
            Some(NativeMenuEvent::Quit)
        } else if *event.id() == settings_id {
            Some(NativeMenuEvent::Settings)
        } else if *event.id() == new_id {
            Some(NativeMenuEvent::New)
        } else if *event.id() == open_id {
            Some(NativeMenuEvent::Open)
        } else if *event.id() == save_id {
            Some(NativeMenuEvent::Save)
        } else if *event.id() == save_as_id {
            Some(NativeMenuEvent::SaveAs)
        } else if *event.id() == import_palette_image_id {
            Some(NativeMenuEvent::ImportPaletteImage)
        } else if *event.id() == export_palette_image_id {
            Some(NativeMenuEvent::ExportPaletteImage)
        } else if *event.id() == import_palette_lospec_id {
            Some(NativeMenuEvent::ImportPaletteLospec)
        } else if *event.id() == export_palette_lospec_id {
            Some(NativeMenuEvent::ExportPaletteLospec)
        } else if *event.id() == import_tiled_id {
            Some(NativeMenuEvent::ImportTiled)
        } else if *event.id() == export_tiled_id {
            Some(NativeMenuEvent::ExportTiled)
        } else if *event.id() == export_png_id {
            Some(NativeMenuEvent::ExportPng)
        } else if *event.id() == export_from_workspace_id {
            Some(NativeMenuEvent::ExportFromWorkspace)
        } else if *event.id() == reset_zoom_id {
            Some(NativeMenuEvent::ResetZoom)
        } else if *event.id() == help_id {
            Some(NativeMenuEvent::Help)
        } else {
            None
        };
        if let Some(e) = native_menu_event {
            let _ = tx.send(e);
            ctx.request_repaint();
        }

        let native_menu_raw_event = if *event.id() == cut_id {
            Some(NativeMenuRawEvent::Cut)
        } else if *event.id() == copy_id {
            Some(NativeMenuRawEvent::Copy)
        } else if *event.id() == paste_id {
            match get_clipboard_text() {
                Ok(text) => Some(NativeMenuRawEvent::Paste(text)),
                Err(e) => {
                    log::warn!("Error creating/reading clipboard: {}", e);
                    None
                }
            }
        } else if *event.id() == undo_id {
            Some(NativeMenuRawEvent::Undo)
        } else if *event.id() == redo_id {
            Some(NativeMenuRawEvent::Redo)
        } else if *event.id() == select_all_id {
            Some(NativeMenuRawEvent::SelectAll)
        } else {
            None
        };
        if let Some(e) = native_menu_raw_event {
            let _ = tx_raw.send(e);
            ctx.request_repaint();
        }
    }));

    Ok(NativeMenu { menu, rx, rx_raw })
}

#[cfg(target_os = "macos")]
fn get_clipboard_text() -> eyre::Result<String> {
    let mut clipboard = arboard::Clipboard::new()?;
    let text = clipboard.get_text()?;
    Ok(text)
}
