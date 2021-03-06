use crate::simulation::configs::SimulationConfigs;
use eframe::egui;

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Copy, Clone)]
pub struct SimulationWidget {
    enabled: bool,
    /// The configurations for the simulation
    pub simulation_configs: SimulationConfigs,
}

impl Default for SimulationWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            simulation_configs: SimulationConfigs::default(),
        }
    }
}

impl super::Setting for SimulationWidget {
    fn name(&self) -> &'static str {
        "🔨 Simulation"
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

impl super::View for SimulationWidget {
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

impl SimulationWidget {
    /// Sets up the content of the widget.
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            // Field not used, ignore it. Item placed for completeness.
            enabled: _,
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
        if simulation_configs.size < 1 {
            simulation_configs.size = 1;
        }
        ui.end_row();
    }
}
