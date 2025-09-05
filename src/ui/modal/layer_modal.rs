use egui::{Id, Modal, Slider, Ui};

use crate::{
    app::App,
    data::action::Action,
    data::modal::{ModalResult, ModalState},
    ui::egui_utils::singleline_focus_and_select,
    ui::theme::DEFAULT_THEME,
};

pub fn layer_modal_ui(ui: &mut Ui, app: &mut App) {
    let modal_to_apply = if let ModalState::Layer {
        ref mut name,
        ref mut opacity,
        ref mut result,
        ..
    } = app.edit.modal
    {
        Modal::new(Id::new("Layer Modal"))
            .frame(DEFAULT_THEME.modal_frame())
            .show(ui.ctx(), |ui| {
                ui.set_width(250.0);

                ui.heading("Edit Layer");
                ui.add_space(DEFAULT_THEME.modal_spacing);

                ui.label("Name");
                let r = singleline_focus_and_select(ui, name, *result == ModalResult::Init);

                if r.lost_focus() {
                    if r.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        *result = ModalResult::Apply;
                    } else if r.ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        *result = ModalResult::Cancel;
                    }
                }

                ui.add_space(DEFAULT_THEME.modal_spacing);

                match opacity.as_mut() {
                    Some(opacity_value) => {
                        let mut checked = true;
                        ui.checkbox(&mut checked, "Opacity enabled");
                        ui.add(
                            Slider::new(opacity_value, 0.0..=1.0)
                                .clamping(egui::SliderClamping::Always),
                        );
                        if !checked {
                            *opacity = None;
                        }
                    }
                    None => {
                        let mut checked = false;
                        ui.checkbox(&mut checked, "Opacity enabled");
                        if checked {
                            *opacity = Some(1.0);
                        }
                    }
                };

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

    if let Some(ModalState::Layer {
        map_id,
        layer_id,
        name,
        opacity,
        ..
    }) = modal_to_apply
    {
        let action = Action::EditLayer {
            map_id,
            layer_id,
            name: name.to_string(),
            opacity,
        };

        app.act(action);
    }
}
