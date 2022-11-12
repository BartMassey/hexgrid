/*!

API for working with hex grid coordinates as commonly used in
game boards.

This code is currently opinionated. The crate exposes q-r
axial coordinates as the primary coordinate type, in a
"right-handed" (*q* increasing east, *r* increasing north)
flat-topped coordinate system.  It also provides cube
coordinates and flat-topped hexes.

Pointy-topped hexes and various other coordinate systems
should probably be an option: patches welcome.

This crate is almost entirely derived from the excellent
[discussion](https://www.redblobgames.com/grids/hexagons/)
of hex grids at the Red Blob Games website. Many thanks to
Amit Patel for a definitive and crystal clear exposition.

!*/

use std::convert::TryFrom;
use std::fmt::Debug;

pub use num;
use num::{Float, Num};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// "Compass" directions on the flat-topped hex grid.
pub enum Direction {
    /// Northeast
    NE,
    /// North
    N,
    /// Northwest
    NW,
    /// Southwest
    SW,
    /// South
    S,
    /// Southeast
    SE,
}

/// Error indicating that specified direction coordinate
/// is out of range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectionError(pub usize);

impl std::fmt::Display for DirectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "direction error: {}", self.0)
    }
}

impl std::error::Error for DirectionError {}

impl TryFrom<usize> for Direction {
    type Error = DirectionError;
    fn try_from(d: usize) -> Result<Self, Self::Error> {
        use Direction::*;
        const DIRNS: [Direction; 6] = [N, NW, SW, S, SE, NE];
        if d >= DIRNS.len() {
            return Err(DirectionError(d));
        }
        Ok(DIRNS[d])
    }
}

impl From<Direction> for usize {
    fn from(d: Direction) -> usize {
        d as usize
    }
}

fn num_const<T: Num>(s: &str) -> T {
    T::from_str_radix(s, 10)
        .unwrap_or_else(|_| panic!("no {} for numeric type", s))
}

/// Hex grid location in axial coordinates, parameterized by
/// the number type used for coordinates. This is
/// transparent, but in normal use there is no need to look
/// at its internals.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct HexCoord<T> {
    pub q: T,
    pub r: T,
}

// Constant macros for Cartesian coordinate calculations.
macro_rules! nc {
    ($numstr:literal, $u:ty) => {
        num_const::<$u>($numstr)
    };
}

macro_rules! half {
    ($u:ty) => {
        nc!("0.5", $u)
    };
}

macro_rules! quarter {
    ($u:ty) => {
        nc!("0.25", $u)
    };
}

macro_rules! sqrt3 {
    ($u:ty) => {
        nc!("3.0", $u).sqrt()
    };
}

macro_rules! sqrt3d2 {
    ($u:ty) => {
        half!($u) * sqrt3!($u)
    };
}

impl<T: Num> HexCoord<T> {
    /// Make a hex axial coordinate, in a "right-handed"
    /// flat-topped coordinate system (`q` increasing east,
    /// `r` increasing north).
    pub fn new(q: T, r: T) -> Self {
        HexCoord { q, r }
    }

    /// Axial coordinate of hex neighboring `self` in
    /// direction `d`.
    pub fn neighbor(self, d: Direction) -> Self {
        use Direction::*;
        match d {
            NE => {
                HexCoord::new(self.q + num::one(), self.r + num::one())
            }
            N => HexCoord::new(self.q, self.r + num::one()),
            NW => HexCoord::new(self.q - num::one(), self.r),
            SW => {
                HexCoord::new(self.q - num::one(), self.r - num::one())
            }
            S => HexCoord::new(self.q, self.r - num::one()),
            SE => HexCoord::new(self.q + num::one(), self.r),
        }
    }

    /// "Manhattan distance" from `self` to `b`.
    pub fn distance(self, b: Self) -> T
    where
        T: PartialOrd + Clone,
    {
        HexCubeCoord::distance(self.into(), b.into())
    }

