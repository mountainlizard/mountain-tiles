use std::sync::mpsc::Receiver;

#[cfg(target_os = "macos")]
use egui::Context;
#[cfg(target_os = "macos")]
use muda::accelerator::CMD_OR_CTRL;
use muda::{
    Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code},
};

pub struct NativeMenu {
    /// Muda menu
    pub menu: Menu,

    /// Receiver for menu events
    pub rx: Receiver<NativeMenuEvent>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NativeMenuEvent {
    Quit,
}

#[cfg(target_os = "macos")]
pub fn create_for_macos(ctx: Context) -> muda::Result<NativeMenu> {
    use muda::MenuEvent;

    let menu = Menu::new();

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

    let predefined_quit_item = PredefinedMenuItem::quit(None);

    // TODO: This item should have an icon to match predefined,
    // although I think this is going in macOS27 anyway... Not
    // sure how the icon is assigned, maybe based on the
    // terminate selector, which we don't want...
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
    // let edit_menu = Submenu::new("Edit", true);
    // edit_menu.append(&PredefinedMenuItem::undo(None))?;
    // edit_menu.append(&PredefinedMenuItem::redo(None))?;
    // edit_menu.append(&PredefinedMenuItem::separator())?;
    // edit_menu.append(&PredefinedMenuItem::cut(None))?;
    // edit_menu.append(&PredefinedMenuItem::copy(None))?;
    // edit_menu.append(&PredefinedMenuItem::paste(None))?;
    // edit_menu.append(&PredefinedMenuItem::select_all(None))?;
    // menu.append(&edit_menu)?;

    // Window menu
    let window_menu = Submenu::new("Window", true);
    window_menu.append(&PredefinedMenuItem::minimize(None))?;
    window_menu.append(&PredefinedMenuItem::maximize(None))?;
    window_menu.append(&PredefinedMenuItem::separator())?;
    window_menu.append(&PredefinedMenuItem::fullscreen(None))?;
    menu.append(&window_menu)?;

    // Set up menu event channel
    let (tx, rx) = std::sync::mpsc::channel();
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
    }));

    Ok(NativeMenu { menu, rx })
}
