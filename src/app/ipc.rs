use egui::{UserAttentionType, ViewportCommand};

use crate::{app::App, instance::IpcMessage};

impl App {
    fn handle_ipc_message(&mut self, message: IpcMessage) {
        match message {
            IpcMessage::FileOpen { filepath } => {
                self.check_data_loss_then_open_document_from_file_argument(filepath)
            }
        }
    }

    fn poll_and_handle_ipc_message(&mut self) -> bool {
        if let Some(ref mut l) = self.ipc_listener {
            if let Some(message) = l.poll_recv() {
                self.handle_ipc_message(message);
                return true;
            }
        }
        false
    }

    /// Check for IPC messages, handle as many as are queued, and if any are handled,
    /// request user attention to the window
    pub(super) fn poll_and_handle_all_ipc_messages(&mut self, ctx: &egui::Context) -> bool {
        let mut message = false;
        while self.poll_and_handle_ipc_message() {
            message = true;
        }
        if message {
            // Use both methods of requesting attention to the window.
            // Wayland currently ignores focus but at least on KDE gives some indication
            // on request attention that window is active (shows taskbar and highlights icon).
            // Windows and at least some X11 desktops e.g. RPi seem to ignore request attention,
            // but respond to focus.
            // So far no platforms seem to support both commands, hopefully if any do they will
            // not respond badly to both being called.
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            ctx.send_viewport_cmd(ViewportCommand::RequestUserAttention(
                UserAttentionType::Critical,
            ));
        }
        message
    }
}
