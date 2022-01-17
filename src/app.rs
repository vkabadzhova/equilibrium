use crate::renderer::Renderer;
use crossbeam_utils::thread;
use eframe::{egui, epi};
use egui::Context;
use image::GenericImageView;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

// We derive Deserialize/Serialize so we can persist app state on shutdown.
/// Entry-point for the fluid simulation application
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    label: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    current_frame: i64,

    #[cfg_attr(feature = "persistence", serde(skip))]
    renderer: Renderer,

    #[cfg_attr(feature = "persistence", serde(skip))]
    is_simulated: bool,

    #[cfg_attr(feature = "persistence", serde(skip))]
    is_in_simulation_process: bool,

    #[cfg_attr(feature = "persistence", serde(skip))]
    signal_receiver: Receiver<i64>,
}

macro_rules! density_img_path {
    ($rendered_images_dir:expr, $frame_number:expr) => {
        &($rendered_images_dir.clone().to_owned()
            + "/density"
            + &$frame_number.to_string()
            + ".jpg")
    };
}

impl App {
    /// Creates new App instance
    pub fn new(renderer: Renderer) -> Self {
        let (_, signal_receiver) = mpsc::channel();

        Self {
            label: "Type frame number".to_owned(),
            current_frame: 0,
            renderer: renderer,
            is_simulated: false,
            is_in_simulation_process: false,
            signal_receiver: signal_receiver,
        }
    }

    fn show_image(image_path: &str, frame: &epi::Frame, ui: &mut egui::Ui) {
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

    fn render(renderer: &mut Renderer, frame: &epi::Frame, ui: &mut egui::Ui) -> Receiver<i64> {
        let (tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();

        let total_number_of_frames = renderer.simulation_configs.frames;
        let rendered_images_dir = renderer.rendered_images_dir.clone();

        thread::scope(move |s| {
            s.spawn(|_| {
                renderer.simulate(tx);
            });
        })
        .unwrap();
        rx
    }

    fn render_next_received_img(&mut self, frame: &epi::Frame, ui: &mut egui::Ui) {
        println!(
            "self.current_frame[{}] < self.renderer.simulation_configs.frames - 1
                && self.is_in_simulation_process",
            self.current_frame
        );

        let mut current_frame = self.signal_receiver.try_recv().unwrap();
        if current_frame > self.current_frame {
            App::show_image(
                density_img_path!(&self.renderer.rendered_images_dir, current_frame),
                frame,
                ui,
            );
            self.current_frame = current_frame;
        }

        if current_frame == self.renderer.simulation_configs.frames - 1 {
            self.is_in_simulation_process = false;
        }

        frame.request_repaint();
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
            is_simulated,
            is_in_simulation_process,
            signal_receiver,
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
                *is_simulated = true;
                *is_in_simulation_process = true;
                *current_frame = 0;
                self.signal_receiver = App::render(&mut self.renderer, frame, ui);
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
            ui.heading("Welcome to the Equilibrium Fluid Simulator!");

            if self.current_frame < self.renderer.simulation_configs.frames - 1
                && self.is_in_simulation_process
            {
                self.render_next_received_img(frame, ui);
            } else if self.is_simulated {
                App::show_image(
                    density_img_path!(&self.renderer.rendered_images_dir, self.current_frame),
                    frame,
                    ui,
                );
            }

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
