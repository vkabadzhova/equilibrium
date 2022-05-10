use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::fluid::ContainerWall;
use crate::simulation::fluid::Fluid;
use crate::simulation::obstacle::ObstaclesType;
use std::fs;
use std::sync::mpsc::{Receiver, Sender};

/// Creates a name of the a rendered density file based on the frame number and
/// a given directory
macro_rules! density_img_path {
    ($save_into_dir:expr, $frame_number:expr) => {
        &($save_into_dir.clone().to_owned() + "/density" + &$frame_number.to_string() + ".jpg")
    };
}

/// Saves the state of the current step of the fluid. The purpose of this structure is to be sent
/// over from [`CurrentSimulation`](crate::simulation::current_simulation::CurrentSimulation) to
/// the [`Renderer`](crate::simulation::renderer::Renderer).
pub struct FluidStep {
    fluid: Fluid,
    frame_number: i64,
}

/// Performs the simulation of the fluid. It sends a
/// [`FluidStep`](crate::simulation::current_simulation::FluidStep) over a [`std::sync::mpsc`]
/// channel directly to the [`Renderer`].
#[derive(Clone)]
pub struct CurrentSimulation {
    /// The Renderer owns the fluid that it simulates
    pub fluid: Fluid,

    /// Buffered fluid configurations for the next run. The configurations of the fluid are not
    /// changed while the fluid is being simulated.
    pub fluid_configs: FluidConfigs,

    /// Buffered simulation configurations for the next run. The configurations of the fluid are
    /// not changed while the fluid is being simulated.
    pub simulation_configs: SimulationConfigs,

    /// Collection of all the obstacles. To update the fluid's behaviour to correspond to the
    /// obstacles, use [`update_obstacles`].
    pub obstacles: Vec<ObstaclesType>,
}

impl Default for CurrentSimulation {
    fn default() -> Self {
        Self {
            fluid: Fluid::default(),
            fluid_configs: FluidConfigs::default(),
            simulation_configs: SimulationConfigs::default(),
            obstacles: vec![ObstaclesType::Rectangle(
                crate::simulation::obstacle::Rectangle::default(),
            )],
        }
    }
}

impl CurrentSimulation {
    /// Runs the fluid simulation
    pub fn simulate(&mut self, tx: Sender<FluidStep>) {
        self.mark_fluid_obstacles();
        for i in 0..self.fluid.simulation_configs.frames {
            if self.fluid.fluid_configs.has_perlin_noise {
                self.fluid.add_noise();
            }

            self.fluid.step();

            tx.send(FluidStep {
                fluid: self.fluid.clone(),
                frame_number: i,
            })
            .unwrap();
        }
    }

    /// After altering the obstacles list. Refresh the fluid's configuration regarding its
    /// obstacles.
    fn mark_fluid_obstacles(&mut self) {
        for obstacle in self.obstacles.iter_mut() {
            self.fluid.fill_obstacle(obstacle);
        }
    }
}

/// Listens for a signal to render the image with the next fluid state. After that it sends a
/// signal to the [`App`](crate::app::App) to display it.
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

    fn render_image(&self, fluid_step: FluidStep) {
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
        for i in 0..max_frames - 1 {
            self.render_image(
                simulation_rx
                    .recv()
                    .expect("Problem occured while listening for FluidStep"),
            );

            rendering_tx
                .send(i)
                .expect("Could not properly send current frame number through rendering_tx");
        }
    }
}
