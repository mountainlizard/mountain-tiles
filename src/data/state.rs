use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use camino::Utf8PathBuf;
use log::info;

use crate::{
    data::palette::Palette,
    data::tiles::Tile,
    data::tilesets::TilesetId,
    data::{
        action::ActionResult,
        file_format::{confirm_format, FileContents, FileFormat},
        maps::Maps,
        resources::{Resources, TileResourceLocation, TileResourceUse},
    },
    undo::Undoable,
};

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq)]
pub struct State {
    pub resources: Resources,
    pub maps: Maps,

    /// This is incremented whenever a new revision is made, indicating that
    /// there should be a new undo state, and that the data has unsaved changes.
    /// Not every change to the data produces a new revision - for example, dragging
    /// to draw multiple tiles on a map will change the data at each stage of the drag, but only
    /// produce a new revision when the drag is complete. While there are changes to
    /// the data that require a new revision at an appropriate point, [`State::pending_change`]
    /// will be true.
    #[serde(skip)]
    revision: u64,

    /// This tracks whether there have been any changes to the [`State`] that have
    /// not been cleared by producing a new revision.
    /// This is set to true whenever a change is made.
    /// It is set to false when a revision is produced (by incrementing revision field)
    #[serde(skip)]
    pending_change: bool,
}

impl Undoable for State {
    fn has_changed_from(&self, previous: &Self) -> bool {
        self.revision != previous.revision
    }
}

impl State {
    pub fn add_action_result(&mut self, result: &ActionResult) {
        // Accumulate changes
        if result.change() {
            self.pending_change = true;
        }

        // If we have reached a revision point, produce a revision if there are any pending changes
        if result.revision() && self.pending_change {
            self.add_revision();
        }
    }

    fn add_revision(&mut self) {
        self.revision += 1;
        self.pending_change = false;
    }

    pub fn can_undo(&self) -> bool {
        !self.pending_change
    }

    pub fn can_redo(&self) -> bool {
        !self.pending_change
    }

    pub fn find_use_of_resource<P>(&self, predicate: P) -> Option<TileResourceUse>
    where
        P: Fn(&Option<Tile>) -> bool,
    {
        let mut tile_count: usize = 0;
        let mut locations = vec![];
        for map in self.maps.iter() {
            for layer in map.tiles.layers() {
                let layer_tile_count = layer.tiles_iter().filter(|t| predicate(t)).count();
                if layer_tile_count > 0 {
                    locations.push(TileResourceLocation {
                        map_name: map.name(),
                        layer_name: layer.name(),
                    });
                    tile_count += layer_tile_count;
                }
            }
        }
        if tile_count > 0 {
            Some(TileResourceUse {
                locations,
                tile_count,
            })
        } else {
            None
        }
    }

    pub fn find_use_of_tileset(&self, tileset_id: TilesetId) -> Option<TileResourceUse> {
        self.find_use_of_resource(|t| t.is_some_and(|t| t.source.tileset_id == tileset_id))
    }

    pub fn find_use_of_colors_outside_palette(&self, palette: &Palette) -> Option<TileResourceUse> {
        self.find_use_of_resource(|t| t.is_some_and(|t| !palette.is_tilecolor_available(&t.color)))
    }

    pub fn clear_tiles_with_tileset(&mut self, tileset_id: TilesetId) -> bool {
        let mut change = false;
        for tiles in self.maps.iter_mut().map(|map| &mut map.tiles) {
            change |= tiles.clear_tiles_with_tileset(tileset_id);
        }
        change
    }

    pub fn clear_tiles_outside_palette(&mut self, palette: &Palette) {
        for map in self.maps.iter_mut() {
            map.tiles.clear_tiles_outside_palette(palette);
        }
    }

    pub fn from_path(path: Utf8PathBuf) -> eyre::Result<State> {
        // First confirm file has correct format - this will give better errors
        // on unsupported files than trying to load as [`FileContents`] directly
        let format = confirm_format(path.clone())?;

        // Note that in future we might need to adapt loading for format, at the moment
        // we can read all formats using the current data model (e.g. v1 only adds
        // possible enum values to v0, so we can load v0 JSON as v1 data model)
        info!("Loading format '{}'", format);

        let file = File::open(path.clone())?;
        let buf_reader = BufReader::new(file);
        let file_contents: FileContents = serde_json::from_reader(buf_reader)?;

        Ok(file_contents.state)
    }

    pub fn save_to_path(&mut self, path: Utf8PathBuf) -> eyre::Result<()> {
        let file = File::create(path.clone())?;
        let buf_writer = BufWriter::new(file);

        // We'll save a clone of the map, so we can prepare it for saving
        let mut state = self.clone();
        state.on_save(path.clone());

        let file_contents = FileContents {
            state,
            format: FileFormat::CURRENT,
        };

        serde_json::to_writer(buf_writer, &file_contents)?;
        Ok(())
    }

    pub fn on_save(&mut self, path: Utf8PathBuf) {
        self.resources.tilesets.on_save(path);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn defaults_make_sense() -> eyre::Result<()> {
        let state = State::default();
        println!("{}", serde_json::to_string_pretty(&state)?);

        // assert_eq!(resources.tilesets.len(), 0);
        // assert_eq!(resources.palette.len(), 1);
        // assert_eq!(
        //     resources.palette.color_option(PaletteIndex::new(0)),
        //     Some(UserColor::WHITE)
        // );

        Ok(())
    }
}
