use egui::{vec2, Id, Modal, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    app::App,
    data::modal::{ModalResult, ModalState},
    data::palette::palette_index,
    data::tiles::tile_color::UserColor,
    ui::egui_utils::{
        enabled_square_button, sized_main_dir_cross_align_center, square_button,
        unselectable_label, unselectable_label_strong, user_color_edit_button,
    },
    ui::theme::DEFAULT_THEME,
};

const TABLE_HEIGHT: f32 = 250.0;

pub fn palette_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::Palette {
        ref mut selected_index,
        ref mut palette,
        ref mut scroll_to_row,
        ref mut edit_color_as_text,
        ref mut result,
    } = &mut app.edit.modal
    {
        selected_index.limit_to_palette(palette);

        Modal::new(Id::new("Palette Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Palette");
                ui.add_space(DEFAULT_THEME.modal_spacing);

                let size = vec2(ui.available_width(), TABLE_HEIGHT);

                let table_response = sized_main_dir_cross_align_center(ui, size, |ui| {
                    let mut table = TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(20.0))
                        .column(Column::remainder().clip(true))
                        .min_scrolled_height(0.0)
                        .max_scroll_height(99999999.0)
                        .sense(egui::Sense::click());

                    if let Some(row) = scroll_to_row {
                        table = table.scroll_to_row(*row, None);
                        *scroll_to_row = None;
                    }

                    table
                        .header(DEFAULT_THEME.row_height, |mut header| {
                            header.col(|ui| {
                                unselectable_label_strong(ui, "#");
                            });
                            header.col(|ui| {
                                unselectable_label_strong(ui, "Color");
                            });
                        })
                        .body(|mut body| {
                            // Display layers reversed so that the layer with the highest index, which is
                            // drawn over all other layers, is on first row of table, i.e. on the "top"
                            for (index, color) in palette.colors_mut().enumerate() {
                                let selected = selected_index.index() == index as u32;
                                body.row(DEFAULT_THEME.row_height, |mut row| {
                                    row.set_selected(selected);
                                    row.col(|ui| {
                                        unselectable_label(ui, index.to_string());
                                    });
                                    row.col(|ui| {
                                        let text = color.as_css_string();
                                        user_color_edit_button(ui, color, edit_color_as_text);
                                        ui.add_space(6.0);
                                        unselectable_label(
                                            ui,
                                            RichText::new(text)
                                                .text_style(egui::TextStyle::Monospace)
                                                .color(DEFAULT_THEME.base_subcontent),
                                        );
                                    });
                                    if row.response().clicked() {
                                        *selected_index = palette_index(index as u32);
                                    }
                                });
                            }
                        });
                });

                // Push buttons to the bottom of the maximum table rect, rather than
                // having them always just under the bottom row - only makes a difference
                // when all rows can be displayed without
                if table_response.response.rect.height() < TABLE_HEIGHT {
                    ui.add_space(TABLE_HEIGHT - table_response.response.rect.height());
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if square_button(ui, "󰐕").clicked() {
                        *selected_index = palette.insert_color(*selected_index, UserColor::WHITE);
                        *scroll_to_row = Some(selected_index.index() as usize);
                    }

                    if square_button(ui, "󰍴").clicked() {
                        *selected_index = palette.delete_color(*selected_index);
                        *scroll_to_row = Some(selected_index.index() as usize);
                    }

                    if enabled_square_button(
                        ui,
                        palette.can_move_color_previous(*selected_index),
                        "󰁝",
                    )
                    .clicked()
                    {
                        *selected_index = palette.move_color_previous(*selected_index);
                        *scroll_to_row = Some(selected_index.index() as usize);
                    }

                    if enabled_square_button(ui, palette.can_move_color_next(*selected_index), "󰁅")
                        .clicked()
                    {
                        *selected_index = palette.move_color_next(*selected_index);
                        *scroll_to_row = Some(selected_index.index() as usize);
                    }
                });
                ui.add_space(DEFAULT_THEME.modal_spacing);
                ui.separator();
                ui.add_space(DEFAULT_THEME.modal_spacing);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Apply").clicked() {
                            *result = ModalResult::Apply;
                        }
                        if ui.button("Cancel").clicked() {
                            *result = ModalResult::Cancel;
                        }
                    },
                );
            });

        app.progress_modal_state()
    } else {
        None
    };

    if let Some(ModalState::Palette { palette, .. }) = modal_to_apply {
        app.prompt_to_replace_palette(palette);
    }
}
