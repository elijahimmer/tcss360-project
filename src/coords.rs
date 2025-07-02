use bevy::prelude::*;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use std::ops;

pub type Size = i32;

/// TODO: Replace with `std::f32::consts::SQRT_3` when that is stable.
//pub const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;
pub const SQRT_3_2: f32 = 0.866025403784438646763723170752936183_f32;

/// The full hex size
pub const HEX_SIZE: IVec2 = IVec2 { x: 24, y: 26 };
/// The offset from the center of one hexagon to the center of another
/// hex that is in the `S` or `Q` direction.
pub const HEX_HORI: IVec2 = IVec2 { x: 12, y: -20 };

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
/// Internally, this uses a normal vector, where the `x` coordinate is the `q` coordinate
/// and the `y` coordinate is actually the `r` coordinate.
///
///
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Axial(pub IVec2);

impl Axial {
    pub const ZERO: Self = Self(IVec2::ZERO);

    pub const Q: Self = Self(IVec2 { x: 1, y: 0 });
    pub const R: Self = Self(IVec2 { x: 0, y: 1 });
    pub const S: Self = Self(IVec2 { x: 1, y: -1 });

    pub fn new(q: Size, r: Size) -> Axial {
        Axial(IVec2 { x: q, y: r })
    }

    pub fn neighbor(self, dir: Direction) -> Self {
        self + dir.into()
    }

    pub fn as_ivec2(self) -> IVec2 {
        IVec2 {
            x: self.0.y * HEX_SIZE.x,
            y: 0,
        } + (self.0.x * HEX_HORI)
    }

    pub fn as_vec2(self) -> Vec2 {
        self.as_ivec2().as_vec2()
    }

    pub fn as_ivec3(self, z: i32) -> IVec3 {
        self.as_ivec2().extend(z)
    }

    pub fn as_vec3(self, z: f32) -> Vec3 {
        self.as_vec2().extend(z)
    }
}

impl From<Direction> for Axial {
    fn from(dir: Direction) -> Self {
        dir.as_axial()
    }
}

pub const AXIAL_TRANSLATION_MATRIX: Mat2 = Mat2::from_cols_array(&[SQRT_3_2, 1. / 3., 0., 2. / 3.]);

impl From<Vec2> for Axial {
    fn from(value: Vec2) -> Self {
        Self((AXIAL_TRANSLATION_MATRIX * value).as_ivec2())
    }
}

impl From<Axial> for IVec2 {
    fn from(value: Axial) -> IVec2 {
        value.as_ivec2()
    }
}

impl From<IVec2> for Axial {
    fn from(value: IVec2) -> Self {
        Self::from(value.as_vec2())
    }
}

impl ops::Add for Axial {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl ops::Sub for Axial {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl ops::Neg for Axial {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl ops::Mul<Size> for Axial {
    type Output = Self;

    fn mul(self, rhs: Size) -> Self {
        Self(self.0 * rhs)
    }
}

impl Display for Axial {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "({}, {}, {})", self.0.x, self.0.y, -self.0.x - self.0.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(Axial::ZERO + Axial::ZERO, Axial::ZERO);
        assert_eq!(Axial::ZERO + Axial { q: 5., r: 1 }, Axial { q: 5., r: 1 });
        assert_eq!(Axial::ZERO + Direction::East.into(), Direction::East.into());
        assert_eq!(
            Axial::ZERO + Direction::NorthEast.into(),
            Direction::NorthEast.into()
        );
        assert_eq!(
            Axial::ZERO + Direction::NorthWest.into(),
            Direction::NorthWest.into()
        );
        assert_eq!(Axial::ZERO + Direction::West.into(), Direction::West.into());
        assert_eq!(
            Axial::ZERO + Direction::SouthWest.into(),
            Direction::SouthWest.into()
        );
        assert_eq!(
            Axial::ZERO + Direction::SouthEast.into(),
            Direction::SouthEast.into()
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(Axial::ZERO * 10, Axial::ZERO);
        assert_eq!(Axial { q: 0, r: 1 } * 0, Axial::ZERO);
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
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum Direction {
    #[default]
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

    /// Returns a Cartesian angle in radians where 00 is
    /// the x axis, and going counter-clockwise (the standard direction).
    pub const fn as_angle(self) -> f32 {
        match self {
            Direction::East => 0.,
            Direction::NorthEast => PI / 3.,
            Direction::NorthWest => PI * 2. / 3.,
            Direction::West => PI,
            Direction::SouthWest => PI * 4. / 3.,
            Direction::SouthEast => PI * 5. / 3.,
        }
    }

    pub fn as_ivec2(self) -> IVec2 {
        self.as_axial().as_ivec2()
    }

    pub fn as_vec2(self) -> Vec2 {
        self.as_axial().as_vec2()
    }

    pub fn as_ivec3(self, z: i32) -> IVec3 {
        self.as_axial().as_ivec3(z)
    }

    pub fn as_vec3(self, z: f32) -> Vec3 {
        self.as_axial().as_vec3(z)
    }

    pub fn as_axial(self) -> Axial {
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
        dir.as_vec2()
    }
}
