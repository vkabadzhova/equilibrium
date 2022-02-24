use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::obstacle::Obstacle;
use geo::algorithm::rotate::RotatePoint;
use geo::{line_string, point};
use log::debug;
use noise::{NoiseFn, Perlin};
use rand::Rng;

/// Address the direction in world-directions style
/// similliar to: mid-point line generation algorithm, etc.
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum ContainerWall {
    /// the side which is parallel to Ox and has lowest x values
    /// in the coordinate system
    West,
    /// All lines which are not parallel to the main axises
    /// and whose average point's y coordinate is on left of the obstacle's
    /// horizontal mediana is considered NorthWest.
    NorthWest,
    /// the side which is parallel to Ox and has highest y values
    /// in the coordinate system
    North,
    /// All lines which are not parallel to the main axises
    /// and whose average point's y coordinate is on right of the obstacle's
    /// horizontal mediana is considered NorthEast.
    NorthEast,
    /// the side which is parallel to Oy and has highest x values
    /// in the coordinate system
    East,
    /// All lines which are not parallel to the main axises
    /// and whose average point's y coordinate is on right of the obstacle's
    /// horizontal mediana is considered SouthEast.
    SouthEast,
    /// the side which is parallel to Ox and has lowest y values
    /// in the coordinate system
    South,
    /// All lines which are not parallel to the main axises
    /// and whose average point's y coordinate is on left of the obstacle's
    /// horizontal mediana is considered NorthEast.
    SouthWest,
    /// Deafult wall variant. Used for filling the inner cells of an obstacles
    DefaultWall,
    /// No wall variant. See [`set_boundaries`].
    NoWall,
}

macro_rules! idx {
    ($x:expr, $y:expr, $size: expr) => {
        ($x.clamp(0, $size - 1) + ($y.clamp(0, $size - 1) * $size)) as usize
    };
}

/// The [`Fluid::lin_solver()`] and other functions of the [`Fluid`] struct solve an equation
/// only for a single dimesion. This is why a differentiation between the two is in need of.
/// Developed for the [`Fluid::set_boundaries()`], etc.
enum Dimension {
    /// X dimension
    X,
    /// Y dimension
    Y,
}

/// The struct that is responsible for simulating the fluid's behavour.
///
/// *Note:*
/// The velocity vector for each cell (i.e. its direction and magnitute) is
/// given by the sum of the vector in the velocities_x and velocities_y coefficients
/// that form a part of normalized vectors, too.
///
///              ^
/// velocities_y |^
///              | \  sum of the two vectors forms a new vector: the direction of the
///              |  \ fluid in that exact cell
///              ---->
///            velocities_x
pub struct Fluid {
    /// general configurations for the simulated fluid
    pub fluid_configs: FluidConfigs,
    /// general configurations for the simulation itself
    pub simulation_configs: SimulationConfigs,
    s: Vec<f32>,
    /// the distributed densities for the given step
    pub density: Vec<f32>,
    /// the distributed velocities in the x direction for the given step.
    /// **Note:** The velocity is a vector formed by both the `velocities_x` and `velocities_y`
    /// vector structures
    pub velocities_x: Vec<f32>,
    /// the distributed velocities in the y direction for the given step.
    /// **Note:** The velocity is a vector formed by both the `velocities_x` and `velocities_y`
    /// vector structures
    pub velocities_y: Vec<f32>,
    velocities_x0: Vec<f32>,
    velocities_y0: Vec<f32>,
    /// Defines which cells are "allowed" for the fluid to run into and which are "obsticles"
    /// by also defining which side of a given obstacle a cell is via the [`ContainerWall`]
    pub allowed_cells: Vec<ContainerWall>,
}

impl Fluid {
    /// Creates new Fluid struct
    pub fn new(init_fluid: FluidConfigs, init_simulation: SimulationConfigs) -> Self {
        let fluid_field_size = (init_simulation.size * init_simulation.size) as usize;

        Fluid {
            s: vec![0.0; fluid_field_size],
            density: vec![0.0; fluid_field_size],
            velocities_x: vec![0.0; fluid_field_size],
            velocities_y: vec![0.0; fluid_field_size],
            velocities_x0: vec![0.0; fluid_field_size],
            velocities_y0: vec![0.0; fluid_field_size],
            allowed_cells: vec![ContainerWall::NoWall; fluid_field_size],
            fluid_configs: init_fluid,
            simulation_configs: init_simulation,
        }
    }

