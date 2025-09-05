use camino::Utf8PathBuf;

use crate::{
    data::maps::{Map, MapId},
    data::palette::Palette,
    data::tiles::layer_tiles::LayerId,
    data::tiles::tile_color::UserColor,
    data::tilesets::{Tileset, TilesetId},
    geom::{i32pos2::I32Pos2, u32size2::U32Size2},
    tiled::tiled_json::Tiled,
};

pub enum Action {
    /// Draw the current stamp at the specified center position
    Draw {
        map_id: MapId,
        pos: I32Pos2,
        /// True if this drawing operation is complete (e.g. drag stopped),
        /// false if still in progress (e.g. still dragging)
        complete: bool,
    },

    /// Erase the tile at the specified position (if it lies within the map)
    Erase {
        map_id: MapId,
        pos: I32Pos2,
        /// True if this drawing operation is complete (e.g. drag stopped),
        /// false if still in progress (e.g. still dragging)
        complete: bool,
    },

    /// Erase selected visible tiles (i.e. tiles within the selected areas, on selected, visible layers)
    EraseSelectedVisibleTiles { map_id: MapId },

    /// Update the tileset with specified id (if any) to be the specified tileset
    UpdateTileset { id: TilesetId, tileset: Tileset },

    /// Add a new tileset
    AddTileset { tileset: Tileset },

    /// Delete a tileset
    DeleteTileset { id: TilesetId },

    /// Replace the palette
    ReplacePalette { palette: Palette },

    /// Add a new map
    AddMap { map: Map },

    /// Append a map (and possibly tilesets and palette entries)
    /// from an imported Tiled map
    AppendTiledMap {
        tiled: Tiled,
        squash_layers: bool,
        prefer_relative_path: bool,
    },

    /// Update a map
    UpdateMap {
        map_id: MapId,
        name: String,
        map_size: U32Size2,
        tile_size: U32Size2,
        background_color: UserColor,
    },

    /// Delete a map
    DeleteMap { id: MapId },

    /// Set layer visibility
    SetLayerVisible {
        map_id: MapId,
        layer_id: LayerId,
        visible: bool,
    },

    /// Rename layer
    SetLayerName {
        map_id: MapId,
        layer_id: LayerId,
        name: String,
    },

    /// Rename and/or edit opacity of layer (if name and/or opacity match current values, no change will be made to data)
    EditLayer {
        map_id: MapId,
        layer_id: LayerId,
        name: String,
        opacity: Option<f32>,
    },

    /// Add layer
    AddLayer { map_id: MapId },

    /// Delete selected layers
    DeleteSelectedLayers { map_id: MapId },

    /// Merge selected layers on given map.
    /// This retains the top-most selected layer,
    /// and in each empty tile of that layer, inserts
    /// the tile from the highest layer that contains
    /// a non-empty tile (if any). The layers other
    /// than the top layer are then deleted.
    /// Has no effect if less than 2 layers are selected.
    MergeSelectedLayers { map_id: MapId },

    /// Move selected layers higher
    MoveSelectedLayersHigher { map_id: MapId },

    /// Move selected layers lower
    MoveSelectedLayersLower { map_id: MapId },

    /// Update data on save (e.g. to make tileset paths relative if requested)
    OnSave { path: Utf8PathBuf },
}

/// The result of performing an `Action`, in terms of whether it changed data, and
/// whether it should cause a new revision to be produced if any changes are pending.
pub struct ActionResult {
    /// True if this action has made changes to the data, whether or not
    /// the action should immediately produce a revision. Changes are
    /// accumulated until a revision point is reached.
    change: bool,
    /// True if this action is a possible revision point. If there are any
    /// accumulated changes to the data since the last revision, this will
    /// lead to a new revision being created, and accumulated changes being
    /// cleared.
    /// Note that it is possible for `revision` to be true and `change` to be
    /// false - this indicates that the action is a suitable point for a revision,
    /// and hasn't directly produced any changes itself, but there may still need to
    /// be a revision if there are pending changes from previous actions. For example
    /// this happens for the final action produced while dragging to draw, if
    /// an earlier stage of the drag produced changes, but the final stage of the drag
    /// (drag complete) does not itself produce changes. It still marks the right point
    /// to insert a revision with the changes from the previous actions.
    revision: bool,
}

impl ActionResult {
    pub fn new(change: bool, revision: bool) -> Self {
        Self { change, revision }
    }

    pub fn change(&self) -> bool {
        self.change
    }

    pub fn revision(&self) -> bool {
        self.revision
    }

    pub const NONE: ActionResult = ActionResult {
        change: false,
        revision: false,
    };

    pub const CHANGE_AND_REVISION: ActionResult = ActionResult {
        change: true,
        revision: true,
    };
}
