use crate::renderer::Renderer;
use eframe::{egui, epi};
use image::GenericImageView;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

/// Entry-point for the fluid simulation application
pub struct App {
    label: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    current_frame: i64,

    #[cfg_attr(feature = "persistence", serde(skip))]
    renderer: Renderer,

    #[cfg_attr(feature = "persistence", serde(skip))]
    simulated: bool,
}

impl App {
    /// Creates new App instance
    pub fn new(renderer: Renderer) -> Self {
        Self {
            label: "Type frame number".to_owned(),
            current_frame: 0,
            renderer: renderer,
            simulated: false,
        }
    }

    fn show_image(&self, frame: &epi::Frame, ui: &mut egui::Ui) {
        let image_path = self.renderer.rendered_images_dir.clone()
            + "/density"
            + &self.current_frame.to_string()
            + ".jpg";

        let image = image::open(image_path).unwrap();
        let size = image.dimensions();

        let image = epi::Image::from_rgba_unmultiplied(
            [size.0.try_into().unwrap(), size.1.try_into().unwrap()],
            &image.into_rgba8().into_raw(),
        );
        let texture_id = frame.alloc_texture(image);
        let mut size = egui::Vec2::new(size.0 as f32, size.1 as f32);
        size *= (ui.available_width() / size.x).min(1.0);
        ui.image(texture_id, size);
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "eframe template"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let Self {
            label,
            current_frame,
            renderer,
            simulated,
        } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Settings");

            ui.horizontal(|ui| {
                ui.label("Frame: ");
                ui.text_edit_singleline(label);
            });

            ui.add(
                egui::Slider::new(
                    current_frame,
                    0..=self.renderer.simulation_configs.frames - 1,
                )
                .text("Current frame"),
            );
            if ui.button("Previous").clicked() {
                *current_frame = (*current_frame - 1) % self.renderer.simulation_configs.frames;
                if *current_frame < 0 {
                    *current_frame = self.renderer.simulation_configs.frames - 1;
                }
            }
            if ui.button("Next").clicked() {
                *current_frame = (*current_frame + 1) % self.renderer.simulation_configs.frames;
            }

            if ui.button("Simulate fluid").clicked() {
                self.renderer.simulate();
                *simulated = true;
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Welcome to the Equilibrium Fluid Simulator!");
            if self.simulated {
                self.show_image(frame, ui);
            }
            /*
            if *simulated {
                ui.image(
                    egui::TextureId::Egui,
                    [
                        self.renderer.fluid.simulation_configs.size as f32,
                        self.renderer.fluid.simulation_configs.size as f32,
                    ],
                );
            }
            */
            ui.hyperlink("https://github.com/vkabadzhova/equilibrium");
            ui.add(egui::github_link_file!(
                "https://github.com/vkabadzhova/equilibrium",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
