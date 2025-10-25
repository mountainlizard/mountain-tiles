use egui::{vec2, Color32, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    app::App,
    data::tiles::Tiles,
    data::{action::Action, maps::MapId},
    ui::egui_utils::{
        enabled_square_button, sized_button, sized_main_dir_cross_align_center, square_button,
        unselectable_label, unselectable_label_strong,
    },
    ui::theme::DEFAULT_THEME,
};

pub enum LayerOperation {
    Action(Action),
    ShowLayerModal { map_id: MapId, layer_index: usize },
}

pub fn layers_ui(ui: &mut Ui, app: &mut App) {
    let table_height = ui.available_height() - 36.0;
    let size = vec2(ui.available_width(), table_height);

    let mut operations = vec![];

    if let Some(mut me) = app.selected_map_editing_mut() {
        let map_id = me.map.id();
        let stamp_layers = me.layer_index_to_stamp_layer_index();
        let table_response = sized_main_dir_cross_align_center(ui, size, |ui| {
            let shift = ui.input(|i| i.modifiers.shift);
            let command = ui.input(|i| i.modifiers.command);

            let table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(20.0))
                .column(Column::remainder().clip(true))
                .column(Column::exact(40.0))
                .column(Column::exact(24.0))
                .column(Column::exact(40.0))
                .min_scrolled_height(0.0)
                .max_scroll_height(99999999.0)
                .sense(egui::Sense::click());

            table
                .header(DEFAULT_THEME.row_height, |mut header| {
                    header.col(|ui| {
                        unselectable_label_strong(ui, "#");
                    });
                    header.col(|ui| {
                        unselectable_label_strong(ui, "Layer name");
                    });
                    header.col(|ui| {
                        unselectable_label_strong(ui, "");
                    });
                    header.col(|ui| {
                        unselectable_label_strong(ui, "");
                    });
                    header.col(|ui| {
                        unselectable_label_strong(ui, "Visible");
                    });
                })
                .body(|mut body| {
                    // Display layers reversed so that the layer with the highest index, which is
                    // drawn over all other layers, is on first row of table, i.e. on the "top"
                    for layer_index in 0..me.map.tiles().layer_count() {
                        // let selected = app.state.view.selected_layer_index == layer_index;
                        let selected = me
                            .map
                            .tiles()
                            .layer_id(layer_index)
                            .map(|id| me.edit.layer_selection.is_selected(id))
                            .unwrap_or(false);
                        body.row(DEFAULT_THEME.row_height, |mut row| {
                            row.set_selected(selected);
                            row.col(|ui| {
                                unselectable_label(ui, format!("{}", layer_index + 1));
                            });
                            row.col(|ui| {
                                if let Some(layer_name) = me.map.tiles().layer_name(layer_index) {
                                    unselectable_label(ui, layer_name);
                                }
                            });
                            row.col(|ui| {
                                if let Some(Some(stamp_layer_index)) = stamp_layers.get(layer_index)
                                {
                                    let text = &format!("(󰴹{})", (stamp_layer_index + 1));
                                    unselectable_label(ui, text);
                                }
                            });
                            row.col(|ui| {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                    Color32::TRANSPARENT;
                                if sized_button(ui, vec2(24.0, 24.0), "󰏫").clicked() {
                                    operations.push(LayerOperation::ShowLayerModal {
                                        map_id,
                                        layer_index,
                                    });
                                    // app.show_layer_modal(map, layer_index);
                                };
                            });

                            row.col(|ui| {
                                if let (Some(visible_initial), Some(id)) = (
                                    me.map.tiles().layer_visible(layer_index),
                                    me.map.tiles().layer_id(layer_index),
                                ) {
                                    // checkbox is a little awkward - it needs a mutable bool
                                    // We make one from initial state, and if it gets changed then
                                    // we fire of an action to actually do the work
                                    let mut visible = visible_initial;
                                    ui.add_space(12.0);
                                    ui.checkbox(&mut visible, "");
                                    if visible != visible_initial {
                                        operations.push(LayerOperation::Action(
                                            Action::SetLayerVisible {
                                                map_id: me.map.id(),
                                                layer_id: id,
                                                visible,
                                            },
                                        ));
                                    }
                                }
                            });
                            if row.response().clicked() {
                                if let Some(id) = me.map.tiles().layer_id(layer_index) {
                                    me.edit.layer_selection.update_from_click(
                                        me.map.tiles(),
                                        id,
                                        shift,
                                        command,
                                    );
                                }
                            }
                            if row.response().double_clicked() {
                                operations.push(LayerOperation::ShowLayerModal {
                                    map_id,
                                    layer_index,
                                });
                            }
                        });
                    }
                });
        });
        // Push buttons to the bottom of the maximum table rect, rather than
        // having them always just under the bottom row - only makes a difference
        // when all rows can be displayed without
        if table_response.response.rect.height() < table_height {
            ui.add_space(table_height - table_response.response.rect.height());
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if square_button(ui, "󰐕").clicked() {
                operations.push(LayerOperation::Action(Action::AddLayer { map_id }));
            }

            if enabled_square_button(ui, me.can_delete_selected_layers(), "󰍴").clicked() {
                operations.push(LayerOperation::Action(Action::DeleteSelectedLayers {
                    map_id,
                }));
            }

            if enabled_square_button(ui, me.can_merge_selected_layers(), "󰘭 Merge").clicked() {
                operations.push(LayerOperation::Action(Action::MergeSelectedLayers {
                    map_id,
                }));
            }

            if enabled_square_button(ui, me.can_move_selected_layers_higher(), "󰁝").clicked() {
                operations.push(LayerOperation::Action(Action::MoveSelectedLayersHigher {
                    map_id: me.map.id(),
                }));
            }

            if enabled_square_button(ui, me.can_move_selected_layers_lower(), "󰁅").clicked() {
                operations.push(LayerOperation::Action(Action::MoveSelectedLayersLower {
                    map_id: me.map.id(),
                }));
            }
        });
    };

    // Carry out operations here, we can't do them above because app is already borrowed
    for operation in operations.into_iter() {
        match operation {
            LayerOperation::Action(action) => app.act(action),
            LayerOperation::ShowLayerModal {
                map_id,
                layer_index,
            } => app.show_layer_modal(map_id, layer_index),
        }
    }
}
