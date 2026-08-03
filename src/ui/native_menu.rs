use std::sync::mpsc::Receiver;

use muda::{
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};

pub struct NativeMenu {
    /// Muda menu
    pub menu: Menu,

    /// Receiver for menu events
    pub rx: Receiver<MenuEvent>,
}

#[cfg(target_os = "macos")]
pub fn create_for_macos() -> muda::Result<NativeMenu> {
    use muda::MenuEvent;

    let menu = Menu::new();

    // Set up menu event channel
    let (tx, rx) = std::sync::mpsc::channel();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = tx.send(event);
    }));

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
    app_menu.append(&PredefinedMenuItem::quit(None))?;
    menu.append(&app_menu)?;

    // File menu
    let file_menu = Submenu::new("File", true);
    let new_item = MenuItem::new("New", true, None);
    let open_item = MenuItem::new("Open", true, None);
    let save_item = MenuItem::new(
        "Save",
        true,
        Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyS)),
    );
    file_menu.append(&new_item)?;
    file_menu.append(&open_item)?;
    file_menu.append(&save_item)?;
    file_menu.append(&PredefinedMenuItem::separator())?;
    file_menu.append(&PredefinedMenuItem::close_window(None))?;
    menu.append(&file_menu)?;

    // Edit menu
    let edit_menu = Submenu::new("Edit", true);
    edit_menu.append(&PredefinedMenuItem::undo(None))?;
    edit_menu.append(&PredefinedMenuItem::redo(None))?;
    edit_menu.append(&PredefinedMenuItem::separator())?;
    edit_menu.append(&PredefinedMenuItem::cut(None))?;
    edit_menu.append(&PredefinedMenuItem::copy(None))?;
    edit_menu.append(&PredefinedMenuItem::paste(None))?;
    edit_menu.append(&PredefinedMenuItem::select_all(None))?;
    menu.append(&edit_menu)?;

    // Window menu
    let window_menu = Submenu::new("Window", true);
    window_menu.append(&PredefinedMenuItem::minimize(None))?;
    window_menu.append(&PredefinedMenuItem::maximize(None))?;
    window_menu.append(&PredefinedMenuItem::separator())?;
    window_menu.append(&PredefinedMenuItem::fullscreen(None))?;
    menu.append(&window_menu)?;

    Ok(NativeMenu { menu, rx })
}
