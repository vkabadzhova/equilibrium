/// This module defines and manipulates the dynamics of the fluid.
///
/// The fluid initially acts as if the fluid is contrained by the image's "walls".
#[macro_use]
pub mod fluid;

/// Renderes the simulation and stores the result images in the `equilibrium/rendered_images` directory
/// (located directly under the project's root).
///
/// **Note:** The image's coordinates are from top to bottom and from left to right, i.e.,
/// the following pattern:
/// ```text
/// 0,0 | 0,1 | 0,2
/// 1,0 | 1,1 | 1,2
/// 2,0 | 2,1 | 2,2
///
/// ```
pub mod renderer;

/// Various obstacles defined by a vector with their points can be put into the simulation,
/// as long as the obstacle's points are inside the fluid's container. The fluid will avoid those.
pub mod obstacle;

/// Contains all configuration types for the project. The renderer and the fluid
/// use them to define their own behaviour, while the application's widgets store
/// a copy of those same configs, so they can further manipulate and interact with
/// the drivers (a.k.a. the [`Renderer`](crate::simulation::renderer::Renderer)
/// and the [`Fluid`](crate::simulation::fluid::Fluid).
pub mod configs;
