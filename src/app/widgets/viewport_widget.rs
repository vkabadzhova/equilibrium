use eframe::egui;

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
pub struct ViewportWidget {
    enabled: bool,
    /// Configurations for the size of the simulation image as regards the size of the central panel
    /// *Note:* in percents
    pub image_resize_factor: u8,
    /// The name of the directory in which the rendered images will be saved.
    pub save_into_dir: String,
}

impl Default for ViewportWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            image_resize_factor: 50,
            save_into_dir: "rendered_images".to_string(),
        }
    }
}

impl super::Setting for ViewportWidget {
    fn name(&self) -> &'static str {
        "🚪Viewport"
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

impl super::View for ViewportWidget {
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

impl ViewportWidget {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            // Field not used, ignore it. Item placed for completeness.
            enabled: _,
            image_resize_factor,
            save_into_dir,
        } = self;

        ui.label("Rendered image resize factor")
        .on_hover_text("Configurations for the size of the simulation image as regards the size of the central panel. *Note:* in percents");
        ui.add(egui::DragValue::new(image_resize_factor).speed(1.0));
        if *image_resize_factor > 100 {
            *image_resize_factor = 100;
        } else if *image_resize_factor <= 1 {
            *image_resize_factor = 1;
        }
        ui.end_row();

        ui.label("Save into directory:")
            .on_hover_text("The name of the directory in which the rendered images will be saved.");
        ui.text_edit_singleline(save_into_dir);

        ui.end_row();
    }
}
