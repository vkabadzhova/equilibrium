use crate::simulation::configs::FluidConfigs;
use eframe::egui;

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Copy, Clone)]
pub struct FluidWidget {
    enabled: bool,
    /// The fluid configurations
    pub fluid_configs: FluidConfigs,
}

impl Default for FluidWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            fluid_configs: FluidConfigs::default(),
        }
    }
}

impl super::Setting for FluidWidget {
    fn name(&self) -> &'static str {
        "🌊 Fluid"
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

impl super::View for FluidWidget {
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

        ui.separator();

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.enabled, "Interactive").on_hover_text(
                "Uncheck to disable the widget so you could inspect the simulation securely.",
            );
        });

        ui.separator();

        ui.vertical_centered(|ui| {
            let tooltip_text = "The full documentation on the simulation parameters can be found by typing `cargo d --open`";
            ui.label(tooltip_text);
        });
    }
}

impl FluidWidget {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled: _,
            fluid_configs,
        } = self;

        ui.label("Setting panel descritpion");
        ui.add(
            egui::Label::new("Here you can find settings related to the fluid itself.").wrap(true),
        );
        ui.end_row();

        ui.label("Add perlin noise to simulation");
        ui.checkbox(&mut fluid_configs.has_perlin_noise, "");
        ui.end_row();

        ui.label("Choose fluid color");
        ui.color_edit_button_srgba(&mut fluid_configs.fluid_color);
        ui.end_row();

        ui.label("Choose world color");
        ui.color_edit_button_srgba(&mut fluid_configs.world_color);
        ui.end_row();

        ui.hyperlink_to("Viscousity", "https://en.wikipedia.org/wiki/Viscosity");
        ui.add(egui::Slider::new(&mut fluid_configs.viscousity, 0.0..=1.0).suffix(" m2/s"));
        ui.end_row();

        ui.hyperlink_to("Diffusion", "https://en.wikipedia.org/wiki/Diffusion");
        ui.add(egui::Slider::new(&mut fluid_configs.diffusion, 0.0..=1.0));
        ui.end_row();
    }
}
