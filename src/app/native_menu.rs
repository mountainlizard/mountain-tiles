use crate::{app::App, ui::native_menu::NativeMenuEvent};

impl App {
    pub(super) fn poll_and_handle_native_menu_events(&mut self, ctx: &egui::Context) -> bool {
        let mut handled = false;
        // Take native menu out so we can use it without holding self
        if let Some(native_menu) = self.native_menu.take() {
            while let Ok(event) = native_menu.rx.try_recv() {
                match event {
                    NativeMenuEvent::Quit => self.check_data_loss_then_quit(ctx),
                }
                handled = true;
            }
            // Put it back
            self.native_menu = Some(native_menu);
        }
        handled
    }
}
