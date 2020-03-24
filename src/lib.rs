/*!

API for working with hex grid coordinates as commonly used in
game boards.

This code is currently opinionated: it exposes positive y
(right-handed coordinates) x-z axial coordinates, cube
coordinates, and flat-topped hexes. Pointy-topped hexes and
various other coordinate systems should probably be an
option: patches welcome.

This crate is almost entirely derived from the excellent
[discussion](https://www.redblobgames.com/grids/hexagons/)
of hex grids at the Red Blob Games website. Many thanks to
Amit Patel for a definitive and crystal clear exposition.

!*/

use std::convert::TryFrom;
use std::fmt::Debug;

use num::Num;

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
#[derive(Debug)]
pub struct DirnError(usize);

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
        const DIRNS: [Dirn; 6] = [
            N, NW, SW, S, SE, NE,
        ];
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

/// Hex grid location in axial coordinates, parameterized by
/// the number type used for coordinates. This is
/// transparent, but in normal use there is no need to look
/// at its internals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexCoord<T> {
    pub x: T,
    pub z: T,
}

impl<T> HexCoord<T>
where
    T: Num,
{
    pub fn new(x: T, z: T) -> Self {
        HexCoord { x, z }
    }

    pub fn neighbor(self, d: Dirn) -> Self {
        use Dirn::*;
        match d {
            N => HexCoord::new(self.x, self.z - num::one()),
            NW => HexCoord::new(self.x - num::one(), self.z),
            SW => HexCoord::new(self.x - num::one(), self.z + num::one()),
            S => HexCoord::new(self.x, self.z + num::one()),
            SE => HexCoord::new(self.x + num::one(), self.z),
            NE => HexCoord::new(self.x + num::one(), self.z - num::one()),
        }
    }
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

/// Hex cube coordinates.This is transparent, but in normal
/// use there is no need to look at its internals. Note that
/// this coordinate system is redundant: the coordinate
/// invariant
///
/// > `x + y + z != 0`
///
/// will be maintained by code in this crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexCubeCoord<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> HexCubeCoord<T>
where
    T: Num,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        HexCubeCoord { x, y, z }
    }
}

impl<T: Num + Clone> HexCubeCoord<T> {
    pub fn neighbor(self, d: Dirn) -> Result<Self, CubeInvariantError<T>> {
        Ok(HexCoord::try_from(self)?.neighbor(d).into())
    }
}

#[test]
fn test_neighbor_cube() {
    use Dirn::*;
    let dirns = vec![N, SW, S, SE, NE, N, NW, S];
    let start = HexCubeCoord::new(0i8, 0i8, 0i8);
    let mut cur = start;
    for d in dirns {
        cur = cur.neighbor(d).unwrap();
    }
    assert_eq!(cur, start);
}

impl<T: Num + Clone> From<HexCoord<T>> for HexCubeCoord<T> {
    fn from(c: HexCoord<T>) -> Self {
        let cl = c.clone();
        let y = num::zero::<T>() - cl.x - cl.z;
        HexCubeCoord::new(c.x, y, c.z)
    }
}

/// Error indicating that the cube invariant
///
/// > `x + y + z == 0`
///
/// has been violated by the given `HexCubeCoord`.
#[derive(Debug)]
pub struct CubeInvariantError<T>(HexCubeCoord<T>);

impl<T: Num + Debug> std::fmt::Display for CubeInvariantError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "cube invariant violation: {:?}", self.0)
    }
}

impl<T: Num + Debug> std::error::Error for CubeInvariantError<T> {}

impl<T: Num + Clone> TryFrom<HexCubeCoord<T>>
    for HexCoord<T>
{
    type Error = CubeInvariantError<T>;
    fn try_from(c: HexCubeCoord<T>) -> Result<Self, Self::Error> {
        let cl = c.clone();
        if cl.x + cl.y + cl.z != num::zero() {
            return Err(CubeInvariantError(c));
        }
        Ok(HexCoord::new(c.x, c.z))
    }
}