    /// `(x, y)` Cartesian coordinates of `HexCoord` center,
    /// for flat-topped pixels in a right-handed coordinate
    /// system (`x` increasing east, `y` increasing north)
    /// with hexes of unit width.
    pub fn cartesian_center<U: Float>(self) -> (U, U)
    where
        T: Into<U>,
    {
        let q = self.q.into();
        let r = self.r.into();
        let x = num_const::<U>("0.75") * q;
        let y = -sqrt3d2!(U) * (half!(U) * q - r);
        (x, y)
    }

    /// `(x, y)` Cartesian coordinates of `HexCubeCoord`
    /// corners, for flat-topped pixels in a right-handed
    /// coordinate system (`x` increasing east, `y`
    /// increasing north) with hexes of unit width. Corners
    /// are given counterclockwise starting with the
    /// easternmost.
    pub fn cartesian_corners<U: Float>(self) -> [(U, U); 6]
    where
        T: Into<U>,
    {
        let (x, y) = self.cartesian_center();
        [
            (half!(U) + x, y),
            (quarter!(U) + x, half!(U) * sqrt3d2!(U) + y),
            (-quarter!(U) + x, half!(U) * sqrt3d2!(U) + y),
            (-half!(U) + x, y),
            (-quarter!(U) + x, -half!(U) * sqrt3d2!(U) + y),
            (quarter!(U) + x, -half!(U) * sqrt3d2!(U) + y),
        ]
    }
}

#[cfg(test)]
mod test_cartesian {
    use crate::*;

    fn approx_eq(x0: f64, x: f64) -> bool {
        (x0 - x).abs() < 0.001
    }

    #[test]
    fn test_cartesian_center() {
        assert_eq!((0.0, 0.0), HexCoord::new(0, 0).cartesian_center());

        let ys = 0.25 * f64::sqrt(3.0);
        let yl = 2.0 * ys;
        let xs = 0.75f64;
        let centers = [
            ((1, 1), (xs, ys)),
            ((0, 1), (0.0, yl)),
            ((-1, 0), (-xs, ys)),
            ((-1, -1), (-xs, -ys)),
            ((0, -1), (0.0, -yl)),
            ((1, 0), (xs, -ys)),
        ];
        for &((q, r), (xt, yt)) in &centers {
            let (x, y) = HexCoord::new(q, r).cartesian_center();
            assert!(approx_eq(x, xt));
            assert!(approx_eq(y, yt));
        }

        let (x, y) = HexCoord::new(1, 2).cartesian_center();
        assert!(approx_eq(x, 0.75));
        assert!(approx_eq(y, 0.75 * f64::sqrt(3.0)));
    }

    #[test]
    fn test_cartesian_corners() {
        let cy: f64 = 0.25 * 3.0.sqrt();

        let test = move |corners: HexCoord<i32>,
                         tcorners: [(f64, f64); 6]| {
            let ccs = corners.cartesian_corners();
            let iter = tcorners.iter().copied();
            let citer = ccs.iter().copied();
            for ((x, y), (cx, cy)) in iter.zip(citer) {
                assert!(approx_eq(x, cx));
                assert!(approx_eq(y, cy));
            }
        };

        let start = HexCoord::new(0, 0);
        let tcorners = [
            (0.5, 0.0),
            (0.25, cy),
            (-0.25, cy),
            (-0.5, 0.0),
            (-0.25, -cy),
            (0.25, -cy),
        ];
        test(start, tcorners);

        let target = start.neighbor(Direction::NE);
        let tcorners = [
            (1.25, cy),
            (1.0, 2.0 * cy),
            (0.5, 2.0 * cy),
            (0.25, cy),
            (0.5, 0.0),
            (1.0, 0.0),
        ];
        test(target, tcorners);
    }
}

#[test]
fn test_neighbor_axial() {
    use Direction::*;
    let dirns = vec![N, SW, S, SE, NE, N, NW, S];
    let start = HexCoord::new(0i8, 0i8);
    let mut cur = start;
    for d in dirns {
        cur = cur.neighbor(d);
    }
    assert_eq!(cur, start);
}

