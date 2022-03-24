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
                vec![rectangle.up_left_point, rectangle.down_right_point]
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
    pub up_left_point: line_drawing::Point<i64>,
    /// the most down right vertex point. See [`Rectangle`]'s description
    pub down_right_point: line_drawing::Point<i64>,
}

impl Clone for Rectangle {
    fn clone(&self) -> Rectangle {
        Rectangle {
            up_left_point: self.up_left_point,
            down_right_point: self.down_right_point,
        }
    }
}

impl Default for Rectangle {
    // Try with other default obstacles:
    //Self::new((50, 120), (127, 110), SimulationConfigs::default().size)
    //Self::new((10, i64::from(size - 2)), (i64::from(size - 2), 10), size)
    //Self::new((0, i64::from(size - 1)), (10, i64::from(size - 10)), size)
    //Self::new((0, 10), (10, 0), 128)
    fn default() -> Self {
        let size = SimulationConfigs::default().size;
        Self::new(
            (i64::from(size - 50), i64::from(size - 1)),
            (i64::from(size - 1), i64::from(size - 50)),
            size,
        )
    }
}

impl Rectangle {
    /// Create new Rectangle
    pub fn new(
        up_left_point: line_drawing::Point<i64>,
        down_right_point: line_drawing::Point<i64>,
        fluid_container_size: u32,
    ) -> Self {
        let result = Self {
            up_left_point,
            down_right_point,
        };

        if !result.are_all_points_valid(i64::from(fluid_container_size)) {
            panic!("Invalid input for Rectangle");
        }

        result
    }

    /// Check if all parameters are valid
    pub fn are_all_points_valid(&self, fluid_container_size: i64) -> bool {
        self.up_left_point.0 != self.down_right_point.0
            && self.up_left_point.1 != self.down_right_point.1
            && self.up_left_point.0 < self.down_right_point.0
            && self.up_left_point.1 > self.down_right_point.1
            && vec![
                self.up_left_point.0,
                self.up_left_point.1,
                self.down_right_point.0,
                self.down_right_point.1,
            ]
            .iter()
            .all(|e| *e < fluid_container_size)
    }
}

impl Obstacle for Rectangle {
    fn get_approximate_points(&self) -> Vec<line_drawing::Point<i64>> {
        vec![self.up_left_point, self.down_right_point]
    }

    fn get_approximate_walls(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> {
        let points = self.get_approximate_points();

        let mut result: HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> = HashMap::new();

        // The upper wall of the approximated obstacle is defined by upper_left_point.x, upper_left_point.y --> down_right_point.x, upper_left_point.y
        let upper_wall = ((points[0].0, points[0].1), (points[1].0, points[0].1));
        result.insert(
            ContainerWall::North,
            Bresenham::new(upper_wall.0, upper_wall.1)
                .into_iter()
                .collect(),
        );

        // The down wall of the approximated obstacle is defined by upper_left_point.x, down_right_point.y --> down_right_point.x, upper_left_point.y
        let down_wall = ((points[0].0, points[1].1), (points[1].0, points[1].1));
        result.insert(
            ContainerWall::South,
            Bresenham::new(down_wall.0, down_wall.1)
                .into_iter()
                .collect(),
        );

         // The left wall of the approximated obstacle is defined by upper_left_point.x, upper_left_point.y --> upper_left_point.x, down_right_point.y
         let left_wall = ((points[0].0, points[0].1), (points[0].0, points[1].1));
         result.insert(
             ContainerWall::West,
             Bresenham::new(left_wall.0, left_wall.1)
                 .into_iter()
                 .collect(),
         );
 
         // The right wall of the approximated obstacle is defined by down_right_point.x, upper_left_point.y --> down_right_point.x, down_right_point.y
         let right_wall = ((points[1].0, points[0].1), (points[1].0, points[1].1));
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
    fn rectangle_wrong_parameters_on_creation_panics() {
        let up_left_point = (10, 10);
        // should be DOWN right vertex point, but it is UP right
        let up_right_point = (12, 12);
        Rectangle::new(up_left_point, up_right_point, 128);
    }

    #[test]
    #[should_panic]
    fn rectangle_swapped_parameters_panics() {
        Rectangle::new((10, 8), (8, 10), 128);
    }

    use std::collections::HashMap;

    #[test]
    fn get_approximate_walls() {
        let rectangle = Rectangle::new((8, 10), (10, 8), 11);
        assert_eq!(
            rectangle.get_approximate_walls(),
            HashMap::from([
                (ContainerWall::North, vec![(8, 10), (9, 10), (10, 10)]),
                (ContainerWall::South, vec![(8, 8), (9, 8), (10, 8)]),
                (ContainerWall::West, vec![(8, 10), (8, 9), (8, 8)]),
                (ContainerWall::East, vec![(10, 10), (10, 9), (10, 8)]),
            ])
        );
    }
}
