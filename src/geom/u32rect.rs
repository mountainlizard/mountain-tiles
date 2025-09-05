use crate::{
    geom::u32pos2::{u32pos2, U32Pos2},
    geom::u32size2::U32Size2,
};

/// A rectangular region of space, as u32 coordinates.
///
/// A U32Rect always has non-zero size (use an [`Option`] if empty rect is required),
/// since it is defined in terms of two corners, and both corners are included in the
/// rectangle.
///
/// A rectangle is allowed to have a negative size, which happens when the order
/// of `min` and `max` are swapped. This still includes the same points as for a positive
/// size, but can be used to represent the rect being mirrored, or when representing a selection,
/// we may use min for the start of the drag, and max for the end.
///
/// Normally the unit is in grid cells.
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
pub struct U32Rect {
    /// One of the corners of the rectangle, usually the left top one.
    pub min: U32Pos2,

    /// The other corner, opposing [`Self::min`]. Usually the right bottom one.
    pub max: U32Pos2,
}

impl U32Rect {
    #[inline]
    pub fn from_pos(pos: U32Pos2) -> U32Rect {
        Self { min: pos, max: pos }
    }

    /// Returns the bounding rectangle of the two points.
    #[inline]
    pub fn from_two_pos(a: U32Pos2, b: U32Pos2) -> Self {
        Self {
            min: u32pos2(a.x.min(b.x), a.y.min(b.y)),
            max: u32pos2(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    /// A copy of this rectangle, with a guaranteed positive size (i.e. min is top-left, max is bottom-right)
    #[inline]
    pub fn with_positive_size(&self) -> Self {
        Self::from_two_pos(self.min, self.max)
    }

    /// Size of this rect, from top left of included grid cells, to bottom right (always at least 1 by 1)
    #[inline]
    pub fn size(&self) -> U32Size2 {
        U32Size2 {
            w: self.min.x.max(self.max.x) - self.min.x.min(self.max.x) + 1,
            h: self.min.y.max(self.max.y) - self.min.y.min(self.max.y) + 1,
        }
    }

    /// Center of this rect, rounding down for odd width/height
    pub fn center(&self) -> U32Pos2 {
        u32pos2((self.min.x + self.max.x) / 2, (self.min.y + self.max.y) / 2)
    }
}
