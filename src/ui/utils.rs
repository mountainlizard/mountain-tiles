use egui::{Response, Ui};

use crate::{data::tiles::tile_color::UserColor, ui::color_edit::color_edit_button};

pub fn optional_color_ui(
    ui: &mut Ui,
    color_option: &mut Option<UserColor>,
    color_name: &str,
    default_color: &mut UserColor,
    color_as_text: &mut String,
) {
    match color_option.as_mut() {
        Some(color) => {
            let mut checked = true;
            ui.checkbox(&mut checked, format!("{} enabled", color_name));
            user_color_edit_button(ui, color, color_as_text);
            if !checked {
                *default_color = *color;
                *color_option = None;
            }
        }
        None => {
            let mut checked = false;
            ui.checkbox(&mut checked, format!("{} enabled", color_name));
            if checked {
                *color_option = Some(*default_color);
            }
        }
    };
}

pub fn user_color_edit_button(
    ui: &mut Ui,
    color: &mut UserColor,
    as_text: &mut String,
) -> Response {
    color_edit_button(ui, color, as_text)
}
