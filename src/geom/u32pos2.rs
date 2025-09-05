use core::fmt;
use core::ops::{Div, Mul};

use egui::{Pos2, Vec2};

use crate::geom::i32pos2::I32Pos2;
use crate::geom::u32size2::U32Size2;

/// Represents a pos in integer increments
/// e.g. to represent the pos of a rectangle of
/// tiles on a tile map
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
pub struct U32Pos2 {
    /// X coordinate
    pub x: u32,

    /// Y coordinate
    pub y: u32,
}

impl U32Pos2 {
    #[inline(always)]
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn linear_index(&self, size: U32Size2) -> Option<u32> {
        if self.x < size.w && self.y < size.h {
            Some(self.x + self.y * size.w)
        } else {
            None
        }
    }

    pub fn min_corner(&self, pos: &U32Pos2) -> U32Pos2 {
        Self {
            x: self.x.min(pos.x),
            y: self.y.min(pos.y),
        }
    }

    pub fn max_corner(&self, pos: &U32Pos2) -> U32Pos2 {
        Self {
            x: self.x.max(pos.x),
            y: self.y.max(pos.y),
        }
    }

    pub fn constrain_to(&self, size: U32Size2) -> U32Pos2 {
        u32pos2(self.x.min(size.w - 1), self.y.min(size.h - 1))
    }
}

/// `u32pos2(x, y) == U32Pos2::new(x, y)`
#[inline(always)]
pub const fn u32pos2(x: u32, y: u32) -> U32Pos2 {
    U32Pos2 { x, y }
}

impl fmt::Display for U32Pos2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        self.x.fmt(f)?;
        f.write_str(" ")?;
        self.y.fmt(f)?;
        f.write_str("]")?;
        Ok(())
    }
}

impl From<Vec2> for U32Pos2 {
    #[inline(always)]
    fn from(v: Vec2) -> Self {
        Self {
            x: v.x as u32,
            y: v.y as u32,
        }
    }
}

impl From<&Vec2> for U32Pos2 {
    #[inline(always)]
    fn from(v: &Vec2) -> Self {
        Self {
            x: v.x as u32,
            y: v.y as u32,
        }
    }
}

impl TryFrom<I32Pos2> for U32Pos2 {
    type Error = ();

    #[inline(always)]
    fn try_from(v: I32Pos2) -> Result<Self, Self::Error> {
        if v.x >= 0 && v.y >= 0 {
            Ok(Self {
                x: v.x as u32,
                y: v.y as u32,
            })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&I32Pos2> for U32Pos2 {
    type Error = ();

    #[inline(always)]
    fn try_from(v: &I32Pos2) -> Result<Self, Self::Error> {
        if v.x >= 0 && v.y >= 0 {
            Ok(Self {
                x: v.x as u32,
                y: v.y as u32,
            })
        } else {
            Err(())
        }
    }
}

impl From<U32Pos2> for Pos2 {
    #[inline(always)]
    fn from(p: U32Pos2) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

impl From<U32Pos2> for Vec2 {
    #[inline(always)]
    fn from(p: U32Pos2) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

/// Element-wise division
impl Div<Self> for U32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

/// Element-wise multiplication
impl Mul<Self> for U32Pos2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, p: Self) -> Self {
        Self {
            x: self.x * p.x,
            y: self.y * p.y,
        }
    }
}
