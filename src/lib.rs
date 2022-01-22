//! The crate is simulating fluid dynamics and is showing it in a user-friendly application built
//! upon [egui](crates.io/crates/egui) from where various parameters can be adjusted.
//! The simulation is grid and CPU-based. The results of the simulation are saved into the
//! `rendered_images/` directory which is located directly under the project's root directory. The results
//! are also shown during the processing of the simulation inside the applicaiton.

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#[macro_use]

/// All of the application's behaviour. It is biult upon the [`egui's template`](emilk.github.com/egui)
pub mod app;

/// Space for all the structures that concern the simulation
pub mod simulation;
