use eframe::egui;
use egui::*;

use crate::simulation::obstacle::{Obstacle, ObstaclesType, Rectangle};

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[derive(Copy, Clone)]
pub struct ObstacleWidget {
    enabled: bool,
    // TODO: make collection
    obstacles: ObstacleLayout,
}

impl Default for ObstacleWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            obstacles: ObstacleLayout::default(),
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

        CollapsingHeader::new(&self.obstacles.name)
            .default_open(false)
            .show(ui, |ui| self.obstacles.ui(ui));
    }
}

impl ObstacleWidget {
    fn gallery_grid_contents(&mut self) {
        let Self { .. } = self;
    }
}

/*
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct Tree {
    name: String,
    obstacle: ObstacleLayout,
}

impl Tree {
    pub fn demo() -> Self {
        Tree {
            name: String::from("Rectangle"),
            obstacle: ObstacleLayout {
                obstacle: ObstaclesType::Rectangle(Rectangle::new((50, 120), (120, 110), 128)),
            },
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        self.obstacle.ui(ui)
    }
}
*/

#[derive(Clone)]
/// The inner part of every [`Tree`]
struct ObstacleLayout {
    name: String,
    obstacle: ObstaclesType,
}

impl ObstacleLayout {
    pub fn ui(&mut self, ui: &mut Ui) {
        let mut approximate_points = self.obstacle.get_approximate_points();
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

impl Default for ObstacleLayout {
    fn default() -> Self {
        ObstacleLayout {
            name: "Rectangle".to_string(),
            obstacle: ObstaclesType::Rectangle(Rectangle::default()),
        }
    }
}
