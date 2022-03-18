use eframe::egui;
use egui::{color::*, *};

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[derive(Copy, Clone)]
pub struct ObstacleWidget {
    enabled: bool,
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

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct Tree(String, SubTree);

impl Tree {
    pub fn demo() -> Self {
        Self(
            String::from("root"),
            SubTree(vec![
                SubTree(vec![SubTree::default(); 4]),
                SubTree(vec![SubTree(vec![SubTree::default(); 2]); 3]),
            ]),
        )
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Action {
        self.1.ui(ui, 0, "root", &mut self.0)
    }
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct SubTree(Vec<SubTree>);

impl SubTree {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        depth: usize,
        name: &str,
        selected_name: &mut String,
    ) -> Action {
        let response = CollapsingHeader::new(name)
            .default_open(depth < 1)
            .selectable(true)
            .selected(selected_name.as_str() == name)
            .show(ui, |ui| self.children_ui(ui, name, depth, selected_name));
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
        if depth > 0
            && ui
                .button(RichText::new("delete").color(Color32::RED))
                .clicked()
        {
            return Action::Delete;
        }

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

        Action::Keep
    }
}
