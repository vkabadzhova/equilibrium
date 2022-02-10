use crate::app::widgets::widgets_menu::SettingType;
use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::fluid::Fluid;
use std::fs;
use std::sync::mpsc::Sender;

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
}

impl Renderer {
    fn make_rendered_images_dir() -> String {
        let project_root = project_root::get_project_root()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        project_root + "/rendered_images"
    }

    /// Creates new Renderer
    pub fn new(fluid: Fluid) -> Renderer {
        Renderer {
            next_fluid_configs: fluid.fluid_configs.clone(),
            next_simulation_configs: fluid.simulation_configs.clone(),
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
            if density != 0.0 {
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

        let img_name = format!("{}/density{}.jpg", &self.rendered_images_dir, frame_number);
        imgbuf.save(img_name).unwrap();
    }

    /// Runs the fluid simulation
    pub fn simulate(&mut self, tx: Sender<i64>) {
        self.fluid = Fluid::new(self.next_fluid_configs, self.next_simulation_configs);
        self.fluid.init();
        for i in 0..self.fluid.simulation_configs.frames {
            if self.fluid.fluid_configs.has_perlin_noise {
                self.fluid.add_noise();
            }
            self.fluid.step();

            self.render_density(i);
            tx.send(i).unwrap();
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::widgets::fluid_configs_menu::FluidUiSettings;
    use crate::app::widgets::simulation_configs_menu::SimulationUiSettings;
    use crate::app::widgets::widgets_menu::{SettingsMenu, SettingType};
    use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
    use crate::simulation::{fluid::Fluid, renderer::Renderer};
    use eframe::egui::Color32;

    #[test]
    fn update_initial_configs_works() {
        //---------- Init renderer -----------
        let fluid_configs = FluidConfigs::default();
        let simulation_configs = SimulationConfigs::default();

        let fluid = Fluid::new(fluid_configs, simulation_configs);
        let mut renderer = Renderer::new(fluid);

        // assert it is correctly configured for the test
        assert_ne!(renderer.fluid.fluid_configs.diffusion, 0.4212312);

        //---------- Init SettingMenu -----------
        let simulation_ui_setting = SimulationUiSettings::default();
        let mut fluid_ui_setting = FluidUiSettings::default();

        fluid_ui_setting.fluid_configs.diffusion = 0.4212312;

        // assert it is correctly configured for the test
        assert_eq!(fluid_ui_setting.fluid_configs.diffusion, 0.4212312);

        let settings_menu = SettingsMenu::from_settings(vec![
            SettingType::Fluid(fluid_ui_setting),
        ], false);

        //---------- Test's purpouse -----------
        renderer.update_initial_configs(&settings_menu.settings_menu);

        assert_eq!(renderer.next_fluid_configs.diffusion, 0.4212312);
    }
}
