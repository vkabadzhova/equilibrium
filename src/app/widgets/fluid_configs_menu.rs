use eframe::egui;
use crate::simulation::configs::{FluidConfigs};

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FluidUiSettings {
    enabled: bool,
    /// Setting for [`Fluid::add_noise()`][crate::simulation::fluid::Fluid::add_noise()]
    pub add_random_noise: bool,
    // TODO: add colors to the fluidConfig in the renderer
    /// Modify fluid's color in simulation
    pub fluid_color: egui::Color32,
    /// Modify world's simulation color
    pub world_color: egui::Color32,
    /// The fluid configurations
    pub fluid_configs: FluidConfigs,
}

impl Default for FluidUiSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            add_random_noise: false,
            fluid_color: egui::Color32::LIGHT_BLUE.linear_multiply(0.5),
            world_color: egui::Color32::LIGHT_BLUE.linear_multiply(0.5),
            fluid_configs: FluidConfigs {
                diffusion: 0.42,
                viscousity: 0.3
            }
        }
    }
}

impl super::Setting for FluidUiSettings {
    fn name(&self) -> &'static str {
        "🗄Fluid settings"
    }

    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                use super::View as _;
                self.ui(ui);
            });
    }
}

impl super::View for FluidUiSettings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.enabled, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    self.gallery_grid_contents(ui);
                });
        });
    }
}

impl FluidUiSettings {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled: _,
            add_random_noise,
            fluid_color,
            world_color,
            fluid_configs,
        } = self;

        ui.label("Setting panel descritpion");
        ui.add(
            egui::Label::new("Here you can find settings related to the fluid itself.").wrap(true),
        );
        ui.end_row();

        ui.label("Add perlin noise to simulation");
        ui.checkbox(add_random_noise, "");
        ui.end_row();

        ui.label("Choose fluid color");
        ui.color_edit_button_srgba(fluid_color);
        ui.end_row();

        ui.label("Choose world color");
        ui.color_edit_button_srgba(world_color);
        ui.end_row();

        ui.hyperlink_to("Viscousity", "https://en.wikipedia.org/wiki/Viscosity");
        ui.add(egui::Slider::new(&mut fluid_configs.viscousity, 0.0..=1.0).suffix(" m2/s"));
        ui.end_row();

        ui.hyperlink_to("Diffusion", "https://en.wikipedia.org/wiki/Diffusion");
        ui.add(egui::Slider::new(&mut fluid_configs.diffusion, 0.0..=1.0));
        ui.end_row();
    }
}
