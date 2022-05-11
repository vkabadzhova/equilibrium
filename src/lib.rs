//! The crate is simulating fluid dynamics and is showing it in a user-friendly
//! application built upon [egui](crates.io/crates/egui) from where various
//! parameters regarding the [simulation](crate::app::widgets::simulation_widget)
//! (such as speed /size of step/, length
//! /number of frames/, etc), the [fluid](crate::app::widgets::fluid_widget) (diffusion rate, viscousity, colour),
//! etc. can be adjusted. The simulation is grid- and CPU-based. The results of
//! the simulation are saved in the `rendered_images/` directory which is
//! located directly under the project's root directory. The results are also
//! shown during the processing of the simulation inside the applicaiton.
//!
//!
//! <a href="https://ibb.co/CtNC38P"><img src="https://i.ibb.co/H21wf4h/home-screen.png" alt="home-screen" border="0" width=448 height=359 class="center"></a>
#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#[macro_use]

/// The application's space. The app itsels is built upon the [`emilk`](github.com/emilk)'s
/// [`eframe template`](https://github.com/emilk/eframe_template) for
/// [`egui`](github.com/emilk/egui). It is designed for easier interaction with
/// the simulation. From here you can adjust the parameters, visualize and control the
/// simulation.
pub mod app;

/// Space for all the structures that concern the fluid's behaviour during the simulation, i.e.
/// the [`Renderer`](crate::simulation::renderer::Renderer) and the [`Fluid`](crate::simulation::fluid::Fluid)
/// and their [configuratoins](crate::simulation::configs)
pub mod simulation;
