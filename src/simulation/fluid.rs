use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::obstacle::Obstacle;
use geo::algorithm::rotate::RotatePoint;
use geo::{line_string, point};
use noise::{NoiseFn, Perlin};
use rand::Rng;

use super::obstacle::ObstaclesType;

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

/// The enum is used is created since some functions such as the [`Fluid::set_boundary()`] only
/// need a concise way to express if the line is parallel to Ox or Oy.
#[derive(PartialEq, Eq, Copy, Clone)]
enum Orientation {
    /// The Ox axis and semantically is used as all parallel to it
    AdjustRow,
    /// The Oy axis and semantically is used as all parallel to it
    AdjustColumn,
    /// No axis, the function is operating on a non vector-based concept
    AdjustPassive,
}

macro_rules! idx {
    ($x:expr, $y:expr, $size: expr) => {
        ($x.clamp(0, $size - 1) + ($y.clamp(0, $size - 1) * $size)) as usize
    };
}

/// The struct that is responsible for simulating the fluid's behavour.
///
/// *Note:*
/// The velocity vector for each cell (i.e. its direction and magnitute) is
/// given by the sum of the vector in the velocities_x and velocities_y coefficients
/// that form a part of normalized vectors, too.
///
///
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
    /// We need scratch space for each array so that we can keep old values around while we
    /// compute the new ones
    scratch_space: Vec<f32>,
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
    pub cells_type: Vec<ContainerWall>,
}

impl Default for Fluid {
    fn default() -> Self {
        let mut result = Self::new(FluidConfigs::default(), SimulationConfigs::default());
        result.init();
        result
    }
}

impl Fluid {
    /// Creates new Fluid struct
    pub fn new(init_fluid: FluidConfigs, init_simulation: SimulationConfigs) -> Self {
        let fluid_field_size = (init_simulation.size * init_simulation.size) as usize;

        let mut result = Self {
            scratch_space: vec![0.0; fluid_field_size],
            density: vec![0.0; fluid_field_size],
            velocities_x: vec![0.0; fluid_field_size],
            velocities_y: vec![0.0; fluid_field_size],
            velocities_x0: vec![0.0; fluid_field_size],
            velocities_y0: vec![0.0; fluid_field_size],
            cells_type: vec![ContainerWall::NoWall; fluid_field_size],
            fluid_configs: init_fluid,
            simulation_configs: init_simulation,
        };

        result.init();
        result
    }

    fn add_density(&mut self, x: u32, y: u32, amount: f32) {
        let idx = idx!(x, y, self.simulation_configs.size);
        self.density[idx] += amount;
        self.scratch_space[idx] += amount;
    }

