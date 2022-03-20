use eframe::egui;
use egui::{color::*, *};

use crate::{
    app,
    simulation::obstacle::{Obstacle, ObstaclesType, Rectangle},
};

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[derive(Copy, Clone)]
pub struct ObstacleWidget {
    enabled: bool,
    name: String,
    // TODO: make collection
    tree: Tree,
}

impl Default for ObstacleWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            name: "Rectangle".to_string(),
            tree: Tree::demo(),
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
                .show(ui, |ui| {
                    self.gallery_grid_contents(ui);
                });
        });

        ui.separator();

        CollapsingHeader::new(&self.name)
            .default_open(false)
            .show(ui, |ui| self.tree.ui(ui));
    }
}

impl ObstacleWidget {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled,
            name,
            tree,
        } = self;
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Keep,
    Delete,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct Tree {
    name: String,
    obstacle: ObstacleLayout,
}

impl Tree {
    pub fn demo() -> Self {
        Tree {
            name: String::from("root"),
            obstacle: ObstacleLayout {
                obstacle: ObstaclesType::Rectangle(Rectangle::new((50, 120), (120, 110), 128)),
                name: "Rectangle".to_string(),
            },
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        self.obstacle.ui(ui, "Rectangle", &mut self.name)
    }
}

#[derive(Clone)]
/// The inner part of every [`Tree`]
struct ObstacleLayout {
    obstacle: ObstaclesType,
    name: String,
}

impl ObstacleLayout {
    pub fn ui(&mut self, ui: &mut Ui, name: &str, selected_name: &mut String) {
        ui.label(&self.name);

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
