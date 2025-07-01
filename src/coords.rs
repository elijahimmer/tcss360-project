use bevy::prelude::*;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use std::ops;

pub type Size = f32;

/// TODO: Replace with `std::f32::consts::SQRT_3` when that is stable.
pub const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32; // 1.73205078f32

//pub enum Axis {
//    Q,
//    R,
//    S,
//}

/// The coordinate system is based on the 'Axial Coordinates'
///  https://www.redblobgames.com/grids/hexagons/
///
/// We define the distance from the center of 1 hexagon to the
/// center of any adjacent hexagon as 1 meter.
///
/// The '.' line below is the `q` dimension
/// The '-' line below is the `r` dimension
/// The ',' line below is the `s` dimension
///
///         / \     / \
///       /     \ /     \
///      |       |       |
///      |       |       |
///     / \     / \     / \
///   /     \ /     \ /     \
///  |       |   o---|---r   |
///  |       |  , .  |       |
///   \     / \,   ./ \     /
///     \ /   , \ / .   \ /
///      |   s   |   q   |
///      |       |       |
///       \     / \     /
///         \ /     \ /
///
/// With Cube Coordinates, `q + r + s = 0` is always true. That also means
/// that one of the coordinates is redundant (we choose `s` in this case),
/// as it can be calculated with `s = -q-r`
///
/// With this known, we can ignore the third coordinate, and calculate it as needed.
///
/// The relative coordinates of each adjacent tile is defined as:
///
///                    / \     / \
///                  /     \ /     \
///                 | -1,0  |  1,-1 |
///                 |       |       |
///                / \     / \     / \
///              /     \ /     \ /     \
///             |  0,-1 |  0,0  |  0,1  |
///             |       |       |       |
///              \     / \     / \     /
///                \ /     \ /     \ /
///                 | -1,1  |  0,1  |
///                 |       |       |
///                  \     / \     /
///                    \ /     \ /
///
///
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct Axial {
    /// This represents the horizontal axis
    /// A fixed point number with 8 decimal bits
    q: Size,
    /// This represents the south-east axis
    /// A fixed point number with 8 decimal bits
    r: Size,
}

impl Axial {
    pub const ORIGIN: Self = Self { q: 0., r: 0. };

    pub const Q: Self = Self {
        q: 1.,
        ..Axial::ORIGIN
    };
    pub const R: Self = Self {
        r: 1.,
        ..Axial::ORIGIN
    };
    pub const S: Self = Self {
        r: 1.,
        ..Axial::ORIGIN
    };

    pub fn neighbor(self, dir: Direction) -> Self {
        self + dir.into()
    }

    pub fn round(self) -> Self {
        let q_grid = self.q.round();
        let r_grid = self.r.round();
        let q_frac = self.q.fract();
        let r_frac = self.r.fract();

        let (offset_q, offset_r) = (q_frac * q_frac >= r_frac * r_frac)
            .then_some((1., 0.))
            .unwrap_or((0., 1.));

        let dq = (q_frac + 0.5 * r_frac).round() * offset_q;
        let dr = (r_frac + 0.5 * q_frac).round() * offset_r;
        return Self {
            q: q_grid + dq,
            r: r_grid + dr,
        };
    }
}

impl From<Direction> for Axial {
    fn from(dir: Direction) -> Self {
        dir.to_axial()
    }
}

impl From<Vec2> for Axial {
    fn from(value: Vec2) -> Self {
        Self {
            q: value.x * SQRT_3 / 2. + value.y / 3.,
            r: value.y * 2. / 3.,
        }
    }
}

impl From<Axial> for Vec2 {
    fn from(value: Axial) -> Vec2 {
        Vec2 {
            x: SQRT_3 * value.q + SQRT_3 / 2. * value.r,
            y: value.r * 3. / 2.,
        }
    }
}

impl ops::Add for Axial {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl ops::Sub for Axial {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}

impl ops::Neg for Axial {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            q: -self.q,
            r: -self.r,
        }
    }
}