    fn add_density(&mut self, x: u32, y: u32, amount: f32) {
        let idx = idx!(x, y, self.simulation_configs.size);
        self.density[idx] += amount;
        self.s[idx] += amount;
    }

    fn add_velocity(&mut self, x: u32, y: u32, amount_x: f32, amount_y: f32) {
        let idx = idx!(x, y, self.simulation_configs.size);
        self.velocities_x[idx] += amount_x;
        self.velocities_y[idx] += amount_y;
    }

    // TODO: express side as Ox or Oy

    /// Reflects a cell's velocity when a wall is hit. It works as follows:
    ///
    /// Since the velocity vector (its direction and magnitute) are given by
    /// the sum of the velocities_x and velocities_y coefficient, the hit to a wall
    /// is simulated by mirroring those vectors. This happens just by changing the sign of
    /// the given vector.
    ///
    /// *Note 1.:* depending on which edge the cell is next to, only the x velocity component OR
    /// the y velocity is changed. Also note that this only works if the wall is parallel to
    /// one of the axis of the coordinate system.
    ///
    /// ============== Working principle =================
    /// A cell's velocity vector is a combination of the `velocities_x` and `velocities_y`
    ///
    ///              ^
    /// velocities_y |^
    ///              | \  sum of the two vectors forms a new vector: the direction of the
    ///              |  \ fluid in that exact cell
    ///              ---->
    ///            velocities_x
    ///
    /// Mirroring of the vector with regards to Oy is made by replacing the x component of the
    /// vector with its opposite number (the same value, but with the opposite sign)
    ///
    ///                      ^
    /// new vector which    ^|^
    /// mirrors the        / | \  sum of the two vectors forms a new vector: the direction of the
    /// original          /  |  \ fluid in that exact cell
    ///                 <=====---->
    ///                 velocities_x
    ///
    /// *Note 2.:* The above example is appropriate for the left wall:
    ///  __________
    ///  ||        |
    ///  ||  ^     |
    ///  || /      |
    ///  ||/       |
    ///  ||^       |
    ///  || \      |
    ///  ||  \     |
    ///  -----------
    ///
    /// *Note 3.:* The corners do this mirroring for both their x and y component which results in
    /// a vector symmetrical by both Ox and Oy.
    ///
    ///                      ^
    ///   mirrored vector   ^|^   (1)
    ///   by Oy (2)        / | \  sum of the two vectors forms a new vector: the direction of the
    ///                   /  |  \ fluid in that exact cell
    ///                 <=====---->
    ///                  \   |
    ///  the resulting    \  |
    ///  arrow which is    ^ |
    ///  pointing the oppisite
    ///  direction of (1)
    ///  (ignore downward arrow's tip :))
    ///
    fn set_boundaries(edge_wall: ContainerWall, x: &mut [f32], size: u32) {
        for i in 1..size - 1 {
            x[idx!(i, 0, size)] = if edge_wall == ContainerWall::East {
                -x[idx!(i, 1, size)]
            } else {
                x[idx!(i, 1, size)]
            };
            x[idx!(i, size - 1, size)] = if edge_wall == ContainerWall::East {
                -x[idx!(i, size - 2, size)]
            } else {
                x[idx!(i, size - 2, size)]
            };
        }

        for j in 1..size - 1 {
            x[idx!(0, j, size)] = if edge_wall == ContainerWall::North {
                -x[idx!(1, j, size)]
            } else {
                x[idx!(1, j, size)]
            };
            x[idx!(size - 1, j, size)] = if edge_wall == ContainerWall::North {
                -x[idx!(size - 2, j, size)]
            } else {
                x[idx!(size - 2, j, size)]
            };
        }

        x[idx!(0, 0, size)] = 0.5 * (x[idx!(1, 0, size)] + x[idx!(0, 1, size)]);
        x[idx!(0, size - 1, size)] =
            0.5 * (x[idx!(1, size - 1, size)] + x[idx!(0, size - 2, size)]);
        x[idx!(size - 1, 0, size)] =
            0.5 * (x[idx!(size - 2, 0, size)] + x[idx!(size - 1, 1, size)]);
        x[idx!(size - 1, size - 1, size)] =
            0.5 * (x[idx!(size - 2, size - 1, size)] + x[idx!(size - 1, size - 2, size)]);
    }

