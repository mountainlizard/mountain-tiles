use std::collections::hash_set::Iter;

use crate::{
    data::tiles::TileEvent,
    geom::{
        u32pos2::{u32pos2, U32Pos2},
        u32rect::U32Rect,
    },
};
use egui::ahash::HashSet;
use egui::ahash::HashSetExt;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum SelectionType {
    #[default]
    Selection,
    Erase,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TileSelection {
    selection_type: SelectionType,
    positions: HashSet<U32Pos2>,
    last_position: Option<U32Pos2>,
    pub drag_rect: Option<U32Rect>,
}

impl TileSelection {
    pub fn new() -> TileSelection {
        TileSelection {
            selection_type: SelectionType::Selection,
            positions: HashSet::new(),
            last_position: None,
            drag_rect: None,
        }
    }

    pub fn erase(pos: Option<U32Pos2>) -> TileSelection {
        let mut positions = HashSet::new();
        if let Some(pos) = pos {
            positions.insert(pos);
        }
        TileSelection {
            selection_type: SelectionType::Erase,
            positions,
            last_position: None,
            drag_rect: None,
        }
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.last_position = None;
    }

    pub fn add_selection(&mut self, pos: U32Pos2) {
        self.positions.insert(pos);
        self.last_position = Some(pos);
    }

    pub fn iter(&self) -> Iter<'_, U32Pos2> {
        self.positions.iter()
    }

    pub fn add_rect_selection_from_last(&mut self, pos: U32Pos2) {
        match self.last_position {
            Some(last) => {
                self.add_rect_selection(U32Rect::from_two_pos(pos, last));
            }
            None => self.add_selection(pos),
        }
    }

    pub fn are_all_selected(&mut self, rect: U32Rect) -> bool {
        let pos_rect = rect.with_positive_size();
        for y in pos_rect.min.y..=pos_rect.max.y {
            for x in pos_rect.min.x..=pos_rect.max.x {
                let pos = u32pos2(x, y);
                if !self.positions.contains(&pos) {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_exactly_selected(&mut self, rect: U32Rect) -> bool {
        let pos_rect = rect.with_positive_size();
        let mut new_positions = HashSet::new();
        for y in pos_rect.min.y..=pos_rect.max.y {
            for x in pos_rect.min.x..=pos_rect.max.x {
                let pos = u32pos2(x, y);
                new_positions.insert(pos);
            }
        }
        new_positions == self.positions
    }

    pub fn add_rect_selection(&mut self, rect: U32Rect) {
        self.apply_rect_selection(rect, false);
    }

    pub fn apply_rect_selection(&mut self, rect: U32Rect, remove: bool) {
        let pos_rect = rect.with_positive_size();
        for y in pos_rect.min.y..=pos_rect.max.y {
            for x in pos_rect.min.x..=pos_rect.max.x {
                if remove {
                    self.positions.remove(&u32pos2(x, y));
                } else {
                    self.positions.insert(u32pos2(x, y));
                }
            }
        }
        self.last_position = Some(rect.max);
    }

    pub fn range_rect(&self) -> Option<U32Rect> {
        if let Some(first) = self.positions.iter().next() {
            let mut min = *first;
            let mut max = *first;

            for pos in self.positions.iter() {
                min = min.min_corner(pos);
                max = max.max_corner(pos);
            }
            Some(U32Rect { min, max })
        } else {
            None
        }
    }

    /// Apply a tile event to the selection, return true if selection was updated
    pub fn apply_tile_event(&mut self, event: &TileEvent, shift: bool, command: bool) -> bool {
        match event {
            TileEvent::DragStarted { pos } => {
                self.drag_rect = Some(U32Rect::from_pos(*pos));
                false
            }
            TileEvent::Dragged { pos } => {
                if let Some(mut drag_rect) = self.drag_rect {
                    drag_rect.max = *pos;
                    self.drag_rect = Some(drag_rect);
                }
                false
            }
            TileEvent::DragStopped { pos } => {
                if let Some(mut drag_rect) = self.drag_rect {
                    drag_rect.max = *pos;

                    // Exactly repeating current selection always clears it
                    if self.is_exactly_selected(drag_rect) {
                        self.clear();
                    } else {
                        // Clear old selection unless we are extending
                        if !shift && !command {
                            self.clear();
                        }
                        // If the new selection is entirely within already-selected cells,
                        // then we will remove the new selection, otherwise we add it
                        let remove = self.are_all_selected(drag_rect);
                        self.apply_rect_selection(drag_rect, remove);
                    }

                    self.drag_rect = None;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn selection_type(&self) -> SelectionType {
        self.selection_type
    }
}
