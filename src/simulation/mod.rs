/// This module defines and manipulates the dynamics of the fluid.
///
/// The fluid initially acts as if the fluid is contrained by the image's "walls".
#[macro_use]
pub mod fluid;

/// Renderes the simulation and stores the result images in the `equilibrium/rendered_images` directory
/// (located directly under the project's root)
pub mod renderer;

/// Performs the current simulation only. After the simulation is done, the data is transfered to
/// the renderer. The renderer is one for the whole program. The current structure is developed for
/// syncronization purposes, when the simulation of the fluid behaviour should be delegated to
/// another thread.
pub mod current_simulation;

/// Various obstacles defined by a vector with their points can be put into the simulation,
/// as long as the obstacle's points are inside the fluid's container. The fluid will avoid those.
pub mod obstacle;

/// Contains all configuration types for the project. The renderer and the fluid
/// use them to define their own behaviour, while the application's widgets store
/// a copy of those same configs, so they can further manipulate and interact with
/// the drivers (a.k.a. the [`Renderer`](crate::simulation::renderer::Renderer)
/// and the [`Fluid`](crate::simulation::fluid::Fluid).
pub mod configs;