    fn add_velocity(&mut self, x: u32, y: u32, amount_x: f32, amount_y: f32) {
        let idx = idx!(x, y, self.simulation_configs.size);
        self.velocities_x[idx] += amount_x;
        self.velocities_y[idx] += amount_y;
    }

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
    /// velocities_y |^
    ///              | \  sum of the two vectors forms a new vector: the direction of the
    ///              |  \ fluid in that exact cell
    ///              ---->
    ///            velocities_x
    ///
    /// Mirroring of the vector with regards to Oy is made by replacing the x component of the
    /// vector with its opposite number (the same value, but with the opposite sign)
    ///
    /// new vector which    ^|^
    /// mirrors the        / | \  sum of the two vectors forms a new vector: the direction of the
    /// original          /  |  \ fluid in that exact cell
    ///                 <=====---->
    ///                 velocities_x
    ///
    /// *Note 2.:* The above example is appropriate for the left wall:
    ///  __________
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
    fn set_boundaries(
        orientation: Orientation,
        x: &mut [f32],
        size: u32,
        cells_type: &Vec<ContainerWall>,
    ) {
        let size = i64::from(size);
        for j in 0..=size - 1 {
            for i in 0..=size - 1 {
                if cells_type[idx!(i, j, size)] == ContainerWall::DefaultWall {
                    continue;
                }

                let up_point_coordinates = (i, (j - 1).clamp(0, size - 1));
                let down_point_coordinates = (i, (j + 1).clamp(0, size - 1));
                let left_point_coordinates = ((i - 1).clamp(0, size - 1), j);
                let right_point_coordinates = ((i + 1).clamp(0, size - 1), j);

                match orientation {
                    Orientation::AdjustRow => {
                        if cells_type
                            [idx!(left_point_coordinates.0, left_point_coordinates.1, size)]
                            == ContainerWall::DefaultWall
                        {
                            x[idx!(i, j, size)] =
                                -x[idx!(left_point_coordinates.0, left_point_coordinates.1, size)]
                        }
                        if cells_type
                            [idx!(right_point_coordinates.0, right_point_coordinates.1, size)]
                            == ContainerWall::DefaultWall
                        {
                            x[idx!(i, j, size)] =
                                -x[idx!(right_point_coordinates.0, right_point_coordinates.1, size)]
                        }
                    }
                    Orientation::AdjustColumn => {
                        if cells_type
                            [idx!(down_point_coordinates.0, down_point_coordinates.1, size)]
                            == ContainerWall::DefaultWall
                        {
                            x[idx!(i, j, size)] =
                                -x[idx!(down_point_coordinates.0, down_point_coordinates.1, size)]
                        }
                        if cells_type[idx!(up_point_coordinates.0, up_point_coordinates.1, size)]
                            == ContainerWall::DefaultWall
                        {
                            x[idx!(i, j, size)] =
                                -x[idx!(up_point_coordinates.0, up_point_coordinates.1, size)]
                        }
                    }
                    Orientation::AdjustPassive => {
                        // NB: works only for the edges

                        x[idx!(i, 0, size)] = x[idx!(i, 1, size)];
                        x[idx!(i, size - 1, size)] = x[idx!(i, size - 2, size)];

                        x[idx!(0, j, size)] = x[idx!(1, j, size)];
                        x[idx!(size - 1, j, size)] = x[idx!(size - 2, j, size)];
                    }
                }
            }
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
        orientation: Orientation,
        x: &mut [f32],
        x0: &[f32],
        diffusion: &f32,
        size: u32,
        delta_t: &f32,
        frames: i64,
        cells_type: &Vec<ContainerWall>,
    ) {
        let size_float = (size - 2) as f32;
        let a = delta_t * diffusion * size_float * size_float;
        Fluid::lin_solve(
            orientation,
            x,
            x0,
            a,
            1.0 + 4.0 * a,
            size,
            frames,
            cells_type,
        );
    }

    fn lin_solve(
        orientation: Orientation,
        x: &mut [f32],
        x0: &[f32],
        a: f32,
        c: f32,
        size: u32,
        frames: i64,
        cells_type: &Vec<ContainerWall>,
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
            Fluid::set_boundaries(orientation, x, size, cells_type);
        }
    }

    fn project(
        velocities_x: &mut [f32],
        velocities_y: &mut [f32],
        p: &mut [f32],
        div: &mut [f32],
        size: u32,
        frames: i64,
        cells_type: &Vec<ContainerWall>,
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

        Fluid::set_boundaries(Orientation::AdjustPassive, div, size, cells_type);
        Fluid::set_boundaries(Orientation::AdjustPassive, p, size, cells_type);
        Fluid::lin_solve(
            Orientation::AdjustPassive,
            p,
            div,
            1.0,
            4.0,
            size,
            frames,
            cells_type,
        );

        for j in 1..size - 1 {
            for i in 1..size - 1 {
                velocities_x[idx!(i, j, size)] -=
                    0.5 * (p[idx!(i + 1, j, size)] - p[idx!(i - 1, j, size)]) * size as f32;
                velocities_y[idx!(i, j, size)] -=
                    0.5 * (p[idx!(i, j + 1, size)] - p[idx!(i, j - 1, size)]) * size as f32;
            }
        }

        Fluid::set_boundaries(Orientation::AdjustRow, velocities_x, size, cells_type);
        Fluid::set_boundaries(Orientation::AdjustColumn, velocities_y, size, cells_type);
    }

