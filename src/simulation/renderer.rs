use super::fluid::ContainerWall;
use super::obstacle::ObstaclesType;
use super::renderer_helpers::CurrentSimulation;
use crate::app::widgets::widgets_menu::SettingType;
use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::fluid::Fluid;
use eframe::egui::Color32;
use std::fs;
use std::sync::mpsc::Sender;

/// Creates a name of the a rendered density file based on the frame number and
/// a given directory
macro_rules! density_img_path {
    ($save_into_dir:expr, $frame_number:expr) => {
        &($save_into_dir.clone().to_owned() + "/density" + &$frame_number.to_string() + ".jpg")
    };
}

pub(crate) use density_img_path;

/// Utility for visualization and interaction with the fluid simulation.
///
/// There are two ways to modify the simulation’s parameters while the simulation is
/// going: to rerun the simulation from the beginning, or to continue it with
/// the newly added parameters. Internally, there are parameters of the type `next_*_configs`
/// that are used. In the first case, they are updating the current state of the
/// renderer live during the simulation (the current state is stored in the
/// struct’s member [`fluid`](crate::simulation::renderer::Renderer::fluid)), or,
/// otherwise, update it at the **beginning** of the new simulation (i.e.
/// frame 0). The latter is achieved by bufferring the future state of the configurations.
/// In order to update the next configurations, use [`Renderer::update_configs()`].
pub struct Renderer {
    /// The Renderer owns the fluid that it simulates
    pub fluid: Fluid,

    /// Buffered fluid configurations for the next run. The configurations of the fluid are not
    /// changed while the fluid is being simulated.
    next_fluid_configs: FluidConfigs,

    /// Buffered simulation configurations for the next run. The configurations of the fluid are
    /// not changed while the fluid is being simulated.
    next_simulation_configs: SimulationConfigs,

    /// The directory in which the rendered images that result from the simulation are stored.
    pub save_into_dir: String,

    /// Buffered name for saving directory for the next run. Use when you don't want to change the
    /// directory in the middle of the previous simulation. Use with [`Renderer::update_configs()`]
    next_save_into_dir: String,

    /// The color of all obstacles. Obstacles cannot be set individual colors.
    pub obstacles_color: eframe::egui::Color32,

    /// Collection of all the obstacles. To update the fluid's behaviour to correspond to the
    /// obstacles, use [`update_obstacles`].
    pub obstacles: Vec<ObstaclesType>,

    /// Buffered obstacles for the next run. The configurations of the fluid are not changed while
    /// the fluid is being simulated.
    next_obstacles: Vec<ObstaclesType>,

    /// Contains the state of the current simulation step.
    pub current_simulation: CurrentSimulation,

    /// Listens for the current simulation signal, in order to render the simulation into an image
    pub rendering_listener: RenderingListener,
}

// Safety: No one besides us owns obstacle, so we can safely transfer
// it to another thread
unsafe impl Send for Renderer {}

impl Default for Renderer {
    /// Add default Renderer, containing an obstacle and default [`FluidConfigs`] and [`SimulationConfigs`]
    fn default() -> Self {
        let fluid = Fluid::default();
        let default_dir = Renderer::make_save_into_dir("rendered_images".to_string());

        let mut result = Self {
            next_fluid_configs: fluid.fluid_configs.clone(),
            next_simulation_configs: fluid.simulation_configs.clone(),
            obstacles_color: Color32::RED,
            obstacles: vec![ObstaclesType::Rectangle(
                crate::simulation::obstacle::Rectangle::default(),
            )],
            next_obstacles: vec![ObstaclesType::Rectangle(
                crate::simulation::obstacle::Rectangle::default(),
            )],
            fluid,
            save_into_dir: default_dir.clone(),
            next_save_into_dir: default_dir.clone(),
        };

        result.mark_fluid_obstacles();
        result
    }
}

impl Renderer {
    fn make_save_into_dir(dir_name: String) -> String {
        let project_root = project_root::get_project_root()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        project_root + "/" + &dir_name
    }

