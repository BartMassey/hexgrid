# hexgrid: Rust Hex Grid library crate
Bart Massey

This work-in-progress crate models a hex grid as commonly
used in gaming. This crate is almost entirely derived from the excellent
[discussion](https://www.redblobgames.com/grids/hexagons/)
of hex grids at the Red Blob Games website. You will want to
read that before trying to use this crate.

This code is currently opinionated. The crate exposes
positive y (right-handed coordinates) x-z axial coordinates
(as the primary coordinate type) and cube coordinates, with
flat-topped hexes. Pointy-topped hexes and various other
coordinate systems should probably be an option: patches
welcome.

Many thanks to Amit Patel of Red Blob Games for a definitive
and crystal clear exposition.
