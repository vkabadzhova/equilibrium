use super::fluid::ContainerWall;
use line_drawing::Bresenham;
use std::collections::HashMap;

/// Various obstacles defined by a vector with their points can be put into the simulation,
/// as long as the obstacle's points are inside the fluid's container. The fluid will avoid those.
pub struct Obstacle {
    /// The points by which the obstacle is defined
    pub vertices: Vec<line_drawing::Point<i32>>,
    /// Collection of all contour lines ("edges") that the container have.
    pub countour_points: Vec<Vec<line_drawing::Point<i32>>>,
}

impl Obstacle {
    fn generate_countour_lines(
        vertices: &Vec<line_drawing::Point<i32>>,
    ) -> Vec<Vec<line_drawing::Point<i32>>> {
        let mut result: Vec<Vec<line_drawing::Point<i32>>> = Vec::new();
        for point in vertices.iter() {
            let next_point = match vertices.iter().next() {
                Some(val) => *val,
                None => continue,
            };

            result.push(Bresenham::new(*point, next_point).into_iter().collect());
        }
        result
    }

    /// Create new obstacle
    pub fn new(points: Vec<line_drawing::Point<i32>>) -> Self {
        Self {
            countour_points: Obstacle::generate_countour_lines(&points),
            vertices: points,
        }
    }

    /// Verifies if all points in given vector are in the fluid's field
    pub fn are_all_points_valid(&self, fluid_container_size: u32) -> bool {
        let container_size = i64::from(fluid_container_size);
        for point in self.vertices.iter() {
            if i64::from(point.0) >= container_size || i64::from(point.1) >= container_size {
                return false;
            }
        }
        true
    }

    /// Determines the type of all the points that lie on a line between the output's tuple points.
    /// The line is found via the Bresenhams' algorithm.
    /// 
    /// *Returns:* HashMap with the <`index in the self.countour_points vector`, `wall type`>. 
    pub fn determine_sides(&self) -> HashMap<u16, ContainerWall> {
        unimplemented!();
    }
}
