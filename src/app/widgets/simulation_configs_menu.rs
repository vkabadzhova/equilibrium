use eframe::egui;
use crate::simulation::configs::{SimulationConfigs};

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SimulationUiSettings {
    enabled: bool,
    /// The configurations for the simulation
    pub simulation_configs: SimulationConfigs,
}

impl Default for SimulationUiSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            simulation_configs: SimulationConfigs::default()
        }
    }
}

impl super::Setting for SimulationUiSettings {
    fn name(&self) -> &'static str {
        "🗄Simulation settings"
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

impl super::View for SimulationUiSettings {
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
                ui.checkbox(&mut self.enabled, "Interactive")
                    .on_hover_text("Uncheck to disable the widget so you could inspect the simulation securely.");
        });

        ui.separator();

        ui.vertical_centered(|ui| {
            let tooltip_text = "The full documentation on the simulation parameters can be found by typing `cargo d --open`";
            ui.label(tooltip_text);
            ui.hyperlink("https://docs.rs/egui/").on_hover_text(tooltip_text);
        });
    }
}

impl SimulationUiSettings {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled,
            simulation_configs,
        } = self;

        ui.label("Number of frames");
        ui.add(egui::DragValue::new(&mut simulation_configs.frames).speed(1.0));
        if simulation_configs.frames < 1 {
            simulation_configs.frames = 1;
        }
        ui.end_row();

        ui.label("Simulation step (delta_t)")
                .on_hover_text("delta_t");
        ui.add(egui::DragValue::new(&mut simulation_configs.delta_t).speed(0.01));
        if simulation_configs.delta_t < 0.0 {
            simulation_configs.delta_t = 0.00;
        }
        ui.end_row();

        ui.label("Simulation window size");
        ui.add(egui::DragValue::new(&mut simulation_configs.size).speed(1.0));
        if simulation_configs.size < 128 {
            simulation_configs.size = 128 ;
        }
        ui.end_row();
    }
}
