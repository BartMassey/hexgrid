/*!

API for working with a hexagonal grid as commonly used in
game boards. This crate provides manipulations for hex grid
coordinates as well as a data structure for hex grid
storage.

This code is currently opinionated: it exposes x-z axial
coordinates, cube coordinates, and flat-topped
hexes. Pointy-topped hexes and various other coordinate
systems should probably be an option: patches welcome.

This crate is almost entirely derived from the excellent
[discussion](https://www.redblobgames.com/grids/hexagons/)
of hex grids at the Red Blob Games website. Many thanks to
Amit Patel for a definitive and crystal clear exposition.

!*/

use std::fmt::Debug;

use num::Num;

/// Hex grid location, parameterized by the number type used
/// for coordinates. This is transparent, but in normal use
/// there is no need to look at its internals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
}

/// Used internally for various calculations, and exposed in
/// case it is useful. Note that this coordinate system is
/// redundant: the coordinate invariant
///
/// > `x + y + z != 0`
///
/// will be maintained by code in this crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

impl<T: Num + Copy> From<HexCoord<T>> for HexCubeCoord<T> {
    fn from(c: HexCoord<T>) -> Self {
        HexCubeCoord::new(c.x, c.x - c.z, c.z)
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
        writeln!(f, "cube invariant violation: {:?}", self)
    }
}

impl<T: Num + Debug> std::error::Error for CubeInvariantError<T> {}

impl<T: Num + Copy> std::convert::TryFrom<HexCubeCoord<T>>
    for HexCoord<T>
{
    type Error = CubeInvariantError<T>;
    fn try_from(c: HexCubeCoord<T>) -> Result<Self, Self::Error> {
        if c.x + c.y + c.z != num::zero() {
            return Err(CubeInvariantError(c));
        }
        Ok(HexCoord::new(c.x, c.z))
    }
}
