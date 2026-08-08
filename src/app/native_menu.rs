use crate::{app::App, ui::native_menu::NativeMenuEvent};

impl App {
    pub(super) fn poll_and_handle_native_menu_events(&mut self, ctx: &egui::Context) -> bool {
        let mut handled = false;
        // Take native menu out so we can use it without holding self
        if let Some(native_menu) = self.native_menu.take() {
            while let Ok(event) = native_menu.rx.try_recv() {
                match event {
                    NativeMenuEvent::Settings => self.show_application_settings_modal(),
                    NativeMenuEvent::New => self.check_data_loss_then_new_document(),
                    NativeMenuEvent::Open => self.check_data_loss_then_show_open_document_modal(),
                    NativeMenuEvent::OpenRecent => self.show_select_recent_file_modal(),
                    NativeMenuEvent::Save => self.show_save_document_modal(),
                    NativeMenuEvent::SaveAs => self.show_save_as_document_modal(),
                    NativeMenuEvent::ImportPaletteImage => self.show_import_palette_modal(),
                    NativeMenuEvent::ExportPaletteImage => self.show_export_palette_modal(),
                    NativeMenuEvent::ImportPaletteLospec => self.show_import_palette_lospec_modal(),
                    NativeMenuEvent::ExportPaletteLospec => self.show_export_palette_lospec_modal(),
                    NativeMenuEvent::ImportTiled => self.pick_tiled_file_to_import(),
                    NativeMenuEvent::ExportTiled => self.show_export_tiled_modal(),
                    NativeMenuEvent::ExportPng => self.show_export_png_modal(),
                    NativeMenuEvent::ExportFromWorkspace => self.export_from_workspace(),
                    NativeMenuEvent::ResetZoom => self.reset_selected_map_zoom(),
                    NativeMenuEvent::Help => self.show_help_modal(),
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
