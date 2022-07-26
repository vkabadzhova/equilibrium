use crate::simulation::fluid::ContainerWall;
use crate::simulation::fluid::Fluid;
use crate::simulation::obstacle::ObstaclesType;
use simplelog::*;
use std::fs;
use std::sync::mpsc::{Receiver, Sender};

/// Creates a name of the a rendered density file based on the frame number and
/// a given directory
macro_rules! density_img_path {
    ($save_into_dir:expr, $frame_number:expr) => {
        &($save_into_dir.clone().to_owned() + "/density" + &$frame_number.to_string() + ".jpg")
    };
}

use crate::simulation::animation;
use crate::simulation::animation::{normalized_height, normalized_width};
pub(crate) use density_img_path;

/// Saves the state of the current step of the fluid. The purpose of this structure is to be sent
/// over from [`CurrentSimulation`](crate::simulation::renderer_helpers::CurrentSimulation) to
/// the [`Renderer`](crate::simulation::renderer::Renderer).
pub struct FluidStep {
    fluid: Fluid,
    frame_number: i64,
}

/// Performs the simulation of the fluid. It sends a
/// [`FluidStep`](crate::simulation::renderer_helpers::FluidStep) over a [`std::sync::mpsc`]
/// channel directly to the [`crate::simulation::renderer::Renderer`].
#[derive(Clone)]
pub struct CurrentSimulation {
    /// The fluid state in the current simulation.
    pub fluid: Fluid,

    /// Animate obstacles with Game of Life algorithm
    pub animate_obstacles_using_game_of_life: bool,

    /// Collection of all the obstacles. To update the fluid's behaviour to correspond to the
    /// obstacles, use [`crate::simulation::renderer::Renderer::update_configs()`].
    pub obstacles: Vec<ObstaclesType>,
}

impl Default for CurrentSimulation {
    fn default() -> Self {
        Self {
            fluid: Fluid::default(),
            obstacles: vec![ObstaclesType::Rectangle(
                crate::simulation::obstacle::Rectangle::default(),
            )],
            animate_obstacles_using_game_of_life: false,
        }
    }
}

impl CurrentSimulation {
    pub(crate) fn new(animate_obstacles_using_game_of_life: bool) -> Self {
        Self {
            fluid: Fluid::default(),
            obstacles: vec![ObstaclesType::Rectangle(
                crate::simulation::obstacle::Rectangle::default(),
            )],
            animate_obstacles_using_game_of_life,
        }
    }
}

impl CurrentSimulation {
    fn check_animation_obstacles(&mut self) {
        let min_num_obstacles_always_alive = 20;
        if self.animate_obstacles_using_game_of_life && self.obstacles.len() < min_num_obstacles_always_alive {
            let size: i64 = Fluid::default().simulation_configs.size as i64;
            self.obstacles = animation::cell_bitmap_to_obstacles(
                &animation::random_cell_bitmap(size, size),
                normalized_width(size),
                normalized_height(size),
            )
        }
    }
    /// Runs the fluid simulation
    pub fn simulate(&mut self, tx: Sender<FluidStep>) {
        for i in 0..self.fluid.simulation_configs.frames {
            self.check_animation_obstacles();
            self.draw_obstacles();

            if self.fluid.fluid_configs.has_perlin_noise {
                self.fluid.add_noise();
            }

           self.fluid.step();

            tx.send(FluidStep {
                fluid: self.fluid.clone(),
                frame_number: i,
            })
            .unwrap();

            simplelog::debug!(
                "CurrentSimulation: sent a signal that a frame {} is ready",
                i
            );

            self.undraw_obstacles();
        }
    }

    fn undraw_obstacles(&mut self) {
        for obstacle in self.obstacles.iter_mut() {
            self.fluid.fill_obstacle(obstacle,ContainerWall::NoWall);
        }
    }

    /// After altering the obstacles list. Refresh the fluid's configuration regarding its
    /// obstacles.
    /// Executes animation for each obstacle (if one is set) and fills the inner cells of each.
    /// Seems to me that it is a better approach to, first, animate all obstacles (i.e call
    /// relocate() on each of them according to some policy), and only after tht to draw the whole
    /// obstacle vector.
    fn draw_obstacles(&mut self) {
        if self.animate_obstacles_using_game_of_life {
            self.obstacles = animation::game_of_life(
                &self.obstacles,
                self.fluid.simulation_configs.size as i64,
                self.fluid.simulation_configs.size as i64,
            );
        }
        for obstacle in self.obstacles.iter_mut() {
            self.fluid.fill_obstacle(obstacle, ContainerWall::DefaultWall);
        }
    }
}

/// Listens for a signal to render the image with the next fluid state. After that it sends a
/// signal to the [`App`](crate::app::app::App) to display it.
#[derive(Clone)]
pub struct RenderingListener {
    /// The directory in which the rendered images that result from the simulation are stored.
    pub save_into_dir: String,

    /// The color of all obstacles. Obstacles cannot be set individual colors.
    pub obstacles_color: eframe::egui::Color32,
}