    fn advect(
        orientation: Orientation,
        densities: &mut [f32],
        densities0: &[f32],
        velocities_x: &[f32],
        velocities_y: &[f32],
        size: u32,
        delta_t: &f32,
        cells_type: &Vec<ContainerWall>,
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
        Fluid::set_boundaries(orientation, densities, size, cells_type);
    }

    /// Simulates the next step of the fluid's movement.
    /// That includes applying diffusion and advection to the fluid
    /// and constraining it to not get out of the wall's boundaries
    pub fn step(&mut self) {
        Fluid::diffuse(
            Orientation::AdjustRow,
            &mut self.velocities_x0,
            &self.velocities_x,
            &self.fluid_configs.viscousity,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.frames,
            &self.cells_type,
        );
        Fluid::diffuse(
            Orientation::AdjustColumn,
            &mut self.velocities_y0,
            &self.velocities_y,
            &self.fluid_configs.viscousity,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.frames,
            &self.cells_type,
        );

        Fluid::project(
            &mut self.velocities_x0,
            &mut self.velocities_y0,
            &mut self.velocities_x,
            &mut self.velocities_y,
            self.simulation_configs.size,
            self.simulation_configs.frames,
            &self.cells_type,
        );

        Fluid::advect(
            Orientation::AdjustRow,
            &mut self.velocities_x,
            &self.velocities_x0,
            &self.velocities_x0,
            &self.velocities_y0,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            &self.cells_type,
        );

        Fluid::advect(
            Orientation::AdjustColumn,
            &mut self.velocities_y,
            &self.velocities_y0,
            &self.velocities_x0,
            &self.velocities_y0,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            &self.cells_type,
        );

        Fluid::project(
            &mut self.velocities_x,
            &mut self.velocities_y,
            &mut self.velocities_x0,
            &mut self.velocities_y0,
            self.simulation_configs.size,
            self.simulation_configs.frames,
            &self.cells_type,
        );

        Fluid::diffuse(
            Orientation::AdjustPassive,
            &mut self.scratch_space,
            &self.density,
            &self.fluid_configs.diffusion,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.frames,
            &self.cells_type,
        );

        Fluid::advect(
            Orientation::AdjustPassive,
            &mut self.density,
            &self.scratch_space,
            &self.velocities_x,
            &self.velocities_y,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            &self.cells_type,
        );

        self.scratch_space = self.density.clone();
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

    fn init_cells_type(&mut self) {
        for i in 0..self.simulation_configs.size {
            self.cells_type[idx!(i, 0, self.simulation_configs.size)] = ContainerWall::DefaultWall;
            self.cells_type[idx!(
                i,
                self.simulation_configs.size - 1,
                self.simulation_configs.size
            )] = ContainerWall::DefaultWall;
        }

        for j in 0..self.simulation_configs.size {
            self.cells_type[idx!(0, j, self.simulation_configs.size)] = ContainerWall::DefaultWall;
            self.cells_type[idx!(
                self.simulation_configs.size - 1,
                j,
                self.simulation_configs.size
            )] = ContainerWall::DefaultWall;
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

    /// Initialize the fluid. Every fluid should be initialized prior any manipulation.
    fn init(&mut self) {
        self.init_velocities();
        self.init_density();
        self.init_cells_type();
    }

    /// Fills the inner cells of the obstacles with [`ContainerWall::DefaultWall`]
    /// NB: works as approximation to the real obstacle. By approximating a rectangle.
    pub fn fill_obstacle(&mut self, obstacle: &mut ObstaclesType) {
        let points = obstacle.get_approximate_points();

        for x in points[0].0..points[1].0 {
            for y in points[0].1..points[1].1 {
                self.cells_type[idx!(x, y, i64::from(self.simulation_configs.size))] =
                    ContainerWall::DefaultWall;
            }
        }
    }
}

#[cfg(test)]
mod tests {}
