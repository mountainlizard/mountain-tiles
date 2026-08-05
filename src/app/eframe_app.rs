use egui::{Key, Modifiers, Ui};

use crate::{
    app::App,
    data::modal::{DataLossOperation, ModalState},
    ui::{
        egui_utils::separator,
        layers::layers_ui,
        map::map_ui,
        maps::maps_ui,
        menu::menu_ui,
        modal::{
            data_loss_modal::data_loss_modal_ui, error_modal::error_modal_ui,
            export_png_modal::export_png_modal_ui, export_tiled_modal::export_tiled_modal_ui,
            help_modal::help_modal_ui, import_tiled_modal::import_tiled_modal_ui,
            layer_modal::layer_modal_ui, map_modal::map_modal_ui, palette_modal::palette_modal_ui,
            settings_modal::settings_modal_ui, tileset_modal::tileset_modal_ui,
        },
        native_menu::NativeMenuRawEvent,
        palette::palette_ui,
        shortcuts::consume_shortcuts,
        theme::DEFAULT_THEME,
        tileset::tileset_ui,
    },
};

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.viewport().close_requested())
            && self.may_have_unsaved_changes()
            && !self.quit_requested
        {
            ui.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_data_loss_modal(DataLossOperation::Quit);
        }

        // Make sure app state remains consistent
        self.apply_invariants();

        self.poll_and_handle_all_ipc_messages(ui);

        self.poll_and_handle_native_menu_events(ui);

        let menu_frame = DEFAULT_THEME.base_100_frame(2);

        egui::Panel::top("main_app_top_panel")
            .frame(menu_frame)
            .show(ui, |ui| {
                menu_ui(ui, self);
            });

        let side_frame = DEFAULT_THEME.base_100_frame(16);

        egui::Panel::left("main_app_left_panel")
            .frame(side_frame)
            .resizable(true)
            .default_size(350.0)
            .min_size(275.0)
            .max_size(750.0)
            .show(ui, |ui| {
                maps_ui(ui, self);

                separator(ui);

                tileset_ui(ui, self);

                separator(ui);

                palette_ui(ui, self);

                separator(ui);

                layers_ui(ui, self);
            });

        let centre_frame = DEFAULT_THEME.base_200_frame(0);

        egui::CentralPanel::default()
            .frame(centre_frame)
            .show(ui, |ui| {
                map_ui(ui, self);

                match self.edit.modal {
                    ModalState::None => consume_shortcuts(ui, self),
                    ModalState::Map { .. } => map_modal_ui(ui, self),
                    ModalState::Tileset { .. } => tileset_modal_ui(ui, self),
                    ModalState::Layer { .. } => layer_modal_ui(ui, self),
                    ModalState::ImportTiled { .. } => import_tiled_modal_ui(ui, self),
                    ModalState::Error { .. } => error_modal_ui(ui, self),
                    ModalState::DataLoss { .. } => data_loss_modal_ui(ui, self),
                    ModalState::Palette { .. } => palette_modal_ui(ui, self),
                    ModalState::Settings { .. } => settings_modal_ui(ui, self),
                    ModalState::ExportPng { .. } => export_png_modal_ui(ui, self),
                    ModalState::ExportTiled { .. } => export_tiled_modal_ui(ui, self),
                    ModalState::Help { .. } => help_modal_ui(ui, self),
                }
            });

        self.feed_undo();

        self.toasts.show(ui);
    }

    // TODO: Move to new module e.g. `raw_input`.
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Take native menu out so we can use it without holding self
        if let Some(native_menu) = self.native_menu.take() {
            while let Ok(event) = native_menu.rx_raw.try_recv() {
                println!("Handling raw input for {:?}", event);
                match event {
                    // We have to push non-key events here - confusingly egui does have
                    // some code to convert things like cmd/ctrl+x to a cut event, but
                    // only operating on keys converted from winit, at a lower level of
                    // code, this doesn't happen on raw input we provide.
                    NativeMenuRawEvent::Cut => raw_input.events.push(egui::Event::Cut),
                    NativeMenuRawEvent::Copy => raw_input.events.push(egui::Event::Copy),
                    NativeMenuRawEvent::Paste(s) => raw_input.events.push(egui::Event::Paste(s)),

                    // Note that the key inputs we push to raw input don't need
                    // to match the real platform shortcuts, they just have to:
                    //
                    // 1. Match whatever egui is expecting so that they will
                    //    be used in text boxes (as a direct key input, see egui
                    //    `widgets/text_edit/builder` module)
                    //
                    // 2. Match one of our own shortcuts in `ui/shortcuts` module,
                    //    so we will respond to it.
                    //
                    // Therefore we use the "cmd+z/cmd+y" shortcuts for brevity.
                    //
                    // If there was a specific event for these we would use it,
                    // without that the best we can do is a keyboard input to
                    // trigger the correct effect.
                    NativeMenuRawEvent::Undo => {
                        press_and_release(raw_input, Key::Z, Modifiers::COMMAND)
                    }
                    NativeMenuRawEvent::Redo => {
                        press_and_release(raw_input, Key::Y, Modifiers::COMMAND)
                    }
                }
            }
            // Put it back
            self.native_menu = Some(native_menu);
        }
    }
}

fn press_and_release(raw_input: &mut egui::RawInput, key: Key, modifiers: Modifiers) {
    // Press then release the key, so tracking of whether keys are held down is
    // correct.
    raw_input.events.push(egui::Event::Key {
        key,
        physical_key: Some(Key::Z),
        pressed: true,
        repeat: false,
        modifiers,
    });
    raw_input.events.push(egui::Event::Key {
        key,
        physical_key: Some(Key::Z),
        pressed: false,
        repeat: false,
        modifiers,
    });
}
