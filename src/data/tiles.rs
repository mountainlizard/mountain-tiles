use egui::{Pos2, Rect, Vec2};

use crate::{
    data::{
        tiles::tile_color::{TileColor, UserColor},
        tilesets::TilesetId,
    },
    geom::{
        i32pos2::I32Pos2,
        transform::Transform,
        u32pos2::{u32pos2, U32Pos2},
        u32size2::U32Size2,
    },
};

pub mod layer_tiles;
pub mod stamp_tiles;
pub mod tile_color;
pub mod tile_selection;
pub mod tileset_stacked_tiles;
pub mod tileset_tiles;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TileIndex(u32);

impl TileIndex {
    #[inline(always)]
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub fn index(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
pub struct TileSource {
    pub tileset_id: TilesetId,
    pub tile_index: TileIndex,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Tile {
    pub source: TileSource,
    pub color: TileColor,
    pub transform: Transform,
}

impl Tile {
    pub fn with_transform(&self, transform: Transform) -> Tile {
        Tile {
            source: self.source,
            color: self.color,
            transform: self.transform.and_then(transform),
        }
    }

    pub fn with_color(&self, color: TileColor) -> Tile {
        Tile {
            source: self.source,
            color,
            transform: self.transform,
        }
    }
    pub fn with_override_color(&self, override_color: Option<TileColor>) -> Tile {
        match override_color {
            Some(color) => self.with_color(color),
            None => *self,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TileEvent {
    DragStarted { pos: U32Pos2 },
    Dragged { pos: U32Pos2 },
    DragStopped { pos: U32Pos2 },
}

impl TileEvent {
    pub fn pos(&self) -> U32Pos2 {
        *match self {
            Self::DragStarted { pos } => pos,
            Self::Dragged { pos } => pos,
            Self::DragStopped { pos } => pos,
        }
    }

    pub fn complete(&self) -> bool {
        match self {
            Self::DragStarted { .. } => false,
            Self::Dragged { .. } => false,
            Self::DragStopped { .. } => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SceneEvent {
    DragStarted { pos: I32Pos2 },
    Dragged { pos: I32Pos2 },
    DragStopped { pos: I32Pos2 },
}

impl SceneEvent {
    pub fn pos(&self) -> I32Pos2 {
        *match self {
            Self::DragStarted { pos } => pos,
            Self::Dragged { pos } => pos,
            Self::DragStopped { pos } => pos,
        }
    }

    pub fn complete(&self) -> bool {
        match self {
            Self::DragStarted { .. } => false,
            Self::Dragged { .. } => false,
            Self::DragStopped { .. } => true,
        }
    }

    pub fn as_tile_event(&self, map_size: U32Size2) -> TileEvent {
        match self {
            SceneEvent::DragStarted { pos } => TileEvent::DragStarted {
                pos: map_size.u32pos_constrained(*pos),
            },
            SceneEvent::Dragged { pos } => TileEvent::Dragged {
                pos: map_size.u32pos_constrained(*pos),
            },
            SceneEvent::DragStopped { pos } => TileEvent::DragStopped {
                pos: map_size.u32pos_constrained(*pos),
            },
        }
    }
}

pub struct MapPositionIterator {
    x: u32,
    y: u32,
    map_size: U32Size2,
    done: bool,
}

impl MapPositionIterator {
    pub fn new(map_size: U32Size2) -> Self {
        Self {
            x: 0,
            y: 0,
            map_size,
            done: map_size.w == 0 || map_size.h == 0,
        }
    }
}

impl Iterator for MapPositionIterator {
    type Item = U32Pos2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let pos = u32pos2(self.x, self.y);
            if self.x < self.map_size.w - 1 {
                self.x += 1;
            } else {
                self.x = 0;
                self.y += 1;
            }
            if self.y >= self.map_size.h {
                self.done = true;
            }
            Some(pos)
        }
    }
}

pub trait Tiles {
    fn background(&self) -> UserColor;
    fn layer_count(&self) -> usize;
    fn layer_opacity(&self, layer: usize) -> Option<f32>;
    fn tile_size(&self) -> U32Size2;
    fn map_size(&self) -> U32Size2;
    fn scale(&self) -> f32;
    fn gap(&self) -> U32Size2;
    fn tile(&self, layer: usize, pos: U32Pos2) -> Option<Tile>;

    /// Attempt to set the tile at the specified layer and position to the specified
    /// value.
    /// Return true if this actually changes the layer contents at all (e.g. if the
    /// position is outside the layer, or the tile is already set to an equal value,
    /// no change is made)
    fn set_tile(&mut self, layer: usize, pos: U32Pos2, tile: Option<Tile>) -> bool;

    /// Attempt to set the tile at the specified layer and position to the specified
    /// value.
    /// The position is an [`I32Pos2`], if this is outside the area of the tiles, it
    /// is ignored and no tile is set.
    /// Return true if this actually changes the layer contents at all (e.g. if the
    /// position is outside the layer, or the tile is already set to an equal value,
    /// no change is made)
    fn set_tile_i(&mut self, layer: usize, pos: I32Pos2, tile: Option<Tile>) -> bool {
        if let Some(pos) = self.map_size().u32pos_if_contained(pos) {
            self.set_tile(layer, pos, tile)
        } else {
            false
        }
    }

    fn map_positions(&self) -> MapPositionIterator {
        MapPositionIterator::new(self.map_size())
    }

    fn pixel_size_unscaled(&self) -> U32Size2 {
        self.map_size() * (self.tile_size() + self.gap())
    }

    fn pixel_size(&self) -> Vec2 {
        let unscaled: Vec2 = self.pixel_size_unscaled().into();
        unscaled * self.scale()
    }

    fn scale_for_square_size(&self, size: f32) -> f32 {
        let pixel_size_unscaled = self.pixel_size_unscaled();
        let max_pixel_size = pixel_size_unscaled.w.max(pixel_size_unscaled.h) as f32;
        size / max_pixel_size
    }

    fn response_pos_to_grid_pos(&self, response: &egui::Response, response_pos: Pos2) -> U32Pos2 {
        let rect = response.rect;
        let pixel_space: U32Pos2 = ((response_pos - rect.min) / self.scale()).into();
        let grid_space = pixel_space / (self.tile_size() + self.gap());
        grid_space.constrain_to(self.map_size())
    }

    /// Calculate the grid position for a response position - this uses a scene
    /// position for [`Tiles`] displayed in a [`egui::Scene`], so is allowed
    /// to return a position outside this [`Tiles`], including a negative position.
    fn scene_response_pos_to_grid_pos(&self, response_pos: Pos2) -> I32Pos2 {
        let pixel_space: I32Pos2 = (response_pos / self.scale()).into();
        let mut grid_space = pixel_space / (self.tile_size() + self.gap());
        if pixel_space.x < 0 {
            grid_space.x -= 1
        }
        if pixel_space.y < 0 {
            grid_space.y -= 1
        }
        grid_space
    }

    fn hovered(&self, response: &egui::Response) -> Option<U32Pos2> {
        if response.hovered() {
            response
                .hover_pos()
                .map(|origin| self.response_pos_to_grid_pos(response, origin))
        } else {
            None
        }
    }

    /// Calculate the grid position where hover occurred - this uses the scene
    /// position for [`Tiles`] displayed in a [`egui::Scene`], so is allowed
    /// to return a position outside this [`Tiles`], including a negative position.
    fn scene_hovered(&self, response: &egui::Response) -> Option<I32Pos2> {
        if response.hovered() {
            response
                .hover_pos()
                .map(|origin| self.scene_response_pos_to_grid_pos(origin))
        } else {
            None
        }
    }

    fn event(&self, response: &egui::Response) -> Option<TileEvent> {
        if let Some(pos) = response
            .interact_pointer_pos()
            .map(|origin| self.response_pos_to_grid_pos(response, origin))
        {
            if response.drag_started_by(egui::PointerButton::Primary) {
                Some(TileEvent::DragStarted { pos })
            } else if response.dragged_by(egui::PointerButton::Primary) {
                Some(TileEvent::Dragged { pos })
            } else if response.drag_stopped_by(egui::PointerButton::Primary) {
                Some(TileEvent::DragStopped { pos })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check whether a [`SceneEvent`] has occurred from interaction with a [`egui::Scene`]
    /// containing this [`Tiles`] instance
    fn scene_event(&self, response: &egui::Response) -> Option<SceneEvent> {
        if let Some(pos) = response
            .interact_pointer_pos()
            .map(|origin| self.scene_response_pos_to_grid_pos(origin))
        {
            if response.drag_started_by(egui::PointerButton::Primary) {
                Some(SceneEvent::DragStarted { pos })
            } else if response.dragged_by(egui::PointerButton::Primary) {
                Some(SceneEvent::Dragged { pos })
            } else if response.drag_stopped_by(egui::PointerButton::Primary) {
                Some(SceneEvent::DragStopped { pos })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn screen_size(&self) -> Vec2 {
        let pixel_size: Vec2 = (self.map_size() * self.tile_size()).into();
        pixel_size * self.scale()
    }

    fn screen_rect(&self) -> Rect {
        Rect::from_min_size(Pos2::ZERO, self.screen_size())
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::u32size2::u32size2;

    use super::*;

    fn position_iterator_case(map_size: U32Size2) {
        let iter = MapPositionIterator::new(map_size);
        let positions: Vec<_> = iter.collect();
        let mut expected_positions = vec![];
        for y in 0..map_size.h {
            for x in 0..map_size.w {
                expected_positions.push(u32pos2(x, y));
            }
        }
        assert_eq!(positions, expected_positions);
    }

    #[test]
    fn map_position_iterator() {
        position_iterator_case(u32size2(0, 0));
        position_iterator_case(u32size2(0, 4));
        position_iterator_case(u32size2(3, 0));
        position_iterator_case(u32size2(3, 4));
        position_iterator_case(u32size2(1, 1024));
        position_iterator_case(u32size2(1024, 1));
    }
}