impl ops::Mul<Size> for Axial {
    type Output = Self;

    fn mul(self, rhs: Size) -> Self {
        Self {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}

impl Display for Axial {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "({}, {}, {})", self.q, self.r, -self.q - self.r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(Axial::ORIGIN + Axial::ORIGIN, Axial::ORIGIN);
        assert_eq!(
            Axial::ORIGIN + Axial { q: 5., r: 1. },
            Axial { q: 5., r: 1. }
        );
        assert_eq!(
            Axial::ORIGIN + Direction::East.into(),
            Direction::East.into()
        );
        assert_eq!(
            Axial::ORIGIN + Direction::NorthEast.into(),
            Direction::NorthEast.into()
        );
        assert_eq!(
            Axial::ORIGIN + Direction::NorthWest.into(),
            Direction::NorthWest.into()
        );
        assert_eq!(
            Axial::ORIGIN + Direction::West.into(),
            Direction::West.into()
        );
        assert_eq!(
            Axial::ORIGIN + Direction::SouthWest.into(),
            Direction::SouthWest.into()
        );
        assert_eq!(
            Axial::ORIGIN + Direction::SouthEast.into(),
            Direction::SouthEast.into()
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(Axial::ORIGIN * 10., Axial::ORIGIN);
        assert_eq!(Axial { q: 0., r: 1. } * 0., Axial::ORIGIN);
    }
}

/// Directions relative to the Hexagonal Grid.
///
/// On a hexagon the directions are as such:
///
///         / \     / \
///       /     \ /     \
///      | NORTH | NORTH |
///      |  WEST |  EAST |
///     / \     / \     / \
///   /     \ /     \ /     \
///  | WEST  | START |  EAST |
///  |       |       |       |
///   \     / \     / \     /
///     \ /     \ /     \ /
///      | SOUTH | SOUTH |
///      |  WEST |  EAST |
///       \     / \     /
///         \ /     \ /
///
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
    SouthEast,
}

impl Direction {
    pub const ALL: &[Direction] = &[
        Direction::East,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::West,
        Direction::SouthWest,
        Direction::SouthEast,
    ];

    /// Returns a Cartesian angle in radians where 0.0 is
    /// the x axis, and going counter-clockwise (the standard direction).
    pub const fn to_angle(self) -> f32 {
        match self {
            Direction::East => 0.,
            Direction::NorthEast => PI * 1. / 3.,
            Direction::NorthWest => PI * 2. / 3.,
            Direction::West => PI,
            Direction::SouthWest => PI * 4. / 3.,
            Direction::SouthEast => PI * 5. / 3.,
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2 {
            x: self.to_angle().cos(),
            y: self.to_angle().sin(),
        }
    }

    pub fn to_axial(self) -> Axial {
        match self {
            Direction::East => Axial::R,
            Direction::SouthEast => Axial::Q,
            Direction::SouthWest => Axial::S,
            Direction::West => -Axial::R,
            Direction::NorthWest => -Axial::Q,
            Direction::NorthEast => -Axial::S,
        }
    }

    pub const fn invert_x(self) -> Self {
        match self {
            Direction::East => Direction::West,
            Direction::NorthEast => Direction::NorthWest,
            Direction::NorthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::SouthWest => Direction::SouthEast,
            Direction::SouthEast => Direction::SouthWest,
        }
    }

    pub const fn invert_y(self) -> Self {
        match self {
            Direction::East => Direction::East,
            Direction::NorthEast => Direction::SouthEast,
            Direction::NorthWest => Direction::SouthWest,
            Direction::West => Direction::West,
            Direction::SouthWest => Direction::NorthWest,
            Direction::SouthEast => Direction::NorthEast,
        }
    }
}

impl From<Direction> for Vec2 {
    fn from(dir: Direction) -> Vec2 {
        dir.to_vec2()
    }
}
