use std::sync::mpsc::Receiver;

#[cfg(target_os = "macos")]
use arboard::Clipboard;
#[cfg(target_os = "macos")]
use egui::Context;
#[cfg(target_os = "macos")]
use muda::accelerator::{CMD_OR_CTRL, Modifiers};
use muda::{
    Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code},
};

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
}

#[cfg(target_os = "macos")]
pub fn create_for_macos(ctx: Context) -> muda::Result<NativeMenu> {
    use muda::MenuEvent;

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
    app_menu.append(&PredefinedMenuItem::about(None, None))?;
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
    // let file_menu = Submenu::new("File", true);
    // let new_item = MenuItem::new(
    //     "New",
    //     true,
    //     Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyN)),
    // );
    // let open_item = MenuItem::new(
    //     "Open",
    //     true,
    //     Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyO)),
    // );
    // let save_item = MenuItem::new(
    //     "Save",
    //     true,
    //     Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyS)),
    // );
    // file_menu.append(&new_item)?;
    // file_menu.append(&open_item)?;
    // file_menu.append(&save_item)?;
    // file_menu.append(&PredefinedMenuItem::separator())?;
    // file_menu.append(&PredefinedMenuItem::close_window(None))?;
    // menu.append(&file_menu)?;

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

    // edit_menu.append(&PredefinedMenuItem::select_all(None))?;
    menu.append(&edit_menu)?;

    // Window menu
    let window_menu = Submenu::new("Window", true);
    window_menu.append(&PredefinedMenuItem::minimize(None))?;
    window_menu.append(&PredefinedMenuItem::maximize(None))?;
    window_menu.append(&PredefinedMenuItem::separator())?;
    window_menu.append(&PredefinedMenuItem::fullscreen(None))?;
    menu.append(&window_menu)?;

    // Set up menu event channels
    let (tx, rx) = std::sync::mpsc::channel();
    let (tx_raw, rx_raw) = std::sync::mpsc::channel();

    // Handle events by sending on to the
    // event channels, and triggering an egui repaint
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let native_menu_event = if *event.id() == quit_id {
            Some(NativeMenuEvent::Quit)
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
    let mut clipboard = Clipboard::new()?;
    let text = clipboard.get_text()?;
    Ok(text)
}
