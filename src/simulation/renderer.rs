use crate::app::widgets::widgets_menu::SettingType;
use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::fluid::Fluid;
use crate::simulation::obstacle::Rectangle;
use std::fs;
use std::sync::mpsc::Sender;

/// Creates a name of the a rendered density file based on the frame number and
/// a given directory
macro_rules! density_img_path {
    ($rendered_images_dir:expr, $frame_number:expr) => {
        &($rendered_images_dir.clone().to_owned()
            + "/density"
            + &$frame_number.to_string()
            + ".jpg")
    };
}

pub(crate) use density_img_path;

use super::fluid::ContainerWall;
use super::obstacle::ObstaclesType;

/// Utility for visualization and interaction with the fluid simulation.
///
/// There are two ways to modify the simulation’s parameters while the simulation is
/// going: to rerun the simulation from the beginning, or two continue it with
/// the newly added parameters. Therefore parameters of the type `next_*_configs`
/// are used. In the first case, they are updating the current state of the
/// renderer live during the simulation (the current state is stored in the
/// struct’s member [`fluid`](crate::simulation::renderer::Renderer::fluid)), or,
/// otherwise, update it at the **beginning** of the new simulation (i.e.
/// frame 0).
pub struct Renderer {
    /// The Renderer owns the fluid that it simulates
    pub fluid: Fluid,

    /// The fluid configurations for the next run (either the next initial configs or
    /// live change of the current (depending on the configs. See the documentation
    /// of the [`Renderer`] struct)
    pub next_fluid_configs: FluidConfigs,

    /// The simulation configurations for the next run (either the next initial configs or
    /// live change of the current (depending on the configs. See the documentation
    /// of the [`Renderer`] struct)
    pub next_simulation_configs: SimulationConfigs,

    /// The directory in which the rendered images that result from the simulation
    /// are stored
    pub rendered_images_dir: String,

    /// Collection of all the obstacles. To update the fluid's behaviour to correspond to the
    /// obstacles, use [`update_obstacles`].
    pub obstacles: Vec<ObstaclesType>,
}

// Safety: No one besides us owns obstacle, so we can safely transfer the
// Fluid to another thread if T can be safely transferred.
unsafe impl Send for Renderer {}

impl Default for Renderer {
    /// Add default Renderer, containing an obstacle and default [`FluidConfigs`] and [`SimulationConfigs`]
    fn default() -> Self {
        let fluid = Fluid::default();
        let size = fluid.simulation_configs.size;
        let mut result = Self {
            next_fluid_configs: fluid.fluid_configs.clone(),
            next_simulation_configs: fluid.simulation_configs.clone(),
            // TODO
            //obstacles: vec![ObstaclesType::Rectangle(
            //    crate::simulation::obstacle::Rectangle::default(),
            //)],
            obstacles: Renderer::make_default_walls(size),
            fluid: fluid,
            rendered_images_dir: Renderer::make_rendered_images_dir(),
        };

        result.update_obstacles();
        result
    }
}

impl Renderer {
    /// Creates the default walls of the picture by the following rules:
    /// 1. 2 rows/columns per wall;
    /// 2. The North and the South walls take the corners.
    ///
    /// *NB: * Corners are additionally manipulated, and they contain one of the values of
    /// intersection. We cannot know exactly which one since while they are processed, a HashMap is
    /// used, and they are overwritten in undefined order. However, the unknowness of the value
    /// applies only for the inside corners, since the obstacles are desinged such that an obstacle
    /// overlaps with just a single row of the other.
    ///  
    /// ```text
    /// Illustration of the pattern:
    ///   |W  |E          |W  |E
    /// | N | N | N | N | N | N | <- N
    /// | ? | ? | S | S | ? | ? | <- S
    /// | W | E | - | - | W | E |
    /// | W | E | - | - | W | E |
    /// | ? | ? | N | N | ? | ? | <- N
    /// | S | S | S | S | S | S | <- S
    /// ```
    fn make_default_walls(size: u32) -> Vec<ObstaclesType> {
        vec![
            // left in Cartesian coordinate system
            ObstaclesType::Rectangle(Rectangle::new((0, 1), (1, i64::from(size) - 2), size)),
            // right in Cartesian coordinate system
            ObstaclesType::Rectangle(Rectangle::new(
                (i64::from(size) - 2, 1),
                (i64::from(size) - 1, i64::from(size) - 2),
                size,
            )),
            // down in Cartesian coordinate system
            ObstaclesType::Rectangle(Rectangle::new((0, 0), (i64::from(size) - 1, 1), size)),
            // up in Cartesian coordinate system
            ObstaclesType::Rectangle(Rectangle::new(
                (0, i64::from(size) - 2),
                (i64::from(size) - 1, i64::from(size) - 1),
                size,
            )),
        ]
    }