    fn diffuse(
        edge_wall: ContainerWall,
        x: &mut [f32],
        x0: &[f32],
        diffusion: &f32,
        size: u32,
        delta_t: &f32,
        frames: i64,
    ) {
        let size_float = (size - 2) as f32;
        let a = delta_t * diffusion * size_float * size_float;
        Fluid::lin_solve(edge_wall, x, x0, a, 1.0 + 4.0 * a, size, frames);
    }

    fn lin_solve(
        edge_wall: ContainerWall,
        x: &mut [f32],
        x0: &[f32],
        a: f32,
        c: f32,
        size: u32,
        frames: i64,
    ) {
        let c_recip = 1.0 / c;
        for _k in 0..frames {
            for j in 1..size - 1 {
                for i in 1..size - 1 {
                    x[idx!(i, j, size)] = (x0[idx!(i, j, size)]
                        + a * (x[idx!(i + 1, j, size)]
                            + x[idx!(i - 1, j, size)]
                            + x[idx!(i, j + 1, size)]
                            + x[idx!(i, j - 1, size)]))
                        * c_recip;
                }
            }
            Fluid::set_boundaries(edge_wall, x, size);
        }
    }

    fn project(
        velocities_x: &mut [f32],
        velocities_y: &mut [f32],
        p: &mut [f32],
        div: &mut [f32],
        size: u32,
        frames: i64,
    ) {
        for j in 1..size - 1 {
            for i in 1..size - 1 {
                div[idx!(i, j, size)] = -0.5
                    * (velocities_x[idx!(i + 1, j, size)] - velocities_x[idx!(i - 1, j, size)]
                        + velocities_y[idx!(i, j + 1, size)]
                        - velocities_y[idx!(i, j - 1, size)])
                    / size as f32;

                p[idx!(i, j, size)] = 0.0;
            }
        }

        Fluid::set_boundaries(ContainerWall::NoWall, div, size);
        Fluid::set_boundaries(ContainerWall::NoWall, p, size);
        Fluid::lin_solve(ContainerWall::NoWall, p, div, 1f32, 4f32, size, frames);

        for j in 1..size - 1 {
            for i in 1..size - 1 {
                velocities_x[idx!(i, j, size)] -=
                    0.5 * (p[idx!(i + 1, j, size)] - p[idx!(i - 1, j, size)]) * size as f32;
                velocities_y[idx!(i, j, size)] -=
                    0.5 * (p[idx!(i, j + 1, size)] - p[idx!(i, j - 1, size)]) * size as f32;
            }
        }

        // TODO: sucspicious: add South & North
        Fluid::set_boundaries(ContainerWall::West, velocities_x, size);
        Fluid::set_boundaries(ContainerWall::South, velocities_x, size);
        Fluid::set_boundaries(ContainerWall::East, velocities_x, size);
        Fluid::set_boundaries(ContainerWall::North, velocities_y, size);
    }

    fn advect(
        edge_wall: ContainerWall,
        densities: &mut [f32],
        densities0: &[f32],
        velocities_x: &[f32],
        velocities_y: &[f32],
        size: u32,
        delta_t: &f32,
    ) {
        let (mut i0, mut i1, mut j0, mut j1): (f32, f32, f32, f32);

        let delta_t_x = *delta_t * (size - 2) as f32;
        let delta_t_y = delta_t_x;

        let (mut s0, mut s1, mut t0, mut t1): (f32, f32, f32, f32);
        let (mut x, mut y): (f32, f32);

        let size_float: f32 = size as f32;

        for j in 1..size - 1 {
            for i in 1..size - 1 {
                x = i as f32 - delta_t_x * velocities_x[idx!(i, j, size)];
                y = j as f32 - delta_t_y * velocities_y[idx!(i, j, size)];

                x = x.clamp(0.5, size_float - 1.0);
                y = y.clamp(0.5, size_float - 1.0);

                i0 = x.floor();
                i1 = i0 + 1.0;

                j0 = y.floor();
                j1 = j0 + 1.0;

                s1 = x - i0;
                s0 = 1.0 - s1;
                t1 = y - j0;
                t0 = 1.0 - t1;

                let (i0_int, i1_int) = (i0 as u32, i1 as u32);
                let (j0_int, j1_int) = (j0 as u32, j1 as u32);

                if i1 >= size_float || j1 >= size_float {
                    densities[idx!(i, j, size)] = densities[idx!(i - 1, j, size)];
                    break;
                }
                densities[idx!(i, j, size)] = s0
                    * (t0 * densities0[idx!(i0_int, j0_int, size)]
                        + t1 * densities0[idx!(i0_int, j1_int, size)])
                    + s1 * (t0 * densities0[idx!(i1_int, j0_int, size)]
                        + t1 * densities0[idx!(i1_int, j1_int, size)]);
            }
        }
        Fluid::set_boundaries(edge_wall, densities, size);
    }

