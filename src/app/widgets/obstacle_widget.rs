use eframe::egui;
use egui::*;

use crate::simulation::obstacle::{Obstacle, ObstaclesType, Rectangle};

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[derive(Clone)]
pub struct ObstacleWidget {
    enabled: bool,
    /// Collection of obstacle types. The elements describe each possible type (e.g. Rectangle, Circle,
    /// etc.) and their names must be unique
    pub obstacles: Vec<ObstacleLayout>,
    last_obstacle_id: u32,
}

impl Default for ObstacleWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            obstacles: vec![ObstacleLayout::default()],
            last_obstacle_id: 0,
        }
    }
}

impl super::Setting for ObstacleWidget {
    fn name(&self) -> &'static str {
        "Obstacle"
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

impl super::View for ObstacleWidget {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.enabled, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |_| {
                    self.gallery_grid_contents();
                });
        });

        ui.separator();

        for obstacle in self.obstacles.iter_mut() {
            CollapsingHeader::new(obstacle.name.clone())
                .default_open(false)
                .show(ui, |ui| obstacle.ui(ui));
        }

        if ui.button("+").clicked() {
            self.add_obstacle();
        }

        self.obstacles.retain(|el| el.action == Action::Keep);
    }
}

impl ObstacleWidget {
    fn gallery_grid_contents(&mut self) {
        let Self { .. } = self;
    }

    fn add_obstacle(&mut self) {
        let default_obstacle_layout = ObstacleLayout::default();

        self.last_obstacle_id += 1;

        self.obstacles.push(ObstacleLayout {
            name: format!("{}/{}", default_obstacle_layout.name, self.last_obstacle_id),
            obstacle: default_obstacle_layout.obstacle,
            action: default_obstacle_layout.action,
        });
    }
}

/// Describes if an element should be deleted or kept alive in the next frame.
#[derive(Clone, PartialEq)]
pub enum Action {
    /// The element should be kept available
    Keep,
    /// The element should be deleted
    Delete,
}

#[derive(Clone)]
/// The inner part of the obstacle placement UI. This includes the number of points for every
/// single obstacle type.
pub struct ObstacleLayout {
    /// The name of the obstacle type, e.g. "Circle", "Rectangle", etc.
    pub name: String,
    /// The obstacle per se - the data for the given obstacle
    pub obstacle: ObstaclesType,
    /// When marked as [`Action::Delete`], the obstacle is deleted in the next frame.
    pub action: Action,
}

impl Default for ObstacleLayout {
    fn default() -> Self {
        ObstacleLayout {
            name: "Rectangle".to_string(),
            obstacle: ObstaclesType::Rectangle(Rectangle::default()),
            action: Action::Keep,
        }
    }
}

impl ObstacleLayout {
    /// Creates the layout for a single obstacle type
    pub fn ui(&mut self, ui: &mut Ui) {
        if ui
            .button(RichText::new("delete").color(Color32::RED))
            .clicked()
        {
            self.action = Action::Delete;
        }

        let approximate_points = self.obstacle.get_approximate_points();
        for i in 0..approximate_points.len() {
            ui.label("Point: ");
            ui.add(egui::DragValue::new(&mut approximate_points[i].0).speed(1.0));
            ui.add(egui::DragValue::new(&mut approximate_points[i].1).speed(1.0));
            if approximate_points[i].0 < 0 {
                approximate_points[i].0 = 0;
            }

            if approximate_points[i].1 < 0 {
                approximate_points[i].1 = 0;
            }
        }
    }
}
