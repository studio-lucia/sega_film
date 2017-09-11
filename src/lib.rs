//! This crate contains tools to parse, manipulate, and write Sega FILM containers.
//! Sega FILM is a video container format which was used by Sega in Saturn games
//! released between 1994 and 2000; it's the most common of the several video
//! formats used on the Saturn.

pub mod codec;
/// Contains tools for parsing the FILM container.
pub mod container;
mod utils;
