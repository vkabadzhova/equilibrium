use crate::simulation::configs::SimulationConfigs;

/// Defines every obstacle's behaviour
pub trait Obstacle {
    /// Get up left and down right point using which the obstacle is approximated.
    fn get_approximate_points(&self) -> Vec<line_drawing::Point<i64>>;
}

/// Enum describing the various obstacles' types. This is what unifies all the widgets
/// and is used fot storing them in collections.
#[derive(Clone, Copy)]
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
#[derive(Copy)]
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
