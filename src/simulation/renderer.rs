use super::obstacle::ObstaclesType;
use super::renderer_helpers::{CurrentSimulation, FluidStep, RenderingListener};
use crate::app::widgets::widgets_menu::SettingType;
use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::fluid::Fluid;
use eframe::egui::Color32;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

/// Utility for visualization and interaction with the fluid simulation.
///
/// In future implementations it is planned to have two ways to modify the simulation’s parameters
/// while the simulation is running: to rerun the simulation from the beginning, or to continue it
/// with the newly added parameters. Currently the configurations that are changed, will be applied
/// in the next run of the simulation. Internally, there are parameters of the type `next_*_configs`
/// that are used to buffer the future configurations. In the first case, they will be updating the
/// current state of the renderer live during the simulation (the current state is stored in the
/// struct’s member [`Renderer::current_simulation`]), or, in the other, they will update it at the
/// **beginning** of the new simulation (i.e. frame 0). In order to update the next configurations,
/// use [`Renderer::update_configs()`].
pub struct Renderer {
    /// Buffered fluid configurations for the next run. The configurations of the fluid are not
    /// changed while the fluid is being simulated.
    next_fluid_configs: FluidConfigs,

    /// Buffered simulation configurations for the next run. The configurations of the fluid are
    /// not changed while the fluid is being simulated.
    next_simulation_configs: SimulationConfigs,

    /// Buffered name for saving directory for the next run. Use when you don't want to change the
    /// directory in the middle of the previous simulation. Use with [`Renderer::update_configs()`]
    next_save_into_dir: String,

    /// Buffered color for the obstacles. The configurations of the fluid are not changed while
    /// the simulation is running.
    next_obstacles_color: eframe::egui::Color32,

    /// Buffered obstacles for the next run. The configurations of the fluid are not changed while
    /// the fluid is being simulated.
    next_obstacles: Vec<ObstaclesType>,

    /// Contains the state of the current simulation step.
    pub current_simulation: CurrentSimulation,

    /// Listens for the current simulation signal, in order to render the simulation into an image
    pub rendering_listener: RenderingListener,
}

impl Default for Renderer {
    /// Add default Renderer, containing an obstacle and default [`FluidConfigs`] and [`SimulationConfigs`]
    fn default() -> Self {
        let fluid = Fluid::default();
        let default_dir = RenderingListener::make_save_into_dir("rendered_images");

        Self {
            next_fluid_configs: fluid.fluid_configs.clone(),
            next_simulation_configs: fluid.simulation_configs.clone(),
            next_obstacles_color: Color32::RED,
            next_obstacles: vec![ObstaclesType::Rectangle(
                crate::simulation::obstacle::Rectangle::default(),
            )],
            next_save_into_dir: default_dir.clone(),
            current_simulation: CurrentSimulation::default(),
            rendering_listener: RenderingListener::default(),
        }
    }
}

impl Renderer {
    /// Creates new Renderer
    pub fn new(fluid: Fluid, obstacles_color: Color32, images_dir: String) -> Renderer {
        let save_into_dir = RenderingListener::make_save_into_dir(&images_dir);
        Renderer {
            next_fluid_configs: fluid.fluid_configs.clone(),
            next_simulation_configs: fluid.simulation_configs.clone(),
            next_obstacles_color: obstacles_color,
            next_obstacles: Vec::new(),
            next_save_into_dir: save_into_dir,
            current_simulation: CurrentSimulation::default(),
            rendering_listener: RenderingListener::default(),
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
                    self.next_obstacles_color = obstacle_widget.color;
                }
                SettingType::Viewport(viewport_widget) => {
                    self.next_save_into_dir = RenderingListener::make_save_into_dir(
                        &viewport_widget.save_into_dir.clone(),
                    );
                }
            }
        }
    }

    /// Runs the simulation, and then renders the result. Internally, it fires several more
    /// threads: one to simulate the fluid, and another one to render the result.
    /// As a result a [`std::sync::mpsc::Receiver<i64>`] is returned, by which a signal for every new
    /// render will be sent over.
    pub fn render(&mut self) -> (Receiver<i64>, Vec<JoinHandle<()>>) {
        let (simulation_tx, simulation_rx): (Sender<FluidStep>, Receiver<FluidStep>) =
            mpsc::channel();

        self.prepare_simulation();

        let mut current_simulation = self.current_simulation.clone();
        let simulation_handler = std::thread::spawn(move || {
            current_simulation.simulate(simulation_tx);
        });

        let rendering_listener = self.rendering_listener.clone();
        let max_frames = self.current_simulation.fluid.simulation_configs.frames;

        let (rendering_tx, rendering_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();

        let rendering_handler = std::thread::spawn(move || {
            rendering_listener.listen(max_frames, simulation_rx, rendering_tx);
        });

        let result_joinhandles = vec![simulation_handler, rendering_handler];

        (rendering_rx, result_joinhandles)
    }

    /// Prepares the next simulation by creating new instances of all the needed components
    fn prepare_simulation(&mut self) {
        self.current_simulation = CurrentSimulation {
            fluid: Fluid::new(self.next_fluid_configs, self.next_simulation_configs),
            obstacles: self.next_obstacles.clone(),
        };

        self.rendering_listener = RenderingListener {
            save_into_dir: self.next_save_into_dir.clone(),
            obstacles_color: self.next_obstacles_color,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::app::widgets::fluid_widget::FluidWidget;
    use crate::app::widgets::widgets_menu::{SettingType, SettingsMenu};
    use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
    use crate::simulation::{fluid::Fluid, renderer::Renderer};
    use eframe::egui::Color32;

    #[test]
    fn update_configs() {
        //---------- Init renderer -----------
        let fluid_configs = FluidConfigs::default();
        let simulation_configs = SimulationConfigs::default();

        let fluid = Fluid::new(fluid_configs, simulation_configs);
        let mut renderer = Renderer::new(fluid, Color32::RED, "rendered_images".to_string());

        // assert it is correctly configured for the test
        assert_ne!(
            renderer.current_simulation.fluid.fluid_configs.diffusion,
            0.4212312
        );

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
}
