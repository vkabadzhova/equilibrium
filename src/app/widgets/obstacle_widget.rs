use eframe::egui;
use egui::{color::*, *};

use crate::simulation::obstacle::{ObstaclesType, Rectangle};

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[derive(Copy, Clone)]
pub struct ObstacleWidget {
    enabled: bool,
    // TODO: make collection
    tree: Tree,
}

impl Default for ObstacleWidget {
    fn default() -> Self {
        Self {
            enabled: true,
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

        CollapsingHeader::new("Tree")
            .default_open(false)
            .show(ui, |ui| self.tree.ui(ui));
    }
}

impl ObstacleWidget {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self { enabled, tree } = self;
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
}

impl ObstacleLayout {
    pub fn ui(&mut self, ui: &mut Ui, name: &str, selected_name: &mut String) {
        ui.label("Number of frames");
        /*
        ui.add(egui::DragValue::new(&mut simulation_configs.frames).speed(1.0));
        if simulation_configs.frames < 1 {
            simulation_configs.frames = 1;
        }
        ui.end_row();

        ui.label("Simulation step (delta_t)")
            .on_hover_text("delta_t");
        ui.add(egui::DragValue::new(&mut simulation_configs.delta_t).speed(0.01));
        if simulation_configs.delta_t < 0.0 {
            simulation_configs.delta_t = 0.00;
        }
        ui.end_row();

        ui.label("Simulation window size");
        ui.add(egui::DragValue::new(&mut simulation_configs.size).speed(1.0));
        if simulation_configs.size < 1 {
            simulation_configs.size = 1;
        }
        */

        /*
        let response = CollapsingHeader::new(name)
            .default_open(false)
            .selectable(true)
            .show(ui, |ui| self.children_ui(ui, name, 1, selected_name));
        if response.header_response.clicked() {
            *selected_name = name.to_string();
        }
        response.body_returned.unwrap_or(Action::Keep)
        */
    }
}

/*
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct Tree(String, SubTree);

impl Tree {
    pub fn demo() -> Self {
        Self(
            String::from("root"),
            SubTree(ObstaclesType::Rectangle(Rectangle::new(
                (50, 120),
                (120, 110),
                128,
            ))),
        )
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Action {
        self.1.ui(ui, "Rectangle", &mut self.0)
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct SubTree(ObstaclesType);

impl Default for SubTree {
    fn default() -> Self {
        SubTree(ObstaclesType::Rectangle(Rectangle::default()))
    }
}

impl SubTree {
    pub fn ui(&mut self, ui: &mut Ui, name: &str, selected_name: &mut String) -> Action {
        let response = CollapsingHeader::new(name)
            .default_open(false)
            .selectable(true)
            .show(ui, |ui| self.children_ui(ui, name, 1, selected_name));
        if response.header_response.clicked() {
            *selected_name = name.to_string();
        }
        response.body_returned.unwrap_or(Action::Keep)
    }

    fn children_ui(
        &mut self,
        ui: &mut Ui,
        parent_name: &str,
        depth: usize,
        selected_name: &mut String,
    ) -> Action {
        /*
        if depth > 0
            && ui
                .button(RichText::new("delete").color(Color32::RED))
                .clicked()
        {
            return Action::Delete;
        }
        */

        /*
        if self.0 == Action::Keep {}
        self.0 = std::mem::take(self)
            .0
            .into_iter()
            .enumerate()
            .filter_map(|(i, mut tree)| {
                if tree.ui(
                    ui,
                    depth + 1,
                    &format!("{}/{}", parent_name, i),
                    selected_name,
                ) == Action::Keep
                {
                    Some(tree)
                } else {
                    None
                }
            })
            .collect();

        if ui.button("+").clicked() {
            self.0.push(SubTree::default());
        }
        */

        Action::Keep
    }
}
*/
