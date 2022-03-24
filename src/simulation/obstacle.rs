use crate::simulation::configs::SimulationConfigs;
use crate::simulation::fluid::ContainerWall;
use line_drawing::Bresenham;
use std::collections::HashMap;

/// Defines every obstacle's behaviour
pub trait Obstacle {
    /// Get up left and down right point using which the obstacle is approximated.
    fn get_approximate_points(&self) -> Vec<line_drawing::Point<i64>>;

    /// Get all walls of the approximated obstacle
    fn get_approximate_walls(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>>;
}

/// Enum describing the various obstacles' types. This is what unifies all the widgets
/// and is used fot storing them in collections.
#[derive(Clone)]
pub enum ObstaclesType {
    /// Used for describing the [`Rectangle`] type
    Rectangle(Rectangle),
}

impl Obstacle for ObstaclesType {
    fn get_approximate_points(&self) -> Vec<line_drawing::Point<i64>> {
        match self {
            ObstaclesType::Rectangle(rectangle) => {
                vec![rectangle.down_left_point, rectangle.up_right_point]
            }
        }
    }

    fn get_approximate_walls(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> {
        match self {
            ObstaclesType::Rectangle(rectangle) => rectangle.get_approximate_walls(),
        }
    }
}

/// Rectangle obstacle which is fit parallely with respect to the
/// coordinate system. It is defined by its uppest left vertex point and
/// the most down right vertex point. **_Note:_** It is currently designed for parallel
/// obstacles only
pub struct Rectangle {
    /// uppest left vertex point. See [`Rectangle`]'s description
    pub down_left_point: line_drawing::Point<i64>,
    /// the most down right vertex point. See [`Rectangle`]'s description
    pub up_right_point: line_drawing::Point<i64>,
}

impl Clone for Rectangle {
    fn clone(&self) -> Rectangle {
        Rectangle {
            down_left_point: self.down_left_point,
            up_right_point: self.up_right_point,
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        let size = SimulationConfigs::default().size;
        Self::new(
            (i64::from(size - 10), i64::from(size - 10)),
            (i64::from(size - 1), i64::from(size - 1)),
            size,
        )
    }
}

impl Rectangle {
    /// Create new Rectangle
    pub fn new(
        down_left_point: line_drawing::Point<i64>,
        up_right_point: line_drawing::Point<i64>,
        fluid_container_size: u32,
    ) -> Self {
        let result = Self {
            down_left_point,
            up_right_point,
        };

        if !result.are_all_points_valid(i64::from(fluid_container_size)) {
            panic!("Invalid input for Rectangle");
        }

        result
    }

    /// Check if all parameters are valid
    pub fn are_all_points_valid(&self, fluid_container_size: i64) -> bool {
        self.down_left_point.0 != self.up_right_point.0
            && self.down_left_point.1 != self.up_right_point.1
            && self.down_left_point.0 < self.up_right_point.0
            && self.down_left_point.1 < self.up_right_point.1
            && vec![
                self.down_left_point.0,
                self.down_left_point.1,
                self.up_right_point.0,
                self.up_right_point.1,
            ]
            .iter()
            .all(|e| *e < fluid_container_size)
    }
}

impl Obstacle for Rectangle {
    fn get_approximate_points(&self) -> Vec<line_drawing::Point<i64>> {
        vec![self.down_left_point, self.up_right_point]
    }

    fn get_approximate_walls(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> {
        let points = self.get_approximate_points();

        let mut result: HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> = HashMap::new();

        // The y component of the upper point is constant, x varies.
        let upper_wall = ((points[0].0, points[1].1), (points[1].0, points[1].1));
        result.insert(
            ContainerWall::North,
            Bresenham::new(upper_wall.0, upper_wall.1)
                .into_iter()
                .collect(),
        );

        // The y component of the bottom point is constant, x varies.
        let down_wall = ((points[0].0, points[0].1), (points[1].0, points[0].1));
        result.insert(
            ContainerWall::South,
            Bresenham::new(down_wall.0, down_wall.1)
                .into_iter()
                .collect(),
        );

        // The x component of the left point is constant, y varies. South and North are prered on
        // corners over East and West. Therefore, E and W have smaller lengths.
        let left_wall = (
            (points[0].0, points[0].1 + 1),
            (points[0].0, points[1].1 - 1),
        );
        result.insert(
            ContainerWall::West,
            Bresenham::new(left_wall.0, left_wall.1)
                .into_iter()
                .collect(),
        );

        // The x component of the right point is constant, y varies. South and North are prered on
        // corners over East and West. Therefore, E and W have smaller lengths.
        let right_wall = (
            (points[1].0, points[0].1 + 1),
            (points[1].0, points[1].1 - 1),
        );
        result.insert(
            ContainerWall::East,
            Bresenham::new(right_wall.0, right_wall.1)
                .into_iter()
                .collect(),
        );

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::{
        fluid::ContainerWall,
        obstacle::{Obstacle, Rectangle},
    };

    #[test]
    #[should_panic]
    // Use up_left and down_right points instead of down_left and up_right.
    fn rectangle_wrong_parameters_on_creation_panics() {
        let up_left = (50, 120);
        let down_right = (127, 110);
        Rectangle::new(up_left, down_right, 128);
    }

    #[test]
    #[should_panic]
    fn rectangle_swapped_parameters_panics() {
        let down_left_point = (10, 10);
        let up_right_point = (12, 12);
        Rectangle::new(up_right_point, down_left_point, 128);
    }

    use std::collections::HashMap;

    #[test]
    fn get_approximate_walls() {
        let rectangle = Rectangle::new((8, 8), (10, 10), 11);
        assert_eq!(
            rectangle.get_approximate_walls(),
            HashMap::from([
                (ContainerWall::South, vec![(8, 8), (9, 8), (10, 8)]),
                (ContainerWall::West, vec![(8, 9)]),
                (ContainerWall::North, vec![(8, 10), (9, 10), (10, 10)]),
                (ContainerWall::East, vec![(10, 9)]),
            ])
        );
    }
}
