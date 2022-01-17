//! The crate is simulating fluid dynamics and is showing it in a user-friendly application built
//! upon [egui](crates.io/crates/egui) from where various parameters can be adjusted.
//! The simulation is grid and CPU-based. The results of the simulation are saved into the
//! `rendered_images/` directory which is located directly under the project's root directory. The results
//! are also shown during the processing of the simulation inside the applicaiton.

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#[macro_use]

/// The `fluid` module is defining the fluid's dynamics and acts
/// as if the image's walls are constraining the fluid in a container.
pub mod fluid;

/// Renderes the simulation in images which are stored in the `equilibrium/rendered_images` folder
/// (directly below the project's root directory)
pub mod renderer;

/// The fluid simulator's entry point for the GUI.
/// It is based on the emilk's egui template: [here](https://github.com/emilk/eframe_template)
pub mod app;

/// More widgets related to the fluid simulation such as number of iterations, change of colours,
/// etc.
pub mod app_widgets;
