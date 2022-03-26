use crate::simulation::configs::SimulationConfigs;

/// Defines every obstacle's behaviour
pub trait Obstacle {
    /// Get up left and down right point using which the obstacle is approximated.
    fn get_approximate_points(&mut self) -> &mut Vec<line_drawing::Point<i64>>;
}

/// Enum describing the various obstacles' types. This is what unifies all the widgets
/// and is used fot storing them in collections.
#[derive(Clone)]
pub enum ObstaclesType {
    /// Used for describing the [`Rectangle`] type
    Rectangle(Rectangle),
}

impl Obstacle for ObstaclesType {
    fn get_approximate_points(&mut self) -> &mut Vec<line_drawing::Point<i64>> {
        match self {
            ObstaclesType::Rectangle(rectangle) => &mut rectangle.approximate_points,
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
    approximate_points: Vec<line_drawing::Point<i64>>,
}

impl Clone for Rectangle {
    fn clone(&self) -> Rectangle {
        Rectangle {
            down_left_point: self.down_left_point,
            up_right_point: self.up_right_point,
            approximate_points: self.approximate_points.clone(),
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
            approximate_points: vec![down_left_point, up_right_point],
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
    fn get_approximate_points(&mut self) -> &mut Vec<line_drawing::Point<i64>> {
        &mut self.approximate_points
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::obstacle::Rectangle;

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
}
