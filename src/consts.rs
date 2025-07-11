use bevy::prelude::*;

/// TODO: Replace with `std::f32::consts::SQRT_3` when that is stable.
//pub const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;
pub const SQRT_3_2: f32 = 0.866025403784438646763723170752936183_f32;

/// The full hex size
pub const FLOOR_TILE_SIZE: IVec2 = IVec2 { x: 24, y: 26 };

pub const AXIAL_TRANSLATION_MATRIX: Mat2 =
    Mat2::from_cols_array(&[SQRT_3_2, 1.0 / 3.0, 0.0, 2.0 / 3.0]);

#[cfg(feature = "debug")]
pub const OVERLAY_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
