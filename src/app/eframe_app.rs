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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested())
            && self.may_have_unsaved_changes()
            && !self.quit_requested
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_data_loss_modal(DataLossOperation::Quit);
        }

        // Make sure app state remains consistent
        self.apply_invariants();

        self.poll_and_handle_all_ipc_messages(ctx);

        let menu_frame = DEFAULT_THEME.base_100_frame(2);

        egui::TopBottomPanel::top("top_panel")
            .frame(menu_frame)
            .show(ctx, |ui| {
                menu_ui(ui, self);
            });

        let side_frame = DEFAULT_THEME.base_100_frame(16);

        egui::SidePanel::left("left_panel")
            .frame(side_frame)
            .resizable(true)
            .default_width(350.0)
            .min_width(275.0)
            .max_width(750.0)
            .show(ctx, |ui| {
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
            .show(ctx, |ui| {
                map_ui(ui, self);

                match self.edit.modal {
                    ModalState::None => consume_shortcuts(ctx, self),
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

        self.toasts.show(ctx);
    }
}
