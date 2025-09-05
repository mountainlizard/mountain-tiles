use egui::{
    color_picker::show_color_at, Popup, PopupCloseBehavior, Response, Sense, Slider, StrokeKind,
    Ui, WidgetInfo, WidgetType,
};

use crate::data::tiles::tile_color::UserColor;

fn color_button(ui: &mut Ui, color: UserColor, open: bool) -> Response {
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::new(WidgetType::ColorButton));

    if ui.is_rect_visible(rect) {
        let visuals = if open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&response)
        };
        let rect = rect.expand(visuals.expansion);

        let stroke_width = 1.0;
        show_color_at(
            ui.painter(),
            color.as_premultiplied_color32(),
            rect.shrink(stroke_width),
        );

        let corner_radius = visuals.corner_radius.at_most(2); // Can't do more rounding because the background grid doesn't do any rounding
        ui.painter().rect_stroke(
            rect,
            corner_radius,
            (stroke_width, visuals.bg_fill), // Using fill for stroke is intentional, because default style has no border
            StrokeKind::Inside,
        );
    }

    response
}

pub fn color_edit_popup(ui: &mut Ui, color: &mut UserColor, as_text: &mut String) -> bool {
    let initial = *color;
    ui.label("Red");
    ui.add(Slider::new(color.r_mut(), 0..=255).clamping(egui::SliderClamping::Always));

    ui.label("Green");
    ui.add(Slider::new(color.g_mut(), 0..=255).clamping(egui::SliderClamping::Always));

    ui.label("Blue");
    ui.add(Slider::new(color.b_mut(), 0..=255).clamping(egui::SliderClamping::Always));

    ui.label("Alpha");
    ui.add(Slider::new(color.a_mut(), 0..=255).clamping(egui::SliderClamping::Always));

    ui.label("CSS");
    let text_changed = ui.text_edit_singleline(as_text).changed();
    if text_changed {
        if let Ok(parsed) = csscolorparser::parse(as_text) {
            *color = parsed.into();
        }
    }

    let changed = initial != *color;
    if changed && !text_changed {
        *as_text = color.as_css_string();
    }
    changed
}

pub fn color_edit_button(ui: &mut Ui, color: &mut UserColor, as_text: &mut String) -> Response {
    let popup_id = ui.auto_id_with("popup");
    let open = Popup::is_id_open(ui.ctx(), popup_id);

    let mut button_response = color_button(ui, *color, open);

    // When popup opens, set text to css string
    if button_response.clicked() {
        *as_text = color.as_css_string();
    }

    if ui.style().explanation_tooltips {
        button_response = button_response.on_hover_text("Click to edit color");
    }

    const COLOR_SLIDER_WIDTH: f32 = 275.0;

    Popup::menu(&button_response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = COLOR_SLIDER_WIDTH;
            if color_edit_popup(ui, color, as_text) {
                button_response.mark_changed();
            }
        });

    button_response
}
