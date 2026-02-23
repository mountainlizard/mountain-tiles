use core::fmt::Display;

use crate::geom::i32pos2::{i32pos2, I32Pos2};

/// The set of possible tile transforms.
/// These are described based on a clockwise rotation (common in tilemap editors),
/// first of an un-mirrored tile, then of one that is first mirrored in the x axis,
/// then rotated.
/// To recreate the correct transform, the mirror is performed before the rotation.
/// The representation is a u8 value where the bits (from most significant to least
/// significant), represent mirroring in the x axis (i.e. x maps to -x), then the
/// y axis (i.e. y maps to -y), then the diagonal axis running from the top-left of
/// the tile to the bottom-right. Note that this all applies in the egui/UV coordinate
/// system with the origin in the top left, the x axis running right, and the y axis down.
/// This allows easy conversion to the format used for tile indices in Tiled maps,
/// where bits 31, 30 and 29 correspond to bits 2, 1 and 0 of this enum's values.
/// So for a given enum value, we can just use `value << 29` to produce the bits required
/// in a Tiled index.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Default)]
#[repr(u8)]
pub enum Transform {
    #[default]
    None = 0b000,
    Rotate90 = 0b101,
    Rotate180 = 0b110,
    Rotate270 = 0b011,
    MirrorX = 0b100,
    MirrorXRotate90 = 0b111,
    MirrorXRotate180 = 0b010,
    MirrorXRotate270 = 0b001,
}

impl Transform {
    const MIRROR_X_BIT: u8 = 0b100;
    const MIRROR_Y_BIT: u8 = 0b010;
    const SWAP_XY_BIT: u8 = 0b001;

    pub fn from_tiled_flip_bits(bits: u32) -> Transform {
        let bits = (bits >> 29) as u8;
        Self::from_u8(bits)
    }

    pub fn as_tiled_flip_bits(&self) -> u32 {
        (self.as_u8() as u32) << 29
    }

    pub fn from_u8(bits: u8) -> Transform {
        match bits {
            0b000 => Self::None,
            0b101 => Self::Rotate90,
            0b110 => Self::Rotate180,
            0b011 => Self::Rotate270,
            0b100 => Self::MirrorX,
            0b111 => Self::MirrorXRotate90,
            0b010 => Self::MirrorXRotate180,
            0b001 => Self::MirrorXRotate270,
            _ => Self::None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::None => 0b000,
            Self::Rotate90 => 0b101,
            Self::Rotate180 => 0b110,
            Self::Rotate270 => 0b011,
            Self::MirrorX => 0b100,
            Self::MirrorXRotate90 => 0b111,
            Self::MirrorXRotate180 => 0b010,
            Self::MirrorXRotate270 => 0b001,
        }
    }

    pub fn from_bools(mirror_x: bool, mirror_y: bool, swap_xy: bool) -> Transform {
        let mut bits = 0b000;
        if mirror_x {
            bits |= Self::MIRROR_X_BIT;
        }
        if mirror_y {
            bits |= Self::MIRROR_Y_BIT;
        }
        if swap_xy {
            bits |= Self::SWAP_XY_BIT;
        }
        Self::from_u8(bits)
    }

    pub fn mirror_x(&self) -> bool {
        (*self as u8) & Self::MIRROR_X_BIT != 0
    }
    pub fn mirror_y(&self) -> bool {
        (*self as u8) & Self::MIRROR_Y_BIT != 0
    }
    pub fn swap_xy(&self) -> bool {
        (*self as u8) & Self::SWAP_XY_BIT != 0
    }

    pub fn inverse(&self) -> Transform {
        match self {
            Self::None => Self::None,
            Self::Rotate90 => Self::Rotate270,
            Self::Rotate180 => Self::Rotate180,
            Self::Rotate270 => Self::Rotate90,
            Self::MirrorX => Self::MirrorX,
            Self::MirrorXRotate90 => Self::MirrorXRotate90,
            Self::MirrorXRotate180 => Self::MirrorXRotate180,
            Self::MirrorXRotate270 => Self::MirrorXRotate270,
        }
    }

    pub fn apply_to_pos(&self, pos: &I32Pos2) -> I32Pos2 {
        let mut x = pos.x;
        let mut y = pos.y;
        if self.swap_xy() {
            (x, y) = (y, x);
        }
        if self.mirror_x() {
            x = -x;
        }
        if self.mirror_y() {
            y = -y;
        }
        i32pos2(x, y)
    }

    pub fn and_then(&self, then: Transform) -> Transform {
        let mut mirror_x = self.mirror_x();
        let mut mirror_y = self.mirror_y();
        let mut swap_xy = self.swap_xy();

        if then.swap_xy() {
            swap_xy = !swap_xy;
            (mirror_x, mirror_y) = (mirror_y, mirror_x);
        }
        if then.mirror_x() {
            mirror_x = !mirror_x;
        }
        if then.mirror_y() {
            mirror_y = !mirror_y;
        }

        Self::from_bools(mirror_x, mirror_y, swap_xy)
    }
}

