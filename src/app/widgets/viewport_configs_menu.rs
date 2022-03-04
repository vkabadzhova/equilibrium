use eframe::egui;

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Copy, Clone)]
pub struct ViewportUiSettings {
    enabled: bool,
    /// Configurations for the size of the simulation image as regards the size of the central panel
    /// *Note:* in percents
    pub image_resize_factor: u32,
    /// Configuration for the amount of seconds between frame change for the play button
    pub play_simulation_speed: f32,
}

impl Default for ViewportUiSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            image_resize_factor: 50,
            play_simulation_speed: 0.1,
        }
    }
}

impl super::Setting for ViewportUiSettings {
    fn name(&self) -> &'static str {
        "Viewport"
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

impl super::View for ViewportUiSettings {
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
            ui.hyperlink("https://docs.rs/egui/").on_hover_text(tooltip_text);
        });
    }
}

impl ViewportUiSettings {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled,
            image_resize_factor,
            play_simulation_speed,
        } = self;

        ui.label("Rendered image resize factor");
        //.on_hover_text("Configurations for the size of the simulation image as regards the size of the central panel.
        //*Note:* in percents");
        ui.add(egui::DragValue::new(image_resize_factor).speed(1.0));
        if *image_resize_factor > 100 {
            *image_resize_factor = 100;
        }
        ui.end_row();

        ui.label("Simulation spped on play");
        //.on_hover_text(" Configuration for the amount of seconds between frame change for the play button");
        ui.add(egui::DragValue::new(play_simulation_speed).speed(0.01));
        if *play_simulation_speed < 0.0 {
            *play_simulation_speed = 0.00;
        }
        ui.end_row();
    }
}