    /// Creates new Renderer
    pub fn new(fluid: Fluid, obstacles_color: Color32, images_dir: String) -> Renderer {
        let save_into_dir = Renderer::make_save_into_dir(images_dir);
        Renderer {
            next_fluid_configs: fluid.fluid_configs.clone(),
            next_simulation_configs: fluid.simulation_configs.clone(),
            obstacles_color,
            obstacles: Vec::new(),
            next_obstacles: Vec::new(),
            fluid,
            save_into_dir: save_into_dir.clone(),
            next_save_into_dir: save_into_dir,
            current_simulation: CurrentSimulation::default(),
            rendering_listener: RenderingListener::default(),
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

        let obstacles_rgba = [
            self.obstacles_color.r(),
            self.obstacles_color.g(),
            self.obstacles_color.b(),
            self.obstacles_color.a(),
        ];

        let mut imgbuf = image::ImageBuffer::new(
            self.fluid.simulation_configs.size,
            self.fluid.simulation_configs.size,
        );

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = self.fluid.density[idx!(x, y, self.fluid.simulation_configs.size)];
            let cell_type = self.fluid.cells_type[idx!(x, y, self.fluid.simulation_configs.size)];
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
            .save(density_img_path!(self.save_into_dir, frame_number))
            .expect("Coulnt't save density image");
    }

    /// Runs the fluid simulation
    pub fn simulate(&mut self, tx: Sender<i64>) {
        self.fluid = Fluid::new(self.next_fluid_configs, self.next_simulation_configs);
        self.obstacles = self.next_obstacles.clone();
        self.save_into_dir = self.next_save_into_dir.clone();
        self.mark_fluid_obstacles();
        for i in 0..self.fluid.simulation_configs.frames {
            if self.fluid.fluid_configs.has_perlin_noise {
                self.fluid.add_noise();
            }
            self.fluid.step();

            self.render_density(i);
            tx.send(i).unwrap();
        }
    }

    /// After altering the obstacles list. Refresh the fluid's configuration regarding its
    /// obstacles.
    pub fn mark_fluid_obstacles(&mut self) {
        for obstacle in self.obstacles.iter_mut() {
            self.fluid.fill_obstacle(obstacle);
        }
    }

    /// Updates the configurations for the next simulation. As an example,
    /// during rendering, the configurations new configurations may be saved in
    /// the renderer, but will only be applied after the beginning of the next simulation
    pub fn update_configs(&mut self, settings_menu: &[SettingType]) {
        for setting in settings_menu.iter() {
            match setting {
                SettingType::Fluid(fluid_widget) => {
                    self.next_fluid_configs = fluid_widget.fluid_configs;
                }
                SettingType::Simulation(simulation_widget) => {
                    self.next_simulation_configs = simulation_widget.simulation_configs;
                }
                SettingType::Obstacle(obstacle_widget) => {
                    self.next_obstacles = obstacle_widget
                        .obstacles
                        .iter()
                        .cloned()
                        .map(|el| el.obstacle)
                        .collect();
                    self.obstacles_color = obstacle_widget.color;
                }
                SettingType::Viewport(viewport_widget) => {
                    self.next_save_into_dir = viewport_widget.save_into_dir.clone();
                }
            }
        }
    }

    /// Runs the simulation, and then renders the result. Internally, it fires several more
    /// threads: one to simulate the fluid, and another one to render the result.
    /// As a result a [`std::sync::Receiver<i64>`] is returned, by which a signal for every new
    /// render will be sent over.
    pub fn render(&self) {
        let (simulation_tx, simulation_rx): (Sender<FluidStep>, Receiver<FluidStep>) =
            mpsc::channel();

        // TODO: prep current_simulation with correct values

        let current_simulation = renderer.current_simulation.clone();
        let simulation_handler = std::thread::spawn(move || {
            current_simulation.simulate(simulation_tx);
        });

        let rendering_listener = renderer.rendering_listener.clone();
        let max_frames = renderer.fluid.simulation_configs.frames;
        let rendering_handler = std::thread::spawn(move || {
            rendering_listener.listen(max_frames, simulation_rx);
        });

        (rx, simulation_handler)
    }
}

#[cfg(test)]
mod tests {
    use crate::app::widgets::fluid_widget::FluidWidget;
    use crate::app::widgets::widgets_menu::{SettingType, SettingsMenu};
    use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
    use crate::simulation::{fluid::Fluid, obstacle::Obstacle, renderer::Renderer};
    use eframe::egui::Color32;

    #[test]
    fn update_configs() {
        //---------- Init renderer -----------
        let fluid_configs = FluidConfigs::default();
        let simulation_configs = SimulationConfigs::default();

        let fluid = Fluid::new(fluid_configs, simulation_configs);
        let mut renderer = Renderer::new(fluid, Color32::RED, "rendered_images".to_string());

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
        renderer.update_configs(&settings_menu.settings_menu);

        assert_eq!(renderer.next_fluid_configs.diffusion, 0.4212312);
    }

    use crate::simulation::fluid::ContainerWall;

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
    fn default_renderer() {
        let renderer = Renderer::default();
        let default_walls_real_count = renderer
            .fluid
            .cells_type
            .iter()
            .filter(|&el| el == &ContainerWall::DefaultWall)
            .count();

        let size = renderer.fluid.simulation_configs.size;
        let image_parameter = 2 * (size + (size - 2));

        // Sums up the area of all obstacles. NB: Assumes all obstacles are rectangles
        let obstacles_area = renderer.obstacles.iter().fold(0, |acc, x| {
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