impl Default for RenderingListener {
    fn default() -> Self {
        Self {
            save_into_dir: RenderingListener::make_save_into_dir("rendered_images"),
            obstacles_color: eframe::egui::Color32::RED,
        }
    }
}

impl RenderingListener {
    /// Creates the default directory in which the result images will be saved.
    pub fn make_save_into_dir(dir_name: &str) -> String {
        let project_root = project_root::get_project_root()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        project_root + "/" + &dir_name
    }

    /// Creates the file where the result image is rendered.
    fn render_image(&self, fluid_step: FluidStep) {
        simplelog::debug!(
            "RenderingListener: Received a signal that a frame is ready! Starting to render; Saving into: {}",
            density_img_path!(self.save_into_dir, fluid_step.frame_number)
        );
        let fluid = fluid_step.fluid;

        let world_rgba = [
            fluid.fluid_configs.world_color.r(),
            fluid.fluid_configs.world_color.g(),
            fluid.fluid_configs.world_color.b(),
            fluid.fluid_configs.world_color.a(),
        ];

        let fluid_rgba = [
            fluid.fluid_configs.fluid_color.r(),
            fluid.fluid_configs.fluid_color.g(),
            fluid.fluid_configs.fluid_color.b(),
            fluid.fluid_configs.fluid_color.a(),
        ];

        let obstacles_rgba = [
            self.obstacles_color.r(),
            self.obstacles_color.g(),
            self.obstacles_color.b(),
            self.obstacles_color.a(),
        ];

        let mut imgbuf =
            image::ImageBuffer::new(fluid.simulation_configs.size, fluid.simulation_configs.size);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = fluid.density[idx!(x, y, fluid.simulation_configs.size)];
            let cell_type = fluid.cells_type[idx!(x, y, fluid.simulation_configs.size)];

            if cell_type == ContainerWall::DefaultWall {
                *pixel = image::Rgba([
                    obstacles_rgba[0],
                    obstacles_rgba[1],
                    obstacles_rgba[2],
                    obstacles_rgba[3],
                ]);
            } else if density != 0.0 && cell_type == ContainerWall::NoWall {
                *pixel = image::Rgba([
                    (density * fluid_rgba[0] as f32) as u8,
                    fluid_rgba[1],
                    density as u8,
                    1,
                ]);
            } else {
                *pixel = image::Rgba(world_rgba);
            }
        }

        if !std::path::Path::new(&self.save_into_dir).exists() {
            fs::create_dir(&self.save_into_dir)
                .expect("Error while creating a directory to store the simulation results.");
        }

        imgbuf
            .save(density_img_path!(
                self.save_into_dir,
                fluid_step.frame_number
            ))
            .expect("Coulnt't save density image");
    }

    /// Listens for a signal from [`CurrentSimulation`] that a frame is ready, and then renders it.
    pub fn listen(
        &self,
        max_frames: i64,
        simulation_rx: Receiver<FluidStep>,
        rendering_tx: Sender<i64>,
    ) {
        for i in 0..max_frames {
            self.render_image(simulation_rx.recv().expect(
                "Simulation sender has been disconnected. Cannot listen through RenderingListener",
            ));

            rendering_tx
                .send(i)
                .expect("Could not properly send current frame number through rendering_tx");

            simplelog::debug!("RenderingListener: frame {} is rendered!", i);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::obstacle::Obstacle;
    use crate::simulation::{fluid::ContainerWall, renderer_helpers::CurrentSimulation};

    fn calc_height<T>(a: (T, T), b: (T, T)) -> T::Output
    where
        T: std::ops::Sub,
    {
        b.0 - a.0
    }

    fn calc_width<T>(a: (T, T), b: (T, T)) -> T::Output
    where
        T: std::ops::Sub,
    {
        b.0 - a.0
    }

    #[test]
    fn mark_fluid_obstacles_default_current_simulation() {
        let mut current_simulation = CurrentSimulation::default();
        current_simulation.draw_obstacles();
        let default_walls_real_count = current_simulation
            .fluid
            .cells_type
            .iter()
            .filter(|&el| el == &ContainerWall::DefaultWall)
            .count();

        let size = current_simulation.fluid.simulation_configs.size;
        let image_parameter = 2 * (size + (size - 2));

        // Sums up the area of all obstacles. NB: Assumes all obstacles are rectangles
        let obstacles_area = current_simulation.obstacles.iter().fold(0, |acc, x| {
            acc + calc_width(
                x.clone().get_approximate_points()[0],
                x.clone().get_approximate_points()[1],
            ) * calc_height(
                x.clone().get_approximate_points()[0],
                x.clone().get_approximate_points()[1],
            )
        });

        // NB: We explicitly know how that the obstacle does not overlap with the perimeter.
        let expected_count = obstacles_area + i64::from(image_parameter);

        // Safety note: Default wall size is way smaller than usize.
        assert_eq!(default_walls_real_count, expected_count as usize);
    }
}
