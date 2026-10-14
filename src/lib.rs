//! Stygian J-Ball: a DX-Ball clone with Ultima Underworld-flavoured pixel art.
//!
//! Everything except the window/input/audio glue in `main.rs` lives here and runs
//! headlessly: game rules (`game`), screen flow (`app`), a software pixel renderer
//! (`gfx`), cosmetic effects (`fx`) and procedural sound synthesis (`audio`).

pub mod app;
pub mod audio;
pub mod consts;
pub mod fx;
pub mod game;
pub mod gfx;
pub mod levels;
pub mod rng;
