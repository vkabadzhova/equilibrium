//! This crate is simulating fluid dynamics

//#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#[macro_use]

/// The `fluid` module is defininng the fluid's dynamics and acts
/// as if it is in a container since it is constrained by walls
pub mod fluid;

/// The renderer is the visualizer of the project
pub mod renderer;
