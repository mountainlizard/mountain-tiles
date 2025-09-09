use std::{fmt::Display, slice::Iter};

use camino::Utf8PathBuf;
use egui::ahash::HashSet;
use log::info;

use crate::{
    data::tiles::{tile_color::UserColor, TileSource},
    geom::u32size2::{u32size2, U32Size2},
    selection::{Selectable, SelectableDefault},
};

/// This is unique within a [`Tilesets`], and persists for a
/// given [`Tileset`] even if edited (e.g. image changed). This can be used
/// to track a tileset externally, e.g. to pick a tileset in a tile, or
/// to use to track which tileset is selected
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TilesetId(u32);

impl TilesetId {
    pub const ONE: TilesetId = TilesetId(1);

    fn next(&self) -> TilesetId {
        TilesetId(self.0 + 1)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for TilesetId {
    fn default() -> Self {
        Self::ONE
    }
}

#[derive(
    Debug, Copy, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash, Default,
)]
pub enum TilesetMode {
    #[default]
    Direct,
    TransparentBackground {
        background: UserColor,
    },
}

impl TilesetMode {
    pub fn description(&self) -> &str {
        match self {
            TilesetMode::Direct => "Use image directly",
            TilesetMode::TransparentBackground { .. } => "Use transparent color",
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
pub struct Tileset {
    id: TilesetId,

    /// The name of the tileset, for display in the UI
    pub name: String,

    /// The path to the image file used to produce the tileset texture.
    /// This can be either an absolute path, or relative to the location
    /// of the file the project is stored in (if the project has not been
    /// saved, this will instead be relative to the working directory
    /// of the application).
    /// TODO: Behaviour on web?
    pub path: Utf8PathBuf,

    /// The size of each tile in pixels. Note that this is overridden by the
    /// map's own tile size when used in a normal map, but is used for
    /// displaying tilesets in the UI to allow selecting tiles.
    pub tile_size: U32Size2,

    /// This is the size of the tileset measured in tiles -
    /// e.g. if the image contains 16 columns of tiles (width) and
    /// 8 rows of tiles (height), this would be `u32size2(16, 8)`
    /// This is used to produce UV coordinates to use parts of the
    /// tileset image as tile textures.
    pub size_in_tiles: U32Size2,

    /// The mode for the tileset - this determines how the tileset is
    /// used and processed, for example by treating a particular color
    /// as transparent.
    #[serde(default)]
    pub mode: TilesetMode,

    /// The foreground color to use when displaying this tileset directly -
    /// has no effect on tiles when used in a map.
    pub foreground: Option<UserColor>,

    /// The background color to use when displaying this tileset directly -
    /// has no effect on tiles when used in a map.
    pub background: Option<UserColor>,

    /// If this is true, then the tileset prefers use of a relative
    /// path. If the `path` is absolute, then if possible then when
    /// the tileset is saved, the `path` will be converted to one
    /// relative to the file the tileset is saved to (either
    /// as an individual tileset, or as part of a map, etc.)
    pub prefer_relative_path: bool,
}

impl Display for Tileset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Default for Tileset {
    fn default() -> Self {
        Self {
            id: TilesetId::ONE,
            name: String::new(),
            path: Utf8PathBuf::new(),
            tile_size: u32size2(8, 8),
            size_in_tiles: u32size2(16, 16),
            mode: TilesetMode::Direct,
            foreground: None,
            background: None,
            prefer_relative_path: true,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Tilesets {
    tilesets: Vec<Tileset>,
    next_tileset_id: TilesetId,
}

impl Selectable<TilesetId> for Tilesets {
    fn all_ids(&self) -> HashSet<TilesetId> {
        self.tilesets.iter().map(|t| t.id()).collect()
    }
    fn contains_id(&self, id: &TilesetId) -> bool {
        self.tilesets.iter().any(|l| l.id == *id)
    }
}

impl SelectableDefault<TilesetId> for Tilesets {
    fn default_id(&self) -> Option<TilesetId> {
        self.tilesets.first().map(|t| t.id())
    }
}

impl Tileset {
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: TilesetId,
        name: String,
        path: Utf8PathBuf,
        tile_size: U32Size2,
        size_in_tiles: U32Size2,
        mode: TilesetMode,
        foreground: Option<UserColor>,
        background: Option<UserColor>,
        prefer_relative_path: bool,
    ) -> Self {
        Self {
            id,
            name,
            path,
            tile_size,
            size_in_tiles,
            mode,
            foreground,
            background,
            prefer_relative_path,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_default_id(
        name: String,
        path: Utf8PathBuf,
        tile_size: U32Size2,
        size_in_tiles: U32Size2,
        mode: TilesetMode,
        foreground: Option<UserColor>,
        background: Option<UserColor>,
        prefer_relative_path: bool,
    ) -> Self {
        Self {
            id: Default::default(),
            name,
            path,
            tile_size,
            size_in_tiles,
            mode,
            foreground,
            background,
            prefer_relative_path,
        }
    }

    pub fn id(&self) -> TilesetId {
        self.id
    }

    fn on_save(&mut self, path: Utf8PathBuf) {
        // Note that path is the file we've been saved to, we want the directory containing this to
        // use as the base_dir for any relative paths
        let mut base_dir = path.clone();
        base_dir.pop();

        info!(
            "Tileset {:?}, update_save_path({}), base_dir {}",
            self, path, base_dir
        );

        if self.prefer_relative_path && self.path.is_absolute() {
            if let Some(relative_path) =
                pathdiff::diff_utf8_paths(self.path.clone(), base_dir.clone())
            {
                info!(
                    "...prefers relative path, updating absolute path to {}",
                    relative_path
                );
                self.path = relative_path;
            } else {
                info!("...prefers relative path, pathdiff could not find relative path from base_dir {} to tileset path {}", base_dir, self.path);
            }
        } else if self.prefer_relative_path {
            info!("...prefers relative path but is already relative, leaving path unaltered");
        } else {
            info!("...does not prefer relative path, leaving path unaltered");
        }
    }
}

impl Tilesets {
    pub fn find_or_insert_matching_tileset(&mut self, tileset: Tileset) -> TilesetId {
        // TODO: Better comparison - just uses the filename, should use the actual path
        if let Some(existing) = self.tilesets.iter().find(|existing| {
            existing.path.file_name() == tileset.path.file_name()
                && existing.tile_size == tileset.tile_size
                && existing.size_in_tiles == tileset.size_in_tiles
        }) {
            existing.id()
        } else {
            self.push_tileset(tileset)
        }
    }

    pub fn builtin_tilesets() -> Tilesets {
        Tilesets {
            tilesets: vec![Tileset {
                id: TilesetId::ONE,
                name: "mountain-tiles".into(),
                path: "mountain-tiles".into(),
                tile_size: u32size2(8, 8),
                size_in_tiles: u32size2(16, 16),
                mode: TilesetMode::Direct,
                foreground: Some(UserColor::WHITE),
                background: Some(UserColor::BLACK),
                prefer_relative_path: false,
            }],
            next_tileset_id: TilesetId::ONE.next(),
        }
    }

    pub fn new() -> Self {
        Self {
            tilesets: vec![],
            next_tileset_id: TilesetId::ONE,
        }
    }

    pub fn len(&self) -> usize {
        self.tilesets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tilesets.is_empty()
    }

    pub fn push_tileset(&mut self, mut tileset: Tileset) -> TilesetId {
        let id = self.next_tileset_id;
        self.next_tileset_id = id.next();
        tileset.id = id;
        self.tilesets.push(tileset);
        id
    }

    pub fn update_tileset(&mut self, id: TilesetId, mut tileset: Tileset) -> bool {
        if let Some(target_tileset) = self.tilesets.iter_mut().find(|t| t.id == id) {
            // Set the tileset id - just to be sure; the caller might not have set it
            tileset.id = id;
            if target_tileset != &tileset {
                *target_tileset = tileset;
                return true;
            }
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push(
        &mut self,
        name: String,
        path: Utf8PathBuf,
        tile_size: U32Size2,
        size_in_tiles: U32Size2,
        mode: TilesetMode,
        foreground: Option<UserColor>,
        background: Option<UserColor>,
        prefer_relative_path: bool,
    ) {
        let id = self.next_tileset_id;
        self.next_tileset_id = id.next();
        self.tilesets.push(Tileset::new(
            id,
            name,
            path,
            tile_size,
            size_in_tiles,
            mode,
            foreground,
            background,
            prefer_relative_path,
        ));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert(
        &mut self,
        i: usize,
        name: String,
        path: Utf8PathBuf,
        tile_size: U32Size2,
        size_in_tiles: U32Size2,
        mode: TilesetMode,
        foreground: Option<UserColor>,
        background: Option<UserColor>,
        prefer_relative_path: bool,
    ) {
        let id = self.next_tileset_id;
        self.next_tileset_id = id.next();
        self.tilesets.insert(
            i,
            Tileset::new(
                id,
                name,
                path,
                tile_size,
                size_in_tiles,
                mode,
                foreground,
                background,
                prefer_relative_path,
            ),
        );
    }

    pub fn iter(&self) -> Iter<'_, Tileset> {
        self.tilesets.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Tileset> {
        self.tilesets.get(index)
    }

    pub fn get_by_id(&self, id: TilesetId) -> Option<&Tileset> {
        self.tilesets.iter().find(|t| t.id == id)
    }

    pub fn first(&self) -> Option<&Tileset> {
        self.tilesets.first()
    }

    pub fn last(&self) -> Option<&Tileset> {
        self.tilesets.last()
    }

    pub fn pop(&mut self) -> Option<Tileset> {
        self.tilesets.pop()
    }

    pub fn first_mut(&mut self) -> Option<&mut Tileset> {
        self.tilesets.first_mut()
    }

    pub fn last_mut(&mut self) -> Option<&mut Tileset> {
        self.tilesets.last_mut()
    }

    pub fn on_save(&mut self, path: Utf8PathBuf) {
        for tileset in self.tilesets.iter_mut() {
            tileset.on_save(path.clone());
        }
    }

    pub fn previous_by_id(&self, id: TilesetId) -> Option<&Tileset> {
        if let Some(index) = self.tilesets.iter().position(|t| t.id() == id) {
            if index > 0 {
                self.tilesets.get(index - 1)
            } else {
                self.tilesets.last()
            }
        } else {
            None
        }
    }

    pub fn next_by_id(&self, id: TilesetId) -> Option<&Tileset> {
        if let Some(index) = self.tilesets.iter().position(|t| t.id() == id) {
            if index < self.tilesets.len() - 1 {
                self.tilesets.get(index + 1)
            } else {
                self.tilesets.first()
            }
        } else {
            None
        }
    }

    pub fn delete_by_id(&mut self, tileset_id: TilesetId) -> bool {
        let start_len = self.tilesets.len();
        self.tilesets.retain(|tileset| tileset.id != tileset_id);
        self.tilesets.len() != start_len
    }

    pub fn is_tile_source_available(&self, source: TileSource) -> bool {
        if let Some(tileset) = self.get_by_id(source.tileset_id) {
            source.tile_index.index() < tileset.size_in_tiles.area()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json() -> eyre::Result<()> {
        let mode = TilesetMode::Direct;
        let json = serde_json::to_string(&mode)?;
        assert_eq!(json, "\"Direct\"");
        let mode = TilesetMode::TransparentBackground {
            background: UserColor::WHITE,
        };
        let json = serde_json::to_string(&mode)?;
        assert_eq!(
            json,
            "{\"TransparentBackground\":{\"background\":[255,255,255,255]}}"
        );
        Ok(())
    }
}