/// Hex cube coordinates. This is opaque, to protect the
/// coordinate invariant; this coordinate system
/// is redundant, so the coordinate invariant
///
/// > `x + y + z != 0`
///
/// is maintained internally.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct HexCubeCoord<T> {
    x: T,
    y: T,
    z: T,
}

/// Error indicating that the cube invariant
///
/// > `x + y + z == 0`
///
/// has been violated by the given `HexCubeCoord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CubeInvariantError<T: Num> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Num + Debug> std::fmt::Display for CubeInvariantError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "cube invariant violation: x:{:?}, y:{:?}, z:{:?}",
            self.x, self.y, self.z,
        )
    }
}

impl<T: Num + Debug> std::error::Error for CubeInvariantError<T> {}

impl<T: Num> HexCubeCoord<T> {
    /// Make a cube coordinate, checking that the invariant
    /// is satisfied.
    pub fn new(
        x: T,
        y: T,
        z: T,
    ) -> Result<HexCubeCoord<T>, CubeInvariantError<T>>
    where
        T: Clone + Debug,
    {
        if x.clone() + y.clone() + z.clone() == num::zero() {
            Ok(HexCubeCoord { x, y, z })
        } else {
            Err(CubeInvariantError { x, y, z })
        }
    }

    /// Make a cube coordinate without checking for an invariant
    /// error. This crate will behave strangely if the
    /// invariant is not satisfied, so only use this when
    /// certain.
    pub fn new_unchecked(x: T, y: T, z: T) -> Self {
        HexCubeCoord { x, y, z }
    }

    /// Return the cube coordinates.
    pub fn coords(self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }

    /// "Manhattan distance" from `self` to `b`.
    pub fn distance(self, b: Self) -> T
    where
        T: PartialOrd,
    {
        fn abs_diff<T: Num + PartialOrd>(a: T, b: T) -> T {
            if a <= b {
                b - a
            } else {
                a - b
            }
        }

        let x = abs_diff(self.x, b.x);
        let y = abs_diff(self.y, b.y);
        let z = abs_diff(self.z, b.z);
        (x + y + z) / num_const("2")
    }

    /// Coordinate of hex neighboring `self` in direction
    /// `d`. See `HexCoord::neighbor()` for details.
    pub fn neighbor(self, d: Direction) -> Self
    where
        T: Clone,
    {
        HexCoord::from(self).neighbor(d).into()
    }

    /// Cartesian coordinates of `HexCubeCoord` center. See
    /// `HexCoord::cartesian_center()` for details.
    pub fn cartesian_center<U: Float>(self) -> (U, U)
    where
        T: Into<U>,
    {
        <HexCoord<T>>::from(self).cartesian_center()
    }

    /// Cartesian coordinates of `HexCubeCoord` corners. See
    /// `HexCoord::cartesian_corners()` for details.
    pub fn cartesian_corners<U: Float>(self) -> [(U, U); 6]
    where
        T: Into<U>,
    {
        <HexCoord<T>>::from(self).cartesian_corners()
    }
}

#[test]
fn test_neighbor_cube() {
    use Direction::*;
    let dirns = vec![N, SW, S, SE, NE, N, NW, S];
    let start = HexCubeCoord::new_unchecked(0i8, 0i8, 0i8);
    let mut cur = start;
    for d in dirns {
        cur = cur.neighbor(d);
    }
    assert_eq!(cur, start);
}

#[test]
fn test_distance_cube() {
    let start = HexCubeCoord::new(0.0f32, 0.0f32, 0.0f32).unwrap();
    let end = HexCubeCoord::new(-1.0f32, 3.0f32, -2.0f32).unwrap();
    assert_eq!(3.0f32, start.distance(end));
}

impl<T: Num + Clone> From<HexCoord<T>> for HexCubeCoord<T> {
    fn from(c: HexCoord<T>) -> Self {
        let cl = c.clone();
        let y = num::zero::<T>() - cl.q - cl.r;
        HexCubeCoord::new_unchecked(c.q, y, c.r)
    }
}

impl<T: Num> From<HexCubeCoord<T>> for HexCoord<T> {
    fn from(c: HexCubeCoord<T>) -> Self {
        HexCoord::new(c.x, c.z)
    }
}