impl Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Rotate90 => write!(f, "Rotate90"),
            Self::Rotate180 => write!(f, "Rotate180"),
            Self::Rotate270 => write!(f, "Rotate270"),
            Self::MirrorX => write!(f, "MirrorX"),
            Self::MirrorXRotate90 => write!(f, "MirrorXRotate90"),
            Self::MirrorXRotate180 => write!(f, "MirrorXRotate180"),
            Self::MirrorXRotate270 => write!(f, "MirrorXRotate270"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSFORMS: [Transform; 8] = [
        Transform::None,
        Transform::Rotate90,
        Transform::Rotate180,
        Transform::Rotate270,
        Transform::MirrorX,
        Transform::MirrorXRotate90,
        Transform::MirrorXRotate180,
        Transform::MirrorXRotate270,
    ];

    const CASES: [(Transform, bool, bool, bool); 8] = [
        (Transform::None, false, false, false),
        (Transform::Rotate90, true, false, true),
        (Transform::Rotate180, true, true, false),
        (Transform::Rotate270, false, true, true),
        (Transform::MirrorX, true, false, false),
        (Transform::MirrorXRotate90, true, true, true),
        (Transform::MirrorXRotate180, false, true, false),
        (Transform::MirrorXRotate270, false, false, true),
    ];

    #[test]
    fn mirror_and_swap_bits_should_extract_correctly() {
        for (transform, mirror_x, mirror_y, swap_xy) in CASES.iter() {
            assert_eq!(transform.mirror_x(), *mirror_x);
            assert_eq!(transform.mirror_y(), *mirror_y);
            assert_eq!(transform.swap_xy(), *swap_xy);
        }
    }

    #[test]
    fn from_u8_should_give_correct_transform() {
        // Round trip from transform to u8 and back
        for transform in TRANSFORMS.iter() {
            assert_eq!(*transform, Transform::from_u8(*transform as u8));
        }
    }

    #[test]
    fn from_bools_should_give_correct_transform() {
        for (transform, mirror_x, mirror_y, swap_xy) in CASES.iter() {
            assert_eq!(
                *transform,
                Transform::from_bools(*mirror_x, *mirror_y, *swap_xy)
            );
        }
    }

    // The "from" point for position test cases, using a point without any
    // symmetry for mirroring in x or y, or 90 degree rotations about the origin.
    const FROM_POS: I32Pos2 = i32pos2(1, 2);

    // Each case shows where FROM_POS maps to, under the given transform
    // Worked out by hand with paper :)
    const POS_CASES: [(Transform, I32Pos2); 8] = [
        (Transform::None, i32pos2(1, 2)),
        (Transform::Rotate90, i32pos2(-2, 1)),
        (Transform::Rotate180, i32pos2(-1, -2)),
        (Transform::Rotate270, i32pos2(2, -1)),
        (Transform::MirrorX, i32pos2(-1, 2)),
        (Transform::MirrorXRotate90, i32pos2(-2, -1)),
        (Transform::MirrorXRotate180, i32pos2(1, -2)),
        (Transform::MirrorXRotate270, i32pos2(2, 1)),
    ];

    #[test]
    fn applying_to_pos_should_give_correct_new_pos() {
        for (transform, end_pos) in POS_CASES.iter() {
            let transform_end_pos = transform.apply_to_pos(&FROM_POS);
            assert_eq!(
                end_pos, &transform_end_pos,
                "{:?} should map {} to {}, but we got {}",
                transform, FROM_POS, end_pos, transform_end_pos
            );
        }
    }

    #[test]
    fn applying_transform_then_inverse_should_leave_pos_unaltered() {
        for transform in TRANSFORMS.iter() {
            let transformed = transform.apply_to_pos(&FROM_POS);
            let transformed_back = transform.inverse().apply_to_pos(&transformed);
            assert_eq!(FROM_POS, transformed_back);
        }
    }

    #[test]
    fn applying_inverse_then_transform_should_leave_pos_unaltered() {
        for transform in TRANSFORMS.iter() {
            let inverse_transformed = transform.inverse().apply_to_pos(&FROM_POS);
            let transformed_back = transform.apply_to_pos(&inverse_transformed);
            assert_eq!(FROM_POS, transformed_back);
        }
    }

    #[test]
    fn applying_any_transform_pair_individually_to_a_pos_should_give_same_result_as_applying_combined_single_transform(
    ) {
        for first in TRANSFORMS.iter() {
            for second in TRANSFORMS.iter() {
                let combined = first.and_then(*second);
                assert_eq!(
                    second.apply_to_pos(&first.apply_to_pos(&FROM_POS)),
                    combined.apply_to_pos(&FROM_POS)
                );
            }
        }
    }
}
