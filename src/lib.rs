/*!

API for working with hex grid coordinates as commonly used in
game boards.

This code is currently opinionated. The crate exposes
positive y (right-handed coordinates) x-z axial coordinates
(as the primary coordinate type), cube coordinates, and
flat-topped hexes. Pointy-topped hexes and various other
coordinate systems should probably be an option: patches
welcome.

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
/// "Compass" directions on the hex grid.
pub enum Dirn {
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
    /// Northeast
    NE,
}

/// Error indicating that specified direction coordinate
/// is out of range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirnError(pub usize);

impl std::fmt::Display for DirnError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "direction error: {}", self.0)
    }
}

impl std::error::Error for DirnError {}

impl TryFrom<usize> for Dirn {
    type Error = DirnError;
    fn try_from(d: usize) -> Result<Self, Self::Error> {
        use Dirn::*;
        const DIRNS: [Dirn; 6] = [N, NW, SW, S, SE, NE];
        if d >= DIRNS.len() {
            return Err(DirnError(d));
        }
        Ok(DIRNS[d])
    }
}

impl From<Dirn> for usize {
    fn from(d: Dirn) -> usize {
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

impl<T: Num> HexCoord<T> {
    /// Make a hex coordinate.
    pub fn new(q: T, r: T) -> Self {
        HexCoord { q, r }
    }

    /// Coordinate of hex neighboring `self` in direction `d`.
    pub fn neighbor(self, d: Dirn) -> Self {
        use Dirn::*;
        match d {
            N => HexCoord::new(self.q, self.r - num::one()),
            NW => HexCoord::new(self.q - num::one(), self.r),
            SW => {
                HexCoord::new(self.q - num::one(), self.r + num::one())
            }
            S => HexCoord::new(self.q, self.r + num::one()),
            SE => HexCoord::new(self.q + num::one(), self.r),
            NE => {
                HexCoord::new(self.q + num::one(), self.r - num::one())
            }
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
    /// system (`x` to the right, `y` up) with hexes of unit
    /// width.
    pub fn cartesian_center<U: Float>(self) -> (U, U)
    where
        T: Into<U>,
    {
        let q = self.q.into();
        let r = self.r.into();
        let x = num_const::<U>("1.5") * q;
        let y = num_const::<U>("3.0").sqrt()
            * (num_const::<U>("0.5") * q + r);
        (x, y)
    }
}

#[test]
fn test_cartesian_center() {
    assert_eq!((0.0, 0.0), HexCoord::new(0, 0).cartesian_center());

    let approx_eq = |x0: f64, x: f64| (x0 - x).abs() < 0.001;

    assert_eq!((0.0, 0.0), HexCoord::new(0, 0).cartesian_center());
    let (x, y) = HexCoord::new(1, 2).cartesian_center();
    assert!(approx_eq(x, 1.5));
    assert!(approx_eq(y, 2.5 * 3.0.sqrt()));
}

#[test]
fn test_neighbor_axial() {
    use Dirn::*;
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

    /// Coordinate of hex neighboring `self` in direction `d`.
    pub fn neighbor(self, d: Dirn) -> Self
    where
        T: Clone,
    {
        HexCoord::from(self).neighbor(d).into()
    }

    /// `(x, y)` Cartesian coordinates of `HexCubeCoord` center,
    /// for flat-topped pixels in a right-handed coordinate
    /// system (`x` to the right, `y` up) with hexes of unit
    /// width.
    pub fn cartesian_center<U: Float>(self) -> (U, U)
    where
        T: Into<U>,
    {
        <HexCoord<T>>::from(self).cartesian_center()
    }
}

#[test]
fn test_neighbor_cube() {
    use Dirn::*;
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
