use super::cached_image::CashedImage;
use crate::app::app::egui::ScrollArea;
use crate::app::widgets::widgets_menu::{SettingType, SettingsMenu};
use crate::simulation::renderer::density_img_path;
use crate::simulation::renderer::Renderer;
use crossbeam_utils::thread;
use eframe::egui::global_dark_light_mode_switch;
use eframe::{egui, epi};
use image::imageops::FilterType::Triangle;
use image::GenericImageView;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

/// Entry-point for the fluid simulation application
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    #[cfg_attr(feature = "persistence", serde(skip))]
    /// The "id" of the last showed image in the application of a given simulation
    /// in the context of specific simulation
    current_frame: i64,

    #[cfg_attr(feature = "persistence", serde(skip))]
    /// The fluid driver that renders its state in a file
    renderer: Renderer,

    #[cfg_attr(feature = "persistence", serde(skip))]
    /// Is a fluid simulated and ready to be showed
    is_simulated: bool,

    #[cfg_attr(feature = "persistence", serde(skip))]
    /// The progress of the simulation
    simulation_progress: f32,

    #[cfg_attr(feature = "persistence", serde(skip))]
    /// Between-threads receiver, participant in a channel open between the
    /// renderer and the application
    signal_receiver: Receiver<i64>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    /// Collection of all the widgets in the application
    settings_menu: SettingsMenu,

    /// The last showed image is chached.
    cached_image: Option<CashedImage>,

    /// The play button is on
    is_play_button_on: bool,
}

impl App {
    /// Creates new App instance
    pub fn new(renderer: Renderer) -> Self {
        let (_, signal_receiver) = mpsc::channel();

        Self {
            current_frame: 0,
            renderer,
            is_simulated: false,
            simulation_progress: 1.0,
            signal_receiver,
            settings_menu: SettingsMenu::default(),
            cached_image: None,
            is_play_button_on: false,
        }
    }

    fn get_zoom_factor(&self) -> Option<u8> {
        for i in self.settings_menu.settings_menu.iter() {
            if let SettingType::Viewport(result) = i {
                return Some(result.image_resize_factor);
            }
        }
        None
    }

    fn show_image(&mut self, image_path: &str, frame: &epi::Frame, ui: &mut egui::Ui) {
        let zoom_factor = self
            .get_zoom_factor()
            .expect("A viewport setting should exsist.");

        if self.cached_image.is_some()
            && self
                .cached_image
                .as_ref()
                .unwrap()
                .consists_of(image_path, zoom_factor)
        {
            let dimensions = self.cached_image.as_ref().unwrap().dimensions;

            ui.image(
                self.cached_image.as_ref().unwrap().rendered_texture,
                dimensions,
            );
            return;
        }

        let image = image::open(image_path)
            .expect("Couldn't open image")
            .resize(
                (ui.available_width() * (zoom_factor as f32 / 100.0)) as u32,
                (ui.available_height() * (zoom_factor as f32 / 100.0)) as u32,
                Triangle,
            );

        let size = image.dimensions();

        let image = epi::Image::from_rgba_unmultiplied(
            [size.0.try_into().unwrap(), size.1.try_into().unwrap()],
            &image.into_rgba8().into_raw(),
        );

        let texture_id = frame.alloc_texture(image);
        let mut size = egui::Vec2::new(size.0 as f32, size.1 as f32);
        size *= (ui.available_width() / size.x).min(1.0);

        ui.image(texture_id, size);
        self.cached_image = Some(CashedImage {
            path: image_path.to_string(),
            zoom_factor,
            dimensions: size,
            rendered_texture: texture_id,
        });
    }

    fn render(renderer: &mut Renderer) -> Receiver<i64> {
        let (tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();

        thread::scope(move |s| {
            s.spawn(|_| {
                renderer.simulate(tx);
            });
        })
        .unwrap();
        rx
    }

    fn move_simulation_frame(&mut self, next_frame: i64, frame: &epi::Frame, ui: &mut egui::Ui) {
        self.simulation_progress =
            next_frame as f32 / (self.renderer.fluid.simulation_configs.frames - 1) as f32;

        self.show_image(
            density_img_path!(&self.renderer.rendered_images_dir, next_frame),
            frame,
            ui,
        );
        self.current_frame = next_frame;

        frame.request_repaint();
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "The Equilibrium Fluid Simulator"
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
        self.renderer
            .update_configs(&self.settings_menu.settings_menu);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            Self::bar_content(ui, frame);
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            self.left_panel(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui, frame);
        });

