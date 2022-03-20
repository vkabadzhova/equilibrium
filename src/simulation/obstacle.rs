use super::fluid::ContainerWall;
use crate::simulation::configs::SimulationConfigs;
use line_drawing::Bresenham;
use log::debug;
use std::collections::HashMap;

/// Defines every obstacle's behaviour
pub trait Obstacle {
    /// Returns all countour points with their direction
    //fn get_perimeter(&mut self) -> &HashMap<ContainerWall, Vec<line_drawing::Point<i64>>>;

    /// Retrurns the all the coordinates which are part of the obstacle, including both its
    /// parameter and inside
    //fn get_area(&mut self) -> &Vec<line_drawing::Point<i64>>;

    /// Get up left and down right point using which the obstacle is approximated.
    fn get_approximate_points(&self) -> Vec<line_drawing::Point<i64>>;
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
    /// Collection with all the sides of an obstacle. The sides are
    /// defined via compass direction. See [`ContainerWall`].
    perimeter: HashMap<ContainerWall, Vec<line_drawing::Point<i64>>>,
    area: Vec<line_drawing::Point<i64>>,
}

impl Clone for Rectangle {
    fn clone(&self) -> Rectangle {
        Rectangle {
            up_left_point: self.up_left_point,
            down_right_point: self.down_right_point,
            perimeter: HashMap::new(),
            area: Vec::new(),
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self::new((50, 120), (127, 110), SimulationConfigs::default().size)
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
            up_left_point: up_left_point,
            down_right_point: down_right_point,
            perimeter: HashMap::new(),
            area: Vec::new(),
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

    fn calculate_perimeter(&mut self) -> &HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> {
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

        self.perimeter = result;

        &self.perimeter
    }

    /*
    fn calculate_area(&mut self) {
        let mut result: Vec<line_drawing::Point<i64>> = Vec::new();
        let obstacle_perimeter = self.get_perimeter();

        // safety note (for unwrap): there will always be a perimeter, at least one point,
        // otherwise there won't be an obstacle
        for west_point in obstacle_perimeter.get(&ContainerWall::West).unwrap() {
            // find the corresponding East neighbour point of the current West point of the
            // obstacle, i.e. the one with the same y, but different x and replace all
            // ContainerWall types with [`ContainerWall::DefaultWall`]
            // Like so:
            //  _ _ _ _ _
            // | | | | | |
            // |E|-|-|>|W|
            // | | | | | |
            // |_|_|_|_|_|
            //
            // where E - East, W - West, arrow (`-->`) - the path and its direction in which the algorithm
            // will traverse and put [`ContainerWall::DefaultWall`] in between
            //
            let east_neighbour_point = obstacle_perimeter
                .get(&ContainerWall::East)
                .unwrap()
                .into_iter()
                .find(|east_point| east_point.1 == west_point.1 && east_point.0 != west_point.0);

            match east_neighbour_point {
                Some(east_point) => {
                    for x_coordinate in west_point.0..=east_point.0 {
                        result.push((x_coordinate, east_point.1));
                    }
                }

                None => {
                    debug!(
                        "West point ({}, {}) does not have a corresponding east.",
                        west_point.0.to_string(),
                        west_point.1.to_string()
                    )
                }
            }
        }
    }
    */
}

impl Obstacle for Rectangle {
    /*
    fn get_perimeter(&mut self) -> &HashMap<ContainerWall, Vec<line_drawing::Point<i64>>> {
        if self.perimeter.is_empty() {
            self.calculate_perimeter();
        }

        &self.perimeter
    }

    fn get_area(&mut self) -> &Vec<line_drawing::Point<i64>> {
        if self.area.is_empty() {
            self.calculate_area();
        }

        &self.area
    }
    */

    fn get_approximate_points(&self) -> Vec<line_drawing::Point<i64>> {
        vec![self.up_left_point, self.down_right_point]
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::obstacle::Rectangle;

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
}
