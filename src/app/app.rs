use super::cached_image::CachedImage;
use crate::app::app::egui::ScrollArea;
use crate::app::widgets::widgets_menu::{SettingType, SettingsMenu};
use crate::simulation::renderer::Renderer;
use crate::simulation::renderer_helpers::density_img_path;
use eframe::egui::global_dark_light_mode_switch;
use eframe::{egui, epi};
use image::imageops::FilterType::Triangle;
use image::GenericImageView;
use simplelog::*;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

/// Entry-point for the fluid simulation application
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    /// The "id" of the last showed image in the application of a given simulation
    /// in the context of specific simulation
    #[cfg_attr(feature = "persistence", serde(skip))]
    current_frame: i64,

    /// The fluid driver that renders its state in a file
    #[cfg_attr(feature = "persistence", serde(skip))]
    renderer: Renderer,

    /// The progress of the simulation
    #[cfg_attr(feature = "persistence", serde(skip))]
    simulation_progress: f32,

    /// Between-threads receiver, participant in a channel open between the
    /// renderer and the application
    #[cfg_attr(feature = "persistence", serde(skip))]
    signal_receiver: Receiver<i64>,

    /// Collection of all the widgets in the application
    #[cfg_attr(feature = "persistence", serde(skip))]
    settings_menu: SettingsMenu,

    /// The last showed image is cached.
    #[cfg_attr(feature = "persistence", serde(skip))]
    cached_image: Option<CachedImage>,

    /// The play button has been clicked, and now the simulation should be displayed frame by frame
    #[cfg_attr(feature = "persistence", serde(skip))]
    is_play_button_on: bool,

    /// The simulation is currently in progress
    #[cfg_attr(feature = "persistence", serde(skip))]
    is_simulation_in_process: bool,

    /// Is a fluid simulated and ready to be showed
    #[cfg_attr(feature = "persistence", serde(skip))]
    is_simulation_ready: bool,
}

impl App {
    /// Creates new App instance
    pub fn new(renderer: Renderer) -> Self {
        let (_, signal_receiver) = mpsc::channel();

        Self {
            current_frame: 0,
            renderer,
            simulation_progress: 1.0,
            signal_receiver,
            settings_menu: SettingsMenu::default(),
            cached_image: None,
            is_play_button_on: false,
            is_simulation_in_process: false,
            is_simulation_ready: false,
        }
    }

    /// Returns how zoomed is the simulation result image in the application.
    fn get_zoom_factor(&self) -> Option<u8> {
        for i in self.settings_menu.settings_menu.iter() {
            if let SettingType::Viewport(result) = i {
                return Some(result.image_resize_factor);
            }
        }
        None
    }

    /// Shows the cached image if it should. This is a helper function of [`Self::show_image()`].
    fn show_cached_image_in_ui(
        image: &CachedImage,
        path: &str,
        zoom_factor: u8,
        ui: &mut egui::Ui,
    ) -> Result<(), ()> {
        if !image.has_changed && image.consists_of(path, zoom_factor) {
            ui.image(image.rendered_texture, image.dimensions);
            return Ok(());
        }
        Err(())
    }

    /// Generates a new image, shows it in the ui, and saves it as a cached image. This is a helper
    /// function of [`Self::show_image()`].
    fn generate_cached_image(
        &mut self,
        image_path: &str,
        zoom_factor: u8,
        frame: &epi::Frame,
        ui: &mut egui::Ui,
    ) {
        let image = image::open(image_path)
            .expect(&("Couldn't open image ".to_owned() + image_path))
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
        self.cached_image = Some(CachedImage {
            path: image_path.to_string(),
            zoom_factor,
            dimensions: size,
            rendered_texture: texture_id,
            has_changed: false,
        });
    }

    /// Shows the image with the given path in the ui, by either taking it from the cached image,
    /// or by generating it if it is not cached yet.
    fn show_image(&mut self, image_path: &str, frame: &epi::Frame, ui: &mut egui::Ui) {
        let zoom_factor = self
            .get_zoom_factor()
            .expect("A viewport setting should exsist.");

        let mut should_update_cached_image = false;

        match &self.cached_image {
            Some(image) => {
                should_update_cached_image =
                    Self::show_cached_image_in_ui(image, image_path, zoom_factor, ui).is_err();
            }
            None => self.generate_cached_image(image_path, zoom_factor, frame, ui),
        }

        if should_update_cached_image {
            self.generate_cached_image(image_path, zoom_factor, frame, ui);
        }
    }

    /// Displays the next frame of the simulation in the central panel
    fn move_simulation_frame(&mut self, next_frame: i64, frame: &epi::Frame, ui: &mut egui::Ui) {
        let image_path =
            density_img_path!(&self.renderer.rendering_listener.save_into_dir, next_frame);
        simplelog::debug!(
            "Reading from path: {}, save_into_dir: {}",
            image_path,
            self.renderer.rendering_listener.save_into_dir
        );

        if image::open(image_path).is_err() {
            return;
        }

        self.simulation_progress =
            next_frame as f32 / (self.renderer.fluid.simulation_configs.frames - 1) as f32;

        self.show_image(&image_path, frame, ui);

        frame.request_repaint();
    }

    /// Manages the next frame - either takes it from a channel open between the renderer and the
    /// applicaiton, or directly increments the current_frame.
    fn manage_next_frame(&mut self, frame: &epi::Frame) {
        let frames_count = self.renderer.fluid.simulation_configs.frames;

        if self.is_simulation_in_process {
            if self.current_frame < frames_count - 1 {
                let last_frame = self.current_frame;

                self.current_frame = self
                    .signal_receiver
                    .try_recv()
                    .unwrap_or(self.current_frame);

                if last_frame != self.current_frame {
                    self.is_simulation_ready = true;
                }
            }

            frame.request_repaint();
        } else {
            self.current_frame += 1;
        }

        if self.current_frame == frames_count - 1 {
            self.is_play_button_on = false;
            self.is_simulation_in_process = false;
            self.is_simulation_ready = true;
        }
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
            self.current_frame = 0;
        }

        ui.separator();

        if ui.button("Simulate fluid").clicked() {
            self.simulation_progress = 0.0;
            self.current_frame = 0;

            self.signal_receiver = self.renderer.render();
            self.is_simulation_in_process = true;
            self.is_play_button_on = true;
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

        if self.is_play_button_on {
            self.manage_next_frame(frame);
        }

        if self.is_simulation_ready {
            self.move_simulation_frame(self.current_frame, frame, ui);
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
    use crate::simulation::renderer_helpers::density_img_path;

    #[test]
    fn density_img_path_str_works() {
        assert_eq!(
            density_img_path!("holiday_dir", 0),
            "holiday_dir/density0.jpg"
        );
    }
}
