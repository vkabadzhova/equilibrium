/// This module defines and manipulates the dynamics of the fluid.
///
/// The fluid initially acts as if the fluid is contrained by the image's "walls".
#[macro_use]
pub mod fluid;

/// Renderes the simulation and stores the result images in the `equilibrium/rendered_images` directory
/// (located directly under the project's root)
pub mod renderer;

/// Contains all configuration types for the project. The renderer and the fluid
/// use them to define their own behaviour, while the application's widgets store 
/// a copy of those same configs, so they can further manipulate and interact with
/// the drivers (a.k.a. the [`Renderer`](crate::simulation::renderer::Renderer)
/// and the [`Fluid`](crate::simulation::fluid::Fluid).
pub mod configs;
