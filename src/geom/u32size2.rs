use core::fmt;
use core::ops::{Div, Mul};
use egui::Vec2;
use std::ops::Add;

use crate::geom::i32pos2::I32Pos2;
use crate::geom::u32pos2::{u32pos2, U32Pos2};

/// Represents a size in integer increments
/// e.g. to represent the size of a rectangle of
/// tiles on a tile map
#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    serde::Deserialize,
    serde::Serialize,
    // bytemuck::Pod,
    // bytemuck::Zeroable,
)]
pub struct U32Size2 {
    /// Width.
    pub w: u32,

    /// Height.
    pub h: u32,
}

/// `u32size2(w, h) == U32Size2::new(w, h)`
#[inline(always)]
pub const fn u32size2(w: u32, h: u32) -> U32Size2 {
    U32Size2 { w, h }
}

impl U32Size2 {
    pub const ZERO: U32Size2 = u32size2(0, 0);
    pub const ONE: U32Size2 = u32size2(1, 1);

    #[inline(always)]
    pub const fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }

    #[inline(always)]
    pub fn pos_from_linear_index(&self, i: u32) -> U32Pos2 {
        u32pos2(i % self.w, i / self.w)
    }

    #[inline(always)]
    pub fn area(&self) -> u32 {
        self.w * self.h
    }

    #[inline(always)]
    pub fn lossy_from_vec2(v: &Vec2) -> Self {
        Self {
            w: v.x as u32,
            h: v.y as u32,
        }
    }

    /// Saturating integer subtraction.
    /// Computes self - rhs for both width and height, saturating at the ÷
    /// numeric bounds instead of overflowing.
    pub fn saturating_sub(&self, rhs: &U32Size2) -> Self {
        Self {
            w: self.w.saturating_sub(rhs.w),
            h: self.h.saturating_sub(rhs.h),
        }
    }

    /// True if this size contains specified point,
    /// treating the size as containing points where:
    /// - 0 <= x < self.w
    /// - 0 <= y < self.h
    pub fn contains(&self, pos: I32Pos2) -> bool {
        let w: i64 = self.w as i64;
        let h: i64 = self.h as i64;
        let x: i64 = pos.x as i64;
        let y: i64 = pos.y as i64;
        x >= 0 && y >= 0 && x < w && y < h
    }

    /// If this size contains specified position,
    /// return the position as some [`U32Pos2`], otherwise
    /// None.
    pub fn u32pos_if_contained(&self, pos: I32Pos2) -> Option<U32Pos2> {
        if self.contains(pos) {
            // Since pointis contained, we know both components are within u32 range
            Some(u32pos2(pos.x as u32, pos.y as u32))
        } else {
            None
        }
    }

    /// Return the position as a [`U32Pos2`], constrained to be contained
    /// by this size
    pub fn u32pos_constrained(&self, pos: I32Pos2) -> U32Pos2 {
        let w: i64 = self.w as i64;
        let h: i64 = self.h as i64;
        let mut x: i64 = pos.x as i64;
        let mut y: i64 = pos.y as i64;
        if x < 0 {
            x = 0;
        }
        if y < 0 {
            y = 0;
        }
        if x > w - 1 {
            x = w - 1;
        }
        if y > h - 1 {
            y = h - 1;
        }
        u32pos2(x as u32, y as u32)
    }

    pub fn u32pos_shifted_and_wrapped(&self, p: U32Pos2, shift: I32Pos2) -> U32Pos2 {
        let w: i64 = self.w as i64;
        let h: i64 = self.h as i64;

        let mut x: i64 = p.x as i64;
        let mut y: i64 = p.y as i64;
        x += shift.x as i64;
        y += shift.y as i64;
        x = x.rem_euclid(w);
        y = y.rem_euclid(h);
        u32pos2(x as u32, y as u32)
    }
}

impl fmt::Display for U32Size2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        self.w.fmt(f)?;
        f.write_str(" ")?;
        self.h.fmt(f)?;
        f.write_str("]")?;
        Ok(())
    }
}

impl From<U32Size2> for Vec2 {
    #[inline(always)]
    fn from(size: U32Size2) -> Self {
        Self {
            x: size.w as f32,
            y: size.h as f32,
        }
    }
}

impl From<&U32Size2> for Vec2 {
    #[inline(always)]
    fn from(size: &U32Size2) -> Self {
        Self {
            x: size.w as f32,
            y: size.h as f32,
        }
    }
}

/// Element-wise division
impl Div<Self> for U32Size2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        Self {
            w: self.w / rhs.w,
            h: self.h / rhs.h,
        }
    }
}

/// Element-wise division
impl Div<U32Size2> for U32Pos2 {
    type Output = U32Pos2;

    #[inline(always)]
    fn div(self, rhs: U32Size2) -> U32Pos2 {
        U32Pos2 {
            x: self.x / rhs.w,
            y: self.y / rhs.h,
        }
    }
}

/// Element-wise multiplication
impl Mul<U32Size2> for U32Pos2 {
    type Output = U32Pos2;

    #[inline(always)]
    fn mul(self, rhs: U32Size2) -> U32Pos2 {
        U32Pos2 {
            x: self.x * rhs.w,
            y: self.y * rhs.h,
        }
    }
}

/// Element-wise multiplication
impl Mul<Self> for U32Size2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, size: Self) -> Self {
        Self {
            w: self.w * size.w,
            h: self.h * size.h,
        }
    }
}

/// Element-wise addition
impl Add<Self> for U32Size2 {
    type Output = Self;

    #[inline(always)]
    fn add(self, size: Self) -> Self {
        Self {
            w: self.w + size.w,
            h: self.h + size.h,
        }
    }
}
