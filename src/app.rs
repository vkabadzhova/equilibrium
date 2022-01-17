use crate::app::egui::ScrollArea;
use crate::app_widgets::widgets_menu::SettingsMenu;
use crate::renderer::Renderer;
use crossbeam_utils::thread;
use eframe::{egui, epi};
use image::GenericImageView;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

/// Entry-point for the fluid simulation application
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    next_frames_count: String,

    #[cfg_attr(feature = "persistence", serde(skip))]
    current_frame: i64,

    #[cfg_attr(feature = "persistence", serde(skip))]
    renderer: Renderer,

    #[cfg_attr(feature = "persistence", serde(skip))]
    is_simulated: bool,

    #[cfg_attr(feature = "persistence", serde(skip))]
    simulation_progress: f32,

    #[cfg_attr(feature = "persistence", serde(skip))]
    signal_receiver: Receiver<i64>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    settings_menu: SettingsMenu,
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
            next_frames_count: "Type frame number".to_owned(),
            current_frame: 0,
            renderer: renderer,
            is_simulated: false,
            simulation_progress: 1.0,
            signal_receiver: signal_receiver,
            settings_menu: SettingsMenu::default(),
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

        let total_number_of_frames = renderer.fluid.simulation_configs.frames;
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
        let mut current_frame = self.signal_receiver.try_recv().unwrap();
        self.simulation_progress += 1.0 / self.renderer.fluid.simulation_configs.frames as f32;
        if current_frame > self.current_frame {
            App::show_image(
                density_img_path!(&self.renderer.rendered_images_dir, current_frame),
                frame,
                ui,
            );
            self.current_frame = current_frame;
        }

        if current_frame == self.renderer.fluid.simulation_configs.frames - 1 {
            self.simulation_progress = 1.0;
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
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let Self {
            next_frames_count,
            current_frame,
            renderer,
            is_simulated,
            simulation_progress,
            signal_receiver,
            settings_menu,
        } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Simulation settings");

            ui.horizontal(|ui| {
                ui.label("Number of frames: ");

                ui.text_edit_singleline(next_frames_count)
                    .on_hover_text("Should be a whole number");

                self.renderer.next_simulation_configs.frames =
                    match next_frames_count.parse::<i64>() {
                        Ok(val) => val,
                        Err(_) => self.renderer.fluid.simulation_configs.frames,
                    }
            });

            if ui.button("Simulate fluid").clicked() {
                *is_simulated = true;
                *simulation_progress = 0.0;
                *current_frame = 0;
                self.signal_receiver = App::render(&mut self.renderer, frame, ui);
            }

            ui.label("Simulation Done:");
            let progress_bar = egui::ProgressBar::new(*simulation_progress).show_percentage();
            ui.add(progress_bar);

            ui.separator();
            ui.heading("Fluid settings");

            ui.separator();
            ui.heading("Simulation navigation");

            ui.add(
                egui::Slider::new(
                    current_frame,
                    0..=self.renderer.fluid.simulation_configs.frames - 1,
                )
                .text("Current frame"),
            );
            if ui.button("Previous").clicked() {
                *current_frame =
                    (*current_frame - 1) % self.renderer.fluid.simulation_configs.frames;
                if *current_frame < 0 {
                    *current_frame = self.renderer.fluid.simulation_configs.frames - 1;
                }
            }
            if ui.button("Next").clicked() {
                *current_frame =
                    (*current_frame + 1) % self.renderer.fluid.simulation_configs.frames;
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

            if self.current_frame < self.renderer.fluid.simulation_configs.frames - 1
                && self.simulation_progress != 1.0
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

        egui::SidePanel::right("egui_demo_panel")
            .min_width(150.0)
            .default_width(180.0)
            .show(ctx, |ui| {
                egui::trace!(ui);
                ui.vertical_centered(|ui| {
                    ui.heading("✒ Settings");
                });

                ui.separator();

                ScrollArea::vertical().show(ui, |ui| {
                    use egui::special_emojis::{GITHUB, OS_APPLE, OS_LINUX, OS_WINDOWS};

                    ui.vertical_centered(|ui| {
                        ui.label("Welcome to the equilibrium fluid simulator - my high school diploma thesis.");

                        ui.label(format!(
                            "You run it on the web, or natively on {}{}{}",
                            OS_APPLE, OS_LINUX, OS_WINDOWS,
                        ));

                        ui.hyperlink_to(
                            format!("{} equilibrium home page", GITHUB),
                            "https://github.com/vkabadzhova/equilibrium",
                        );
                    });

                    ui.separator();
                    self.settings_menu.checkboxes(ui);
                    // add checkboxes
                    ui.separator();
                    // add checkboxes
                    ui.separator();

                    ui.vertical_centered(|ui| {
                        if ui.button("Organize windows").clicked() {
                            ui.ctx().memory().reset_areas();
                        }
                    });
                });
            });

        self.settings_menu.windows(ctx);

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