    /// Simulates the next step of the fluid's movement.
    /// That includes applying diffusion and advection to the fluid
    /// and constraining it to not get out of the wall's boundaries
    pub fn step(&mut self) {
        Fluid::diffuse(
            ContainerWall::East,
            &mut self.velocities_x0,
            &self.velocities_x,
            &self.fluid_configs.viscousity,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.frames,
        );
        Fluid::diffuse(
            ContainerWall::North,
            &mut self.velocities_y0,
            &self.velocities_y,
            &self.fluid_configs.viscousity,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.frames,
        );

        Fluid::project(
            &mut self.velocities_x0,
            &mut self.velocities_y0,
            &mut self.velocities_x,
            &mut self.velocities_y,
            self.simulation_configs.size,
            self.simulation_configs.frames,
        );

        Fluid::advect(
            ContainerWall::East,
            &mut self.velocities_x,
            &self.velocities_x0,
            &self.velocities_x0,
            &self.velocities_y0,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
        );
        Fluid::advect(
            ContainerWall::North,
            &mut self.velocities_y,
            &self.velocities_y0,
            &self.velocities_x0,
            &self.velocities_y0,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
        );

        Fluid::project(
            &mut self.velocities_x,
            &mut self.velocities_y,
            &mut self.velocities_x0,
            &mut self.velocities_y0,
            self.simulation_configs.size,
            self.simulation_configs.frames,
        );

        Fluid::diffuse(
            ContainerWall::NoWall,
            &mut self.s,
            &self.density,
            &self.fluid_configs.diffusion,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.frames,
        );

        Fluid::advect(
            ContainerWall::NoWall,
            &mut self.density,
            &self.s,
            &self.velocities_x,
            &self.velocities_y,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
        );

        self.s = self.density.clone();
    }

    fn init_density(&mut self) {
        for j in 0..self.simulation_configs.size {
            for i in 0..self.simulation_configs.size {
                self.add_density(i, j, 0.0);
            }
        }

        for j in self.simulation_configs.size / 2 - 10..=self.simulation_configs.size / 2 + 10 {
            for i in self.simulation_configs.size / 2 - 10..=self.simulation_configs.size / 2 + 10 {
                self.add_density(i, j, 0.9);
            }
        }
    }

    fn init_velocities(&mut self) {
        for j in 0..self.simulation_configs.size {
            for i in 0..self.simulation_configs.size {
                self.add_velocity(i, j, 1.0, 1.0);
            }
        }
    }

    /// Applies random force (noise) to the fluid to make the fluid run
    /// more attractively when there's no specific purpose yet of the
    /// simulation yet.
    pub fn add_noise(&mut self) {
        let perlin = Perlin::new();

        let angle = perlin.get([
            self.simulation_configs.delta_t as f64,
            self.simulation_configs.delta_t as f64,
        ]) * 6.28
            * 2f64;
        let rand_x = rand::thread_rng().gen_range(0..self.simulation_configs.size);
        let rand_y = rand::thread_rng().gen_range(0..self.simulation_configs.size);

        let center_point = point!(x: (self.simulation_configs.size/2) as f32, y: (self.simulation_configs.size/2) as f32);
        let _point_of_rotation = point!(x: rand_x as f32, y: rand_y as f32);

        let ls = line_string![(x: (self.simulation_configs.size/2) as f32, y: (self.simulation_configs.size/2) as f32), (x: rand_x as f32, y: rand_y as f32)];

        let rotated = ls.rotate_around_point(angle as f32, center_point);

        self.add_velocity(
            center_point.x().trunc() as u32,
            center_point.y().trunc() as u32,
            rotated[1].x as f32 * 2.0,
            rotated[1].y as f32 * 2.0,
        );
    }

    /// Initialized the fluid
    pub fn init(&mut self) {
        self.init_velocities();
        self.init_density();
    }

