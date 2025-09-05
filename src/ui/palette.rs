use egui::{vec2, Response, Sense, StrokeKind, Ui, Vec2};

use crate::{
    app::App, data::palette::palette_index, data::tiles::tile_color::UserColor,
    ui::theme::DEFAULT_THEME,
};

const BUTTON_SIZE: Vec2 = vec2(16.0, 16.0);

pub fn color_edit_ui(ui: &mut Ui, color: &UserColor, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(BUTTON_SIZE, Sense::click());
    if ui.is_rect_visible(rect) {
        let corner_radius = ui.visuals().widgets.inactive.corner_radius;
        if selected {
            ui.painter()
                .rect_filled(rect.shrink(3.0), 0.0, color.as_premultiplied_color32());
        } else {
            ui.painter()
                .rect_filled(rect, corner_radius, color.as_premultiplied_color32());
        }
        if selected {
            ui.painter().rect_stroke(
                rect,
                corner_radius,
                DEFAULT_THEME.selected_stroke,
                StrokeKind::Inside,
            );
        }
    }
    response
}

pub fn palette_ui(ui: &mut Ui, app: &mut App) {
    let selected_index = app.edit.selected_palette_index();
    ui.horizontal_wrapped(|ui| {
        ui.style_mut().spacing.item_spacing.x = 4.0;
        ui.style_mut().spacing.item_spacing.y = 1.0;
        let mut new_index = None;
        for (index, color) in app.state.resources.palette().colors().enumerate() {
            let index = index as u32;
            if color_edit_ui(ui, color, index == selected_index.index()).clicked() {
                new_index = Some(palette_index(index));
            }
        }
        if let Some(index) = new_index {
            app.select_palette_index(index);
        }
        if ui.button("󰏫...").clicked() {
            app.show_current_palette_modal();
        }
    });
}