    fn make_rendered_images_dir() -> String {
        let project_root = project_root::get_project_root()
            .expect("couldn't get project's root")
            .to_str()
            .expect("couldn't convert project's root to string")
            .to_string();
        project_root + "/rendered_images"
    }

    /// Creates new Renderer
    pub fn new(fluid: Fluid) -> Renderer {
        Renderer {
            next_fluid_configs: fluid.fluid_configs.clone(),
            next_simulation_configs: fluid.simulation_configs.clone(),
            obstacles: Vec::new(),
            fluid: fluid,
            rendered_images_dir: Renderer::make_rendered_images_dir(),
        }
    }

    fn render_density(&self, frame_number: i64) {
        let world_rgba = [
            self.fluid.fluid_configs.world_color.r(),
            self.fluid.fluid_configs.world_color.g(),
            self.fluid.fluid_configs.world_color.b(),
            self.fluid.fluid_configs.world_color.a(),
        ];

        let fluid_rgba = [
            self.fluid.fluid_configs.fluid_color.r(),
            self.fluid.fluid_configs.fluid_color.g(),
            self.fluid.fluid_configs.fluid_color.b(),
            self.fluid.fluid_configs.fluid_color.a(),
        ];

        let mut imgbuf = image::ImageBuffer::new(
            self.fluid.simulation_configs.size,
            self.fluid.simulation_configs.size,
        );

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = self.fluid.density[idx!(x, y, self.fluid.simulation_configs.size)];
            let cell_type = self.fluid.cells_type[idx!(x, y, self.fluid.simulation_configs.size)];
            if cell_type != ContainerWall::NoWall {
                *pixel = image::Rgba([255, 0, 0, 1]);
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

        if !std::path::Path::new(&self.rendered_images_dir).exists() {
            fs::create_dir(&self.rendered_images_dir)
                .expect("Error while creating a directory to store the simulation results.");
        }

        imgbuf
            .save(density_img_path!(self.rendered_images_dir, frame_number))
            .expect("Coulnt't save density image");
    }

    /// Runs the fluid simulation
    pub fn simulate(&mut self, tx: Sender<i64>) {
        self.fluid = Fluid::new(self.next_fluid_configs, self.next_simulation_configs);
        self.update_obstacles();
        for i in 0..self.fluid.simulation_configs.frames {
            if self.fluid.fluid_configs.has_perlin_noise {
                self.fluid.add_noise();
            }
            self.fluid.step(&self.obstacles);

            self.render_density(i);
            tx.send(i)
                .expect("couln't send singal for rendered image to App's thread");
        }
    }

    /// After altering the obstacles list. Refresh the fluid's configuration by using that
    /// function.
    pub fn update_obstacles(&mut self) {
        for obstacle in self.obstacles.iter() {
            self.fluid.fill_obstacle(obstacle);
        }
    }

    /// Updates the configurations for the next simulation. As an example,
    /// during rendering, the configurations new configurations may be saved in
    /// the renderer, but will only be applied after the beginning of the next simulation
    pub fn update_initial_configs(&mut self, settings_menu: &Vec<SettingType>) {
        for setting in settings_menu.iter() {
            match setting {
                SettingType::Fluid(fluid_ui_config) => {
                    self.next_fluid_configs = fluid_ui_config.fluid_configs
                }
                SettingType::Simulation(simulation_ui_config) => {
                    self.next_simulation_configs = simulation_ui_config.simulation_configs
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::widgets::fluid_widget::FluidWidget;
    use crate::app::widgets::widgets_menu::{SettingType, SettingsMenu};
    use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
    use crate::simulation::{fluid::Fluid, renderer::Renderer};

    #[test]
    fn update_initial_configs() {
        //---------- Init renderer -----------
        let fluid_configs = FluidConfigs::default();
        let simulation_configs = SimulationConfigs::default();

        let fluid = Fluid::new(fluid_configs, simulation_configs);
        let mut renderer = Renderer::new(fluid);

        // assert it is correctly configured for the test
        assert_ne!(renderer.fluid.fluid_configs.diffusion, 0.4212312);

        //---------- Init SettingMenu -----------
        let mut fluid_ui_setting = FluidWidget::default();

        fluid_ui_setting.fluid_configs.diffusion = 0.4212312;

        // assert it is correctly configured for the test
        assert_eq!(fluid_ui_setting.fluid_configs.diffusion, 0.4212312);

        let settings_menu =
            SettingsMenu::from_settings(vec![SettingType::Fluid(fluid_ui_setting)], false);

        //---------- Test's purpouse -----------
        renderer.update_initial_configs(&settings_menu.settings_menu);

        assert_eq!(renderer.next_fluid_configs.diffusion, 0.4212312);
    }

    use crate::simulation::fluid::ContainerWall;

    #[test]
    fn default_renderer_walls_as_obstacle() {
        let renderer = Renderer::default();

        // ----- Count walls by their type --------
        let north_count = renderer
            .fluid
            .cells_type
            .iter()
            .filter(|&el| el == &ContainerWall::North)
            .count();
        let south_count = renderer
            .fluid
            .cells_type
            .iter()
            .filter(|&el| el == &ContainerWall::South)
            .count();
        let east_count = renderer
            .fluid
            .cells_type
            .iter()
            .filter(|&el| el == &ContainerWall::East)
            .count();
        let west_count = renderer
            .fluid
            .cells_type
            .iter()
            .filter(|&el| el == &ContainerWall::West)
            .count();
        let default_count = renderer
            .fluid
            .cells_type
            .iter()
            .filter(|&el| el == &ContainerWall::DefaultWall)
            .count();

        let size = i64::from(renderer.fluid.simulation_configs.size);
        let size_middle = size / 2;

        // --- Assert correct count of the wall types ------
        //
        // NB: the schema is logically (and mathematically) depicted. The actual distribution is
        // flipped by the horizontal axis.
        // | W | N | N | N | N | E |
        // | ? | ? | S | S | ? | ? |
        // | W | E | - | - | W | E |
        // | W | E | - | - | W | E |
        // | ? | ? | N | N | ? | ? |
        // | W | S | S | S | S | E |
        assert_eq!(default_count as i64, 0);
        assert_eq!(
            (north_count + south_count + east_count + west_count) as i64,
            4 * (size - 1) + 4 * (size - 2 - 1)
        );

        // ---- Assert correct order of the wall types ------
        // We do this by taking an arbitary point in the middle of the wall. Corners are undefined
        // but they also does not matter, since they are processed otherwise. The verification is
        // done for the inner wall edges since they are the one of importance.
        assert_eq!(
            renderer.fluid.cells_type[idx!(size_middle, 1, size)],
            ContainerWall::North
        );
        assert_eq!(
            renderer.fluid.cells_type[idx!(size - 2, size_middle, size)],
            ContainerWall::West
        );
        assert_eq!(
            renderer.fluid.cells_type[idx!(size_middle, size - 2, size)],
            ContainerWall::South
        );
        assert_eq!(
            renderer.fluid.cells_type[idx!(1, size_middle, size)],
            ContainerWall::East
        );
    }
}