    #[cfg(debug_assertions)]
    fn print_area(&self, point: &line_drawing::Point<i64>) {
        let vertex_idx = idx!(point.0, point.1, i64::from(self.simulation_configs.size));
        let symbol = match self.allowed_cells[vertex_idx] {
            ContainerWall::West => "| W ",
            ContainerWall::NorthWest => "|NW ",
            ContainerWall::North => "| N ",
            ContainerWall::NorthEast => "|NE ",
            ContainerWall::East => "| E ",
            ContainerWall::SouthEast => "|SE ",
            ContainerWall::South => "| S ",
            ContainerWall::SouthWest => "|SW ",
            ContainerWall::DefaultWall => "| D ",
            ContainerWall::NoWall => "| X ",
        };
        print!("{}", symbol);
    }

    /// Fills the inner cells of the obstacles with [`ContainerWall::DefaultWall`]
    fn fill_obstacle(&mut self, obstacle: &dyn Obstacle) -> Vec<line_drawing::Point<i64>> {
        let result: Vec<line_drawing::Point<i64>> = Vec::new();
        let obstacle_perimeter = obstacle.get_perimeter();

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
                        let idx = idx!(
                            x_coordinate,
                            east_point.1,
                            i64::from(self.simulation_configs.size)
                        );
                        self.allowed_cells[idx] = ContainerWall::DefaultWall;

                        #[cfg(debug_assertions)]
                        {
                            self.print_area(&(x_coordinate, east_point.1));
                        }
                    }
                    #[cfg(debug_assertions)]
                    {
                        self.print_area(&east_neighbour_point.unwrap());
                        println!("");
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

        result
    }

    /// Given the points of a obstacle, tell the fluid to avoid the object's points by telling
    /// each point's [`ContainerWall`] side
    pub fn set_obstacle(&mut self, obstacle: &dyn Obstacle) {
        let obstacle_sides = obstacle.get_perimeter();
        for (side_key, points) in obstacle_sides {
            for point in points {
                self.allowed_cells
                    [idx!(point.0, point.1, i64::from(self.simulation_configs.size))] = *side_key;
            }
        }

        self.fill_obstacle(obstacle);
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
    use crate::simulation::fluid::{ContainerWall, Fluid};
    use crate::simulation::obstacle::Rectangle;

    #[test]
    fn set_obstacle_works() {
        let fluid_configs = FluidConfigs::default();
        let simulation_configs = SimulationConfigs::default();

        let mut fluid = Fluid::new(fluid_configs, simulation_configs);
        fluid.set_obstacle(&Rectangle::new(
            (100, 100),
            (120, 80),
            fluid.simulation_configs.size,
        ));

        let mut count: u64 = 0;
        for cell in fluid.allowed_cells {
            if cell != ContainerWall::NoWall && cell != ContainerWall::DefaultWall {
                count += 1;
            }
        }

        assert_eq!(count, (100 - 80) * 2 + (120 - 100) * 2);
    }

    #[test]
    fn fill_obstacle_works() {
        let fluid_configs = FluidConfigs::default();
        let simulation_configs = SimulationConfigs::default();

        let mut fluid = Fluid::new(fluid_configs, simulation_configs);
        fluid.set_obstacle(&Rectangle::new(
            (100, 100),
            (120, 80),
            fluid.simulation_configs.size,
        ));

        let mut count: u64 = 0;
        for cell in fluid.allowed_cells.iter() {
            if cell == &ContainerWall::DefaultWall {
                count += 1;
            }
        }

        #[cfg(debug_assertions)]
        {
            let mut count_s: u64 = 0;
            for cell in fluid.allowed_cells.iter() {
                if cell == &ContainerWall::South {
                    count_s += 1;
                }
            }
            println!("South count: {}", count_s);

            let mut count_n: u64 = 0;
            for cell in fluid.allowed_cells.iter() {
                if cell == &ContainerWall::North {
                    count_n += 1;
                }
            }
            println!("North count: {}", count_n);

            let mut count_e: u64 = 0;
            for cell in fluid.allowed_cells.iter() {
                if cell == &ContainerWall::East {
                    count_e += 1;
                }
            }
            println!("East count: {}", count_e);

            let mut count_w: u64 = 0;
            for cell in fluid.allowed_cells.iter() {
                if cell == &ContainerWall::West {
                    count_w += 1;
                }
            }
            println!("West count: {}", count_w);
        }

        let vertical_wall_len = 100 - 80 + 1;
        let horizontal_wall_len = 120 - 100 + 1;
        assert_eq!(count, (horizontal_wall_len - 2) * (vertical_wall_len - 2));
    }
}
