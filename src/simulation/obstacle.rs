use std::collections::HashMap;

use line_drawing::Bresenham;

use super::fluid::ContainerWall;

/// Defines every obstacle's behaviour
pub trait Obstacle {
    /// Verifies if all points in given vector are in the fluid's field
    fn are_all_points_valid(&self, fluid_container_size: i64) -> bool;

    /// Returns all countour points with their direction
    fn get_sides_direction(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>>;
}

/// Rectangle obstacle which is fit parallely with respect to the
/// coordinate system
pub struct Rectangle {
    up_left_point: line_drawing::Point<i64>,
    down_right_point: line_drawing::Point<i64>,
}

impl Obstacle for Rectangle {
    fn are_all_points_valid(&self, fluid_container_size: i64) -> bool {
        self.up_left_point.0 != self.down_right_point.0
            && self.up_left_point.1 != self.down_right_point.1
            && vec![
                self.up_left_point.0,
                self.up_left_point.1,
                self.down_right_point.0,
                self.down_right_point.1,
            ]
            .iter()
            .all(|e| *e < fluid_container_size)
    }

    fn get_sides_direction(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> {
        let mut result: HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> = HashMap::new();

        // North wall
        result.insert(
            ContainerWall::North,
            Bresenham::new(
                self.up_left_point,
                (self.down_right_point.0, self.up_left_point.1),
            )
            .into_iter()
            .collect(),
        );

        // South wall
        result.insert(
            ContainerWall::South,
            Bresenham::new(
                self.down_right_point,
                (self.up_left_point.0, self.down_right_point.1),
            )
            .into_iter()
            .collect(),
        );

        // West wall
        result.insert(
            ContainerWall::West,
            Bresenham::new(
                self.up_left_point,
                (self.up_left_point.0, self.down_right_point.1),
            )
            .into_iter()
            .collect(),
        );

        // East wall
        result.insert(
            ContainerWall::East,
            Bresenham::new(
                self.down_right_point,
                (self.down_right_point.0, self.up_left_point.1),
            )
            .into_iter()
            .collect(),
        );

        result
    }
}