        egui::SidePanel::right("right_panel")
            .min_width(180.0)
            .default_width(180.0)
            .show(ctx, |ui| self.right_panel(ui));

        self.settings_menu.windows(ctx);

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
            });
        }
    }
}

impl App {
    /// The GUI organization for the bar on the top of the application.
    fn bar_content(ui: &mut egui::Ui, frame: &epi::Frame) {
        ui.horizontal_wrapped(|ui| {
            global_dark_light_mode_switch(ui);

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        frame.quit();
                    }
                });
            });
        });
    }

    /// The GUI organization for the left panel with the navigation through the simulation.
    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Navigate simulation");

        ui.add(
            egui::Slider::new(
                &mut self.current_frame,
                0..=self.renderer.fluid.simulation_configs.frames - 1,
            )
            .text("Current frame"),
        );

        ui.horizontal_wrapped(|ui| {
            if ui.button("Previous").clicked() {
                self.current_frame =
                    (self.current_frame - 1) % self.renderer.fluid.simulation_configs.frames;
                if self.current_frame < 0 {
                    self.current_frame = self.renderer.fluid.simulation_configs.frames - 1;
                }
            }

            if ui.button("Next").clicked() {
                self.current_frame =
                    (self.current_frame + 1) % self.renderer.fluid.simulation_configs.frames;
            }
        });

        if ui.button("Play simulation").clicked() {
            self.is_play_button_on = true;

            if self.is_simulated {
                self.current_frame = 0;
            }
        }

        ui.separator();

        if ui.button("Simulate fluid").clicked() {
            self.is_simulated = true;
            self.simulation_progress = 0.0;
            self.current_frame = 0;
            self.signal_receiver = App::render(&mut self.renderer);
        }

        ui.label("Simulation Progress:");
        let progress_bar = egui::ProgressBar::new(self.simulation_progress).show_percentage();
        ui.add(progress_bar);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
            });
        });
    }

    /// The GUI organization for the central panel with the rendered images display.
    fn central_panel(&mut self, ui: &mut egui::Ui, frame: &epi::Frame) {
        ui.heading("Welcome to the Equilibrium Fluid Simulator!");

        let frames_count = self.renderer.fluid.simulation_configs.frames;

        if self.current_frame < frames_count - 1 && !self.is_play_button_on && self.is_simulated {
            self.current_frame = self.signal_receiver.try_recv().unwrap();
        }

        if self.is_simulated {
            self.move_simulation_frame(self.current_frame, frame, ui);

            if self.is_play_button_on {
                self.current_frame += 1;
            }

            if self.current_frame == frames_count - 1 {
                self.is_play_button_on = false;
            }
        }

        ui.hyperlink("https://github.com/vkabadzhova/equilibrium");
        ui.add(egui::github_link_file!(
            "https://github.com/vkabadzhova/equilibrium",
            "Source code."
        ));
    }

    /// The GUI organization for the right panel with checkbox menus.
    fn right_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("✒ Settings");
        });

        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            use egui::special_emojis::{GITHUB, OS_APPLE, OS_LINUX, OS_WINDOWS};

            ui.vertical_centered(|ui| {
                ui.label(
                    "Welcome to the equilibrium fluid simulator - my high school diploma thesis.",
                );

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

            ui.separator();
            ui.vertical_centered(|ui| {
                if ui.button("Back to default values").clicked() {
                    for el in self.settings_menu.settings_menu.iter_mut() {
                        el.to_default();
                    }
                }

                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory().reset_areas();
                }

                if ui.button("Close all windows").clicked() {
                    self.settings_menu.close_all();
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::renderer::density_img_path;

    #[test]
    fn density_img_path_str_works() {
        assert_eq!(
            density_img_path!("holiday_dir", 0),
            "holiday_dir/density0.jpg"
        );
    }
}
