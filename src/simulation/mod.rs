/// The `fluid` module is defining the fluid's dynamics and acts
/// as if the image's walls are constraining the fluid in a container.
#[macro_use]
pub mod fluid;

/// Renderes the simulation in images which are stored in the `equilibrium/rendered_images` folder
/// (directly below the project's root directory)
pub mod renderer;

/// Simulation configuration structures
pub mod configs;
