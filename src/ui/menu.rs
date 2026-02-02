use egui::{vec2, Color32, InnerResponse, Stroke, Style, Ui};

#[cfg(not(target_arch = "wasm32"))]
use crate::{app::App, data::mode::Mode, ui::egui_utils::unselectable_label};

fn set_menu_style(style: &mut Style) {
    style.spacing.button_padding = vec2(6.0, 0.0);
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
}

/// The menu bar goes well in a [`crate::TopBottomPanel::top`],
/// but can also be placed in a [`crate::Window`].
/// In the latter case you may want to wrap it in [`Frame`].
pub fn bar<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    ui.horizontal(|ui| {
        set_menu_style(ui.style_mut());

        // Take full width and fixed height:
        // let height = ui.spacing().interact_size.y;
        ui.set_min_size(vec2(ui.available_width(), 24.0));

        add_contents(ui)
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn add_file_menu(ui: &mut Ui, app: &mut App) {
    ui.menu_button("File", |ui| {
        {
            if ui.button("󰈔 New...").clicked() {
                app.check_data_loss_then_new_document();
            }

            if ui.button("󰝰 Open...").clicked() {
                app.check_data_loss_then_show_open_document_modal();
            }

            if ui.button("󰆓 Save").clicked() {
                app.show_save_document_modal();
            }

            if ui.button("󰳻 Save as...").clicked() {
                app.show_save_as_document_modal();
            }

            ui.menu_button("󱋡 Recent files...", |ui| {
                let mut path_to_open = None;
                for (index, path) in app.recent_paths.iter().enumerate() {
                    let name = format!(
                        "{}. {}",
                        index + 1,
                        path.file_name().unwrap_or(path.as_str())
                    );
                    if ui.button(name).on_hover_text(path.as_str()).clicked() {
                        path_to_open = Some(path.clone());
                    }
                }
                if let Some(path_to_open) = path_to_open {
                    app.check_data_loss_then_open_document_from_file_argument(path_to_open);
                }
            });

            if ui.button("󰋺 Import Palette...").clicked() {
                app.show_import_palette_modal();
            }

            if ui.button("󰈇 Export Palette...").clicked() {
                app.show_export_palette_modal();
            }

            if ui.button("󰋺 Import Tiled...").clicked() {
                app.pick_tiled_file_to_import();
            }

            if ui.button("󰈇 Export Tiled...").clicked() {
                app.show_export_tiled_modal();
            }

            if ui.button("󰈇 Export PNG...").clicked() {
                app.show_export_png_modal();
            }

            #[cfg(feature = "export-codegen-rs")]
            if ui.button("󰈇 Export Codegen Rust...").clicked() {
                app.show_export_codegen_file_modal();
            }
        }

        if ui.button("󰩈 Quit").clicked() {
            app.check_data_loss_then_quit(ui.ctx());
        }
    });
}

// No file menu on web
#[cfg(target_arch = "wasm32")]
fn add_file_menu(_ui: &mut Ui, _app: &mut MainApp) {}

pub fn menu_ui(ui: &mut Ui, app: &mut App) {
    bar(ui, |ui| {
        let is_macos = cfg!(target_os = "macos");

        // On macos, we can draw the menu over the title bar, we need to leave space for the
        // macos window buttons
        if is_macos {
            ui.add_space(72.0);
        } else {
            ui.add_space(16.0);
        }

        add_file_menu(ui, app);

        ui.menu_button("Edit", |ui| {
            let undo_clicked = ui
                .add_enabled(app.can_undo(), egui::Button::new("󰕌 Undo"))
                .clicked();

            let redo_clicked = ui
                .add_enabled(app.can_redo(), egui::Button::new("󰑎 Redo"))
                .clicked();

            if undo_clicked {
                app.undo();
            }

            if redo_clicked {
                app.redo();
            }

            if ui.button("󰒓 Application settings...").clicked() {
                app.show_application_settings_modal();
            }
        });

        ui.menu_button("View", |ui| {
            if ui.button("󱉶 Reset zoom").clicked() {
                app.reset_selected_map_zoom();
            }
        });

        if ui.button("Help...").clicked() {
            app.show_help_modal();
        }

        ui.add_space(16.0);
        unselectable_label(ui, "Mode:");
        ui.add_space(4.0);
        let mode = app.edit.mode;
        if ui.selectable_label(mode == Mode::Draw, "󰏫 Draw").clicked() {
            app.draw_mode();
        };
        if ui
            .selectable_label(mode == Mode::Erase, "󰇾 Erase")
            .clicked()
        {
            app.erase_mode();
        };
        if ui
            .selectable_label(mode == Mode::Select, "󰒅 Select")
            .clicked()
        {
            app.select_mode();
        };
    });
}
