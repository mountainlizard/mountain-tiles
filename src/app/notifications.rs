use egui::WidgetText;
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::app::App;

impl App {
    pub fn success(&mut self, caption: impl Into<WidgetText>) {
        self.toasts.add(
            Toast::default()
                .kind(ToastKind::Success)
                .text(caption)
                .options(ToastOptions::default().duration_in_seconds(2.0)),
        );
    }

    pub fn info(&mut self, caption: impl Into<WidgetText>) {
        self.toasts.add(
            Toast::default()
                .kind(ToastKind::Info)
                .text(caption)
                .options(ToastOptions::default().duration_in_seconds(4.0)),
        );
    }
}
