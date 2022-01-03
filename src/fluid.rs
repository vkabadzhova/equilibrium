use geo::algorithm::rotate::RotatePoint;
use geo::{line_string, point};
use noise::{NoiseFn, Perlin};
use rand::Rng;

#[derive(Copy, Clone)]
pub struct SimulationConfig {
    pub delta_t: f32,
    pub iterations: u32,
    pub t: f32,
    pub size: u32,
}

impl Default for SimulationConfig {
    fn default() -> SimulationConfig {
        SimulationConfig {
            // The size of each step
            delta_t: 0.02,
            // Number of iterations
            iterations: 16,
            // Current step
            t: 0.0,
            // The size of the fluid. A square container is used
            size: 128,
        }
    }
}

#[derive(Copy, Clone)]
pub struct FluidConfig {
    pub diffusion: f32,
    pub viscousity: f32,
}

impl Default for FluidConfig {
    fn default() -> FluidConfig {
        FluidConfig {
            diffusion: 0.0,
            viscousity: 0.0000001,
        }
    }
}

/// Address the direction in world-directions style
/// similliar to: mid-point line generation algorithm, etc.
#[derive(PartialEq, Eq, Copy, Clone)]
enum ContainerWall {
    West,
    North,
    East,
    South,
    DefaultWall,
}

macro_rules! idx {
    ($x:expr, $y:expr, $size: expr) => {
        ($x.clamp(0, $size - 1) + ($y.clamp(0, $size - 1) * $size)) as usize
    };
}

pub struct Fluid {
    fluid_configs: FluidConfig,
    simulation_configs: SimulationConfig,
    s: Vec<f32>,
    density: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    velocities_x0: Vec<f32>,
    velocities_y0: Vec<f32>,
}

impl Fluid {
    pub fn new(init_fluid: FluidConfig, init_simulation: SimulationConfig) -> Fluid {
        let fluid_field_size = (init_simulation.size * init_simulation.size) as usize;

        Fluid {
            s: vec![0.0; fluid_field_size],
            density: vec![0.0; fluid_field_size],
            velocities_x: vec![0.0; fluid_field_size],
            velocities_y: vec![0.0; fluid_field_size],
            velocities_x0: vec![0.0; fluid_field_size],
            velocities_y0: vec![0.0; fluid_field_size],
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

    fn render_density(&self, imgbuf: &mut image::RgbImage, frame_number: u32) {
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = self.density[idx!(x, y, self.simulation_configs.size)];
            *pixel = image::Rgb([(density * 255.0) as u8, 200, density as u8]);
        }
        let img_name = format!("rendered_images/density{}.jpg", frame_number);
        imgbuf.save(img_name).unwrap();
    }

    /// Process the edge walls, a.k.a. turn the velocities in the opposite way
    /// The function is used not only for velocities, but for density, etc, where
    /// A turn would have no meaning at all or will cause logical errors.
    /// Use ContainerWall::DefaultWall if no turn is needed
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
        iterations: u32,
    ) {
        let size_float = (size - 2) as f32;
        let a = delta_t * diffusion * size_float * size_float;
        Fluid::lin_solve(edge_wall, x, x0, a, 1.0 + 4.0 * a, size, iterations);
    }

    fn lin_solve(
        edge_wall: ContainerWall,
        x: &mut [f32],
        x0: &[f32],
        a: f32,
        c: f32,
        size: u32,
        iterations: u32,
    ) {
        let c_recip = 1f32 / c;
        for _k in 0..iterations {
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
        iterations: u32,
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

        Fluid::set_boundaries(ContainerWall::DefaultWall, div, size);
        Fluid::set_boundaries(ContainerWall::DefaultWall, p, size);
        Fluid::lin_solve(
            ContainerWall::DefaultWall,
            p,
            div,
            1f32,
            4f32,
            size,
            iterations,
        );

        for j in 1..size - 1 {
            for i in 1..size - 1 {
                velocities_x[idx!(i, j, size)] -=
                    0.5 * (p[idx!(i + 1, j, size)] - p[idx!(i - 1, j, size)]) * size as f32;
                velocities_y[idx!(i, j, size)] -=
                    0.5 * (p[idx!(i, j + 1, size)] - p[idx!(i, j - 1, size)]) * size as f32;
            }
        }

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

    fn step(&mut self) {
        Fluid::diffuse(
            ContainerWall::East,
            &mut self.velocities_x0,
            &self.velocities_x,
            &self.fluid_configs.viscousity,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.iterations,
        );
        Fluid::diffuse(
            ContainerWall::North,
            &mut self.velocities_y0,
            &self.velocities_y,
            &self.fluid_configs.viscousity,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.iterations,
        );

        Fluid::project(
            &mut self.velocities_x0,
            &mut self.velocities_y0,
            &mut self.velocities_x,
            &mut self.velocities_y,
            self.simulation_configs.size,
            self.simulation_configs.iterations,
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
            self.simulation_configs.iterations,
        );

        Fluid::diffuse(
            ContainerWall::DefaultWall,
            &mut self.s,
            &self.density,
            &self.fluid_configs.diffusion,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
            self.simulation_configs.iterations,
        );

        Fluid::advect(
            ContainerWall::DefaultWall,
            &mut self.density,
            &self.s,
            &self.velocities_x,
            &self.velocities_y,
            self.simulation_configs.size,
            &self.simulation_configs.delta_t,
        );

        self.s = self.density.clone();
    }

    fn init_densitity(&mut self) {
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

    fn add_noise(&mut self) {
        let perlin = Perlin::new();
        let t_f64 = self.simulation_configs.t as f64;

        let angle = perlin.get([t_f64, t_f64]) * 6.28 * 2f64;
        let rand_x = rand::thread_rng().gen_range(0..self.simulation_configs.size);
        let rand_y = rand::thread_rng().gen_range(0..self.simulation_configs.size);

        let (center_point, rotating_point) = (
            point!(x: (self.simulation_configs.size/2) as f32, y: (self.simulation_configs.size/2) as f32),
            point!(x: rand_x as f32, y: rand_y as f32),
        );

        let ls = line_string![(x: (self.simulation_configs.size/2) as f32, y: (self.simulation_configs.size/2) as f32), (x: rand_x as f32, y: rand_y as f32)];

        let rotated = ls.rotate_around_point(angle as f32, center_point);

        self.simulation_configs.t += 0.01;

        self.add_velocity(
            center_point.x().trunc() as u32,
            center_point.y().trunc() as u32,
            rotated[1].x as f32 * 2.0,
            rotated[1].y as f32 * 2.0,
        );
    }

    pub fn simulate(&mut self) {
        //Set up
        self.init_velocities();
        self.init_densitity();

        // draw
        for i in 0..self.simulation_configs.iterations {
            self.add_noise();
            self.step();

            let mut imgbuf =
                image::ImageBuffer::new(self.simulation_configs.size, self.simulation_configs.size);
            self.render_density(&mut imgbuf, i);
        }
    }
}
