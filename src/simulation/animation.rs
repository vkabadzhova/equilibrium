use crate::simulation::obstacle::{ObstaclesType, Rectangle};

static CELL_SIZE: i64 = 8;

pub(crate) fn normalized_width(width: i64) -> i64 {
    width / CELL_SIZE
}

pub(crate) fn normalized_height(height: i64) -> i64 {
    // FIXME: Width and height are always the same, right?
    //        Probably better to remove one of them to improve clarity.
    normalized_width(height)
}

pub(crate) fn random_cell_bitmap(width: i64, height: i64) -> Vec<bool> {
    let cells = width / CELL_SIZE * height / CELL_SIZE;
    let mut cell_bitmap: Vec<bool> = vec![false; cells as usize];
    for i in 0..cells {
        cell_bitmap[i as usize] = rand::random();
    }
    cell_bitmap
}

pub(crate) fn cell_bitmap_to_obstacles(
    cell_bitmap: &Vec<bool>,
    normalized_width: i64,
    normalized_height: i64,
) -> Vec<ObstaclesType> {
    let mut obstacles: Vec<ObstaclesType> = vec![];
    let mut cell_x = 0;
    let mut cell_y = 0;

    // FIXME: This seems a bit dangerous.
    let width = normalized_width * CELL_SIZE;

    for cell_flag in cell_bitmap {
        if *cell_flag {
            let p: line_drawing::Point<i64> = ((cell_x - 1) * CELL_SIZE, (cell_y - 1) * CELL_SIZE);
            let q: line_drawing::Point<i64> = (cell_x * CELL_SIZE, cell_y * CELL_SIZE);
            let obstacle: ObstaclesType =
                ObstaclesType::Rectangle(Rectangle::new(p, q, width as u32));
            obstacles.push(obstacle);
        }

        // Go to next cell.
        cell_x += 1;
        if cell_x >= normalized_width {
            cell_x = 0;
            cell_y += 1;
            if cell_y >= normalized_height {
                break;
            }
        }
    }

    obstacles
}

fn obstacles_to_cell_bitmap(
    obstacles: &Vec<ObstaclesType>,
    normalized_width: i64,
    normalized_height: i64,
) -> Vec<bool> {
    // Store a bitmap indicating whether each of the cells is active or not.
    let mut cell_bitmap: Vec<bool> = vec![false; (normalized_height * normalized_height) as usize];
    for obstacle in obstacles {
        match obstacle {
            // FIXME: Verify that it is a square CELL_SIZE x CELL_SIZE in size
            ObstaclesType::Rectangle(rect) => {
                cell_bitmap[idx!(
                    rect.up_right_point.0 / CELL_SIZE,
                    rect.up_right_point.1 / CELL_SIZE,
                    normalized_width
                )] = true
            }
        }
    }
    cell_bitmap
}

// By definition each generation is a pure function.
// The algorithm accepts the previous generation and generates the next one.
pub(crate) fn game_of_life(
    previous_generation: &Vec<ObstaclesType>,
    width: i64,
    height: i64,
) -> Vec<ObstaclesType> {
    let normalized_width = width / CELL_SIZE;
    let normalized_height = height / CELL_SIZE;

    // Store a bitmap indicating whether each of the cells is active or not.
    let cell_bitmap =
        obstacles_to_cell_bitmap(previous_generation, normalized_width, normalized_height);

    // Clone the bitmap and use that as a foundation for calculating the new generation.
    // This is the actual game of life algorithm implementation.
    let mut cell_bitmap_new_generation = cell_bitmap.clone();
    let mut cell_x = 0;
    let mut cell_y = 0;
    for cell_flag in cell_bitmap.iter() {
        let mut cell_active_neighbours = 0;
        // Count number of active neighbours of currently inspected cell.
        if cell_y > 0 && cell_bitmap[idx!(cell_x, cell_y - 1, normalized_width)] {
            cell_active_neighbours += 1;
        }
        if cell_y < normalized_height - 1 && cell_bitmap[idx!(cell_x, cell_y + 1, normalized_width)]
        {
            cell_active_neighbours += 1;
        }
        if cell_x > 0 && cell_bitmap[idx!(cell_x - 1, cell_y, normalized_width)] {
            cell_active_neighbours += 1;
        }
        if cell_x < normalized_width - 1 && cell_bitmap[idx!(cell_x + 1, cell_y, normalized_width)]
        {
            cell_active_neighbours += 1;
        }

        // Apply game of life rules.
        if *cell_flag && !(2..=3).contains(&cell_active_neighbours) {
            cell_bitmap_new_generation[idx!(cell_x, cell_y, normalized_width)] = false;
        } else if !*cell_flag && cell_active_neighbours > 3 {
            cell_bitmap_new_generation[idx!(cell_x, cell_y, normalized_width)] = true;
        } else {
            cell_bitmap_new_generation[idx!(cell_x, cell_y, normalized_width)] = *cell_flag;
        }

        // Go to next cell.
        cell_x += 1;
        if cell_x >= normalized_width {
            cell_x = 0;
            cell_y += 1;
            if cell_y >= normalized_height {
                break;
            }
        }
    }

    // Create obstacles from cell bitmap of new generation.
    let new_generation = cell_bitmap_to_obstacles(
        &cell_bitmap_new_generation,
        normalized_width,
        normalized_height,
    );
    new_generation
}
