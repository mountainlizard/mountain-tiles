use crate::{
    app::App, data::mode::Mode, data::stamp::Stamp, data::tiles::Tiles, geom::transform::Transform,
};

impl App {
    pub fn transform(&mut self, transform: Transform) {
        if self.edit.mode == Mode::Draw {
            self.edit.stamp = self.edit.stamp.with_transform(transform);
        }
    }

    pub fn clear_transform(&mut self) {
        self.transform(self.edit.stamp.transform.inverse());
    }

    pub fn stamp_from_tileset<T: Tiles>(&mut self, tiles: &T) {
        if let (Some(selected_tileset_id), Some(selection)) = (
            self.edit.selected_tileset_id,
            self.edit.selected_tileset_tile_selection(),
        ) {
            self.edit.stamp =
                Stamp::from_selected_tiles(tiles, selection, Some(self.selected_palette_color()));
            // Clear the selections for all tilesets other than the one we just made stamp from (so that
            // tileset selections mirror the stamp)
            for (id, selection) in self.edit.tileset_tile_selection_by_id.iter_mut() {
                if id != &selected_tileset_id {
                    selection.clear();
                }
            }
        }
    }
}
