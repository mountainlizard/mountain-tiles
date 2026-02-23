use std::time::Duration;

use egui::WidgetText;

use crate::app::App;

impl App {
    pub fn success(&mut self, caption: impl Into<WidgetText>) {
        self.toasts
            .success(caption)
            .duration(Duration::from_secs(2));
    }
}
