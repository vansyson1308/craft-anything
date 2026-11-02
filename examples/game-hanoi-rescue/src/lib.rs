//! Giải Cứu Cún: Hà Nội. A mobile-first extraction game in the old quarter of Hà Nội.
//!
//! Everything but `web` is plain Rust with no browser in sight, so the whole game runs (and is
//! tested) natively; `web` paints its frames on a canvas and feeds it touches and keys.

#![forbid(unsafe_code)]

pub mod content;
pub mod draw;
pub mod game;
pub mod geom;
pub mod input;
pub mod map;
pub mod meta;
pub mod raid;
pub mod render;

#[cfg(target_arch = "wasm32")]
mod web;
