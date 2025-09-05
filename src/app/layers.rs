use crate::{app::App, selection::ShiftDirection};

impl App {
    pub fn selected_map_toggle_select_all_layers(&mut self) {
        if let Some(me) = self.selected_map_editing_mut() {
            me.edit.layer_selection.toggle_select_all(me.map.tiles());
        }
    }

    /// This just moves the layer selection for the selected map - not the layers themselves,
    /// just the pattern of which layers are selected.
    /// If higher is true, for each selected layer, the layer above it
    /// is selected instead. If higher is false, the layers underneath are selected.
    pub fn shift_layer_selection(&mut self, direction: ShiftDirection) {
        if let Some(me) = self.selected_map_editing_mut() {
            me.edit
                .layer_selection
                .shift_selection(me.map.tiles(), direction);
        }
    }

    pub fn select_first_layer(&mut self) {
        if let Some(me) = self.selected_map_editing_mut() {
            me.edit.layer_selection.clear();
            if let Some(layer) = me.map.tiles().first_layer() {
                me.edit.layer_selection.select_only(layer.id());
            }
        }
    }
}
