use core::fmt;
use core::ops::{Div, Mul};
use std::ops::{Add, Sub};

use egui::{Pos2, Vec2};

use crate::geom::transform::Transform;
use crate::geom::u32pos2::U32Pos2;
use crate::geom::u32size2::U32Size2;

/// Represents a pos in signed integer increments
/// e.g. to represent the pos of a rectangle of
/// tiles on a tile map, allowing for positions outside the map,
/// or for things like floating selections of tiles centered on
/// the cursor.
#[repr(C)]
#[derive(
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    serde::Deserialize,
    serde::Serialize,
    Debug,
    // bytemuck::Pod,
    // bytemuck::Zeroable,
)]
pub struct I32Pos2 {
    /// X coordinate
    pub x: i32,

    /// Y coordinate
    pub y: i32,
}

/// `i32pos2(x, y) == U32Pos2::new(x, y)`
#[inline(always)]
pub const fn i32pos2(x: i32, y: i32) -> I32Pos2 {
    I32Pos2 { x, y }
}

impl I32Pos2 {
    #[inline(always)]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn min_corner(&self, pos: I32Pos2) -> I32Pos2 {
        Self {
            x: self.x.min(pos.x),
            y: self.y.min(pos.y),
        }
    }

    pub fn max_corner(&self, pos: I32Pos2) -> I32Pos2 {
        Self {
            x: self.x.max(pos.x),
            y: self.y.max(pos.y),
        }
    }

    pub fn with_transform(&self, transform: Transform) -> I32Pos2 {
        transform.apply_to_pos(self)
    }
}

impl fmt::Display for I32Pos2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        self.x.fmt(f)?;
        f.write_str(" ")?;
        self.y.fmt(f)?;
        f.write_str("]")?;
        Ok(())
    }
}

impl From<Vec2> for I32Pos2 {
    #[inline(always)]
    fn from(v: Vec2) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}

impl From<&Vec2> for I32Pos2 {
    #[inline(always)]
    fn from(v: &Vec2) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}

impl From<Pos2> for I32Pos2 {
    #[inline(always)]
    fn from(v: Pos2) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}

impl From<&Pos2> for I32Pos2 {
    #[inline(always)]
    fn from(v: &Pos2) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}

impl From<U32Pos2> for I32Pos2 {
    #[inline(always)]
    fn from(v: U32Pos2) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}

impl From<&U32Pos2> for I32Pos2 {
    #[inline(always)]
    fn from(v: &U32Pos2) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}

impl From<I32Pos2> for Pos2 {
    #[inline(always)]
    fn from(p: I32Pos2) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

impl From<I32Pos2> for Vec2 {
    #[inline(always)]
    fn from(p: I32Pos2) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

/// Element-wise division
impl Div<Self> for I32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

/// Element-wise division
impl Div<U32Pos2> for I32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: U32Pos2) -> Self {
        Self {
            x: self.x / (rhs.x as i32),
            y: self.y / (rhs.y as i32),
        }
    }
}

/// Element-wise division
impl Div<U32Size2> for I32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: U32Size2) -> Self {
        Self {
            x: self.x / (rhs.w as i32),
            y: self.y / (rhs.h as i32),
        }
    }
}

/// Element-wise multiplication
impl Mul<Self> for I32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, p: Self) -> Self {
        Self {
            x: self.x * p.x,
            y: self.y * p.y,
        }
    }
}

/// Element-wise subtraction
impl Sub<Self> for I32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, p: Self) -> Self {
        Self {
            x: self.x - p.x,
            y: self.y - p.y,
        }
    }
}

/// Element-wise addition
impl Add<Self> for I32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn add(self, p: Self) -> Self {
        Self {
            x: self.x + p.x,
            y: self.y + p.y,
        }
    }
}
