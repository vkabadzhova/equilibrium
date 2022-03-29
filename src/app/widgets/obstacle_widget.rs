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
    /// The obstacles' color in the scene
    pub color: Color32,
    last_obstacle_id: u32,
}

impl Default for ObstacleWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            obstacles: vec![ObstacleLayout::default()],
            color: egui::Color32::RED,
            last_obstacle_id: 0,
        }
    }
}

impl super::Setting for ObstacleWidget {
    fn name(&self) -> &'static str {
        "💢 Obstacle"
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
                .show(ui, |_| {});
        });

        ui.separator();

        ui.label("Choose obstacles' color");
        ui.color_edit_button_srgba(&mut self.color);
        ui.end_row();

        ui.separator();

        ui.label("Obstacles:");

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
    /// Adds new obstacle with the appropriate configurations: Every obstacle collapsing header
    /// should have a unique name. Therefore, the type of the obstacle and its serial number. If an
    /// obstacle with a serial number in the middle of the sequence has been deleted, that doesn't
    /// affect the newly generated ones, and their order keeps progressing from the point it has
    /// been left at.
    ///
    /// Example:
    /// ```
    /// use equilibrium::app::widgets::obstacle_widget::*;
    ///
    /// let mut obstacle_widget = ObstacleWidget::default();
    ///
    /// // ----------- Create and delete a new obstacle -----------
    /// obstacle_widget.add_obstacle();
    ///
    /// // Assert it is correctly counted
    /// assert_eq!(
    ///     obstacle_widget.obstacles.last().unwrap().name,
    ///     "Rectangle/1"
    /// );
    ///
    /// obstacle_widget.obstacles.pop();
    /// obstacle_widget.add_obstacle();
    ///
    /// // Main: Assert the last element after removal is correctly counted.
    /// assert_eq!(
    ///     obstacle_widget.obstacles.last().unwrap().name,
    ///     "Rectangle/2"
    /// );
    ///
    /// // Assert Rectangle/1 is missing
    /// let count_rectangle_one = obstacle_widget
    ///     .obstacles
    ///     .iter()
    ///     .filter(|&el| el.name == "Rectangle/1")
    ///     .count();
    ///
    /// assert_eq!(count_rectangle_one, 0);
    ///
    /// ```
    pub fn add_obstacle(&mut self) {
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

#[cfg(test)]
mod test {

    use crate::app::widgets::obstacle_widget::*;

    #[test]
    fn default_obstacle() {
        let obstacle_widget = ObstacleWidget::default();
        assert_eq!(obstacle_widget.obstacles.len(), 1);
    }

    #[test]
    fn add_obstacle() {
        let mut obstacle_widget = ObstacleWidget::default();

        obstacle_widget.add_obstacle();

        // Safety note: we just created an obstacle, so last() will return it.
        assert_eq!(
            obstacle_widget.obstacles.last().unwrap().name,
            "Rectangle/1"
        );

        // ----------- Create and delete a new obstacle -----------
        obstacle_widget.add_obstacle();

        // Assert it is correctly counted
        assert_eq!(
            obstacle_widget.obstacles.last().unwrap().name,
            "Rectangle/2"
        );

        obstacle_widget.obstacles.pop();
        obstacle_widget.add_obstacle();

        // Main assert: Assure the last element after removal is correctly counted.
        assert_eq!(
            obstacle_widget.obstacles.last().unwrap().name,
            "Rectangle/3"
        );

        // Assert Rectangle/2 is missing
        let count_rectangle_two = obstacle_widget
            .obstacles
            .iter()
            .filter(|&el| el.name == "Rectangle/2")
            .count();

        assert_eq!(count_rectangle_two, 0);
    }
}
