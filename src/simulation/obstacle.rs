use line_drawing::Bresenham;
use std::collections::HashMap;

use super::fluid::ContainerWall;

/// Defines every obstacle's behaviour
pub trait Obstacle {
    /// Returns all countour points with their direction
    fn get_sides_direction(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>>;
}

/// Rectangle obstacle which is fit parallely with respect to the
/// coordinate system. It is defined by its uppest left vertex point and
/// the most down right vertex point. **_Note:_** It is currently designed for parallel
/// obstacles only
pub struct Rectangle {
    /// uppest left vertex point. See [`Rectangle`]'s description
    up_left_point: line_drawing::Point<i64>,
    /// the most down right vertex point. See [`Rectangle`]'s description
    down_right_point: line_drawing::Point<i64>,
}

impl Rectangle {
    /// Create new Rectangle
    pub fn new(
        up_left_point: line_drawing::Point<i64>,
        down_right_point: line_drawing::Point<i64>,
        fluid_container_size: u32,
    ) -> Self {
        let result = Self {
            up_left_point: up_left_point,
            down_right_point: down_right_point,
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
    fn get_sides_direction(&self) -> HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> {
        let mut result: HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> = HashMap::new();

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

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::fluid::ContainerWall;
    use crate::simulation::obstacle::{Obstacle, Rectangle};
    use std::collections::{HashMap, HashSet};
    use std::hash::Hash;

    fn iters_equal_anyorder<T>(mut i1: impl Iterator<Item = T>, i2: impl Iterator<Item = T>) -> bool
    where
        T: Eq + Hash,
    {
        let set: HashSet<T> = i2.collect();
        i1.all(|x| set.contains(&x))
    }

    #[test]
    fn rectangle_get_sides_direction_works() {
        let rectangle_obstacle = Rectangle {
            up_left_point: (8, 10),
            down_right_point: (10, 8),
        };
        let expected_result = HashMap::from([
            (ContainerWall::West, vec![(8, 8), (8, 9), (8, 10)]),
            (ContainerWall::North, vec![(8, 10), (9, 10), (10, 10)]),
            (ContainerWall::East, vec![(10, 10), (10, 9), (10, 8)]),
            (ContainerWall::South, vec![(8, 8), (9, 8), (10, 8)]),
        ]);
        let result = rectangle_obstacle.get_sides_direction();
        assert_eq!(result.len(), 4);
        for (key, value) in expected_result {
            assert!(iters_equal_anyorder(
                result.get(&key).unwrap().iter(),
                value.iter()
            ));
        }
    }

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
        let rectangle_obstacle = Rectangle::new((10, 8), (8, 10), 128);
    }
}
