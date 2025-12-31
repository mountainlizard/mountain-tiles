use egui::{Id, ImageSource, Key, KeyboardShortcut, Modal, Modifiers, Sense, TextureOptions, Ui};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    ui::theme::DEFAULT_THEME,
};

const INSTRUCTIONS: ImageSource<'static> = egui::include_image!("../../../assets/instructions.png");
const BORDER: f32 = 128.0;

const CLOSE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::H,
};

const CLOSE_SHORTCUT_ALT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Escape,
};

pub fn help_modal_ui(ui: &mut Ui, app: &mut App) {
    if let ModalState::Help { ref mut result } = &mut app.edit.modal {
        let screen_rect = ui.ctx().content_rect();
        let modal_max_width = screen_rect.width() - BORDER;
        let modal_max_height = screen_rect.height() - BORDER;
        let modal_size = modal_max_width.min(modal_max_height);

        let modal_response =
            Modal::new(Id::new("Help Modal"))
                .frame(DEFAULT_THEME.modal_frame())
                .show(ui.ctx(), |ui| {
                    ui.set_width(modal_size);
                    ui.set_height(modal_size);

                    if ui.add(
                    egui::Image::new(INSTRUCTIONS)
                        .alt_text("Instructions for application, with shortcuts and mouse controls")
                        .texture_options(TextureOptions::NEAREST).sense(Sense::click()),
                ).clicked() {
                    *result = ModalResult::Apply;
                }

                    ui.ctx().input_mut(|i| {
                        if i.consume_shortcut(&CLOSE_SHORTCUT)
                            || i.consume_shortcut(&CLOSE_SHORTCUT_ALT)
                        {
                            *result = ModalResult::Apply;
                        }
                    });
                });

        if modal_response.response.clicked() || modal_response.backdrop_response.clicked() {
            *result = ModalResult::Apply;
        }

        app.progress_modal_state();
    }
}
