#[cfg(target_os = "macos")]
use egui::{Key, Modifiers};

#[cfg(target_os = "macos")]
use crate::{app::App, ui::native_menu::NativeMenuRawEvent};

#[cfg(target_os = "macos")]
impl App {
    /// Poll for native menu events that translate into raw input,
    /// for example cut/copy/paste events, and add them to the
    /// [`egui::RawInput`] to be processed by the UI.
    pub(crate) fn poll_and_handle_native_menu_raw_events(
        &mut self,
        raw_input: &mut egui::RawInput,
    ) {
        // Take native menu out so we can use it without holding self
        if let Some(native_menu) = self.native_menu.take() {
            while let Ok(event) = native_menu.rx_raw.try_recv() {
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
                    NativeMenuRawEvent::SelectAll => {
                        press_and_release(raw_input, Key::A, Modifiers::COMMAND);
                    }
                }
            }
            // Put it back
            self.native_menu = Some(native_menu);
        }
    }
}

#[cfg(target_os = "macos")]
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
