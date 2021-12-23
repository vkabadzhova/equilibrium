extern crate image;
extern crate rand;

use geo::algorithm::rotate::RotatePoint;
use geo::{line_string, point};
use noise::{NoiseFn, Perlin};
use rand::Rng;

//#[derive(Copy, Clone)]
pub struct SimulationConfig {
    pub dt: f32,
    pub iter: u32,
    pub scale: u32,
    pub t: f32,
    pub size: u32,
}

impl Default for SimulationConfig {
    fn default() -> SimulationConfig {
        SimulationConfig {
            dt: 0.02,
            iter: 16,
            scale: 4,
            t: 0f32,
            size: 128,
        }
    }
}

//#[derive(Copy, Clone)]
pub struct FluidConfig {
    pub diffusion: f32,
    pub viscousity: f32,
}

impl Default for FluidConfig {
    fn default() -> FluidConfig {
        FluidConfig {
            diffusion: 0f32,
            viscousity: 0.0000001,
        }
    }
}

//#[derive(Copy, Clone)]
pub struct Fluid {
    pub fluid_configs: FluidConfig,
    pub simulation_configs: SimulationConfig,
    s: Vec<f32>,
    density: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    velocities_x0: Vec<f32>,
    velocities_y0: Vec<f32>,
}

impl Fluid {
    pub fn new(init_fluid: FluidConfig, init_simulation: SimulationConfig) -> Fluid {
        Fluid {
            s: vec![0.0; (init_simulation.size * init_simulation.size) as usize],
            density: vec![0.0; (init_simulation.size * init_simulation.size) as usize],
            velocities_x: vec![0.0; (init_simulation.size * init_simulation.size) as usize],
            velocities_y: vec![0.0; (init_simulation.size * init_simulation.size) as usize],
            velocities_x0: vec![0.0; (init_simulation.size * init_simulation.size) as usize],
            velocities_y0: vec![0.0; (init_simulation.size * init_simulation.size) as usize],
            fluid_configs: init_fluid,
            simulation_configs: init_simulation,
        }
    }

<<<<<<< HEAD
    /// Returns the variable's value constrained between from and to (both inclusive)
    /// # Arguments
    /// * `var` - The variable whoose value to be processed
    /// * `from` - the minimal value that can be hold (inclusive)
    /// * `to` - the maximum value that can be hold (inclusive)
    fn constrain<T: PartialOrd>(var: T, from: T, to: T) -> T {
        match var {
            d if d < from => from,
            d if d > to => to,
            _ => var,
        }
=======
    pub fn IX(x: u32, y: u32) -> usize {
        let new_x = match x {
            // TODO: Refactor
            d if d <= N - 1 => x,
            _ => N - 1,
        };
        let new_y = match y {
            // TODO: Refactor
            d if d <= N - 1 => y,
            _ => N - 1,
        };

        // safe because will be index in array
        (new_x + (new_y * N)) as usize
>>>>>>> Fix 2d to 1d translation (IX)
    }

    fn coordinate_to_idx(x: u32, y: u32, size: &u32) -> usize {
        let x = &Fluid::constrain(x, 0, size - 1);
        let y = &Fluid::constrain(y, 0, size - 1);
        (x + (y * size)) as usize
    }

    fn add_density(&mut self, x: u32, y: u32, amount: f32) {
        let idx: usize = Fluid::coordinate_to_idx(x, y, &self.simulation_configs.size);
        self.density[idx] += amount;
        self.s[idx] += amount;
    }

    fn add_velocity(&mut self, x: u32, y: u32, amount_x: f32, amount_y: f32) {
        let idx: usize = Fluid::coordinate_to_idx(x, y, &self.simulation_configs.size);
        self.velocities_x[idx] += amount_x;
        self.velocities_y[idx] += amount_y;
    }

    fn render_density(&self, imgbuf: &mut image::RgbImage, frame_number: u32) {
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
<<<<<<< HEAD
            let density =
                self.density[Fluid::coordinate_to_idx(x, y, &self.simulation_configs.size)];
=======
            let density = self.density[Fluid::IX(x, y)];
            if (density != 0.0) {
                dbg!(x, y, density);
            }
>>>>>>> Fix 2d to 1d translation (IX)
            *pixel = image::Rgb([(density * 255.0) as u8, 200, density as u8]);
        }
        let img_name = format!("rendered_images/density{}.jpg", frame_number);
        imgbuf.save(img_name).unwrap();
    }

    fn set_boundaries(b: u32, x: &mut [f32], size: &u32) {
        for i in 1..size - 1 {
            x[Fluid::coordinate_to_idx(i, 0, size)] = if b == 2 {
                -x[Fluid::coordinate_to_idx(i, 1, size)]
            } else {
                x[Fluid::coordinate_to_idx(i, 1, size)]
            };
            x[Fluid::coordinate_to_idx(i, size - 1, size)] = if b == 2 {
                -x[Fluid::coordinate_to_idx(i, size - 2, size)]
            } else {
                x[Fluid::coordinate_to_idx(i, size - 2, size)]
            };
        }

        for j in 1..size - 1 {
            x[Fluid::coordinate_to_idx(0, j, size)] = if b == 1 {
                -x[Fluid::coordinate_to_idx(1, j, size)]
            } else {
                x[Fluid::coordinate_to_idx(1, j, size)]
            };
            x[Fluid::coordinate_to_idx(size - 1, j, size)] = if b == 1 {
                -x[Fluid::coordinate_to_idx(size - 2, j, size)]
            } else {
                x[Fluid::coordinate_to_idx(size - 2, j, size)]
            };
        }

        x[Fluid::coordinate_to_idx(0, 0, size)] = 0.5
            * (x[Fluid::coordinate_to_idx(1, 0, size)] + x[Fluid::coordinate_to_idx(0, 1, size)]);
        x[Fluid::coordinate_to_idx(0, size - 1, size)] = 0.5
            * (x[Fluid::coordinate_to_idx(1, *size - 1, size)]
                + x[Fluid::coordinate_to_idx(0, *size - 2, size)]);
        x[Fluid::coordinate_to_idx(size - 1, 0, size)] = 0.5
            * (x[Fluid::coordinate_to_idx(size - 2, 0, size)]
                + x[Fluid::coordinate_to_idx(size - 1, 1, size)]);
        x[Fluid::coordinate_to_idx(size - 1, size - 1, size)] = 0.5
            * (x[Fluid::coordinate_to_idx(size - 2, size - 1, size)]
                + x[Fluid::coordinate_to_idx(size - 1, size - 2, size)]);
    }

    fn diffuse(
        b: u32,
        x: &mut [f32],
        x0: &[f32],
        diffusion: &f32,
        size: &u32,
        dt: &f32,
        iter: &u32,
    ) {
        let size_float: f32 = (size - 2) as f32;
        let a: f32 = dt * diffusion * size_float * size_float;
        Fluid::lin_solve(b, x, x0, a, 1.0 + 4.0 * a, size, iter);
    }

    fn lin_solve(b: u32, x: &mut [f32], x0: &[f32], a: f32, c: f32, size: &u32, iter: &u32) {
        let c_recip = 1f32 / c;
        for _k in 0..*iter {
            for j in 1..size - 1 {
                for i in 1..size - 1 {
                    x[Fluid::coordinate_to_idx(i, j, size)] = (x0
                        [Fluid::coordinate_to_idx(i, j, size)]
                        + a * (x[Fluid::coordinate_to_idx(i + 1, j, size)]
                            + x[Fluid::coordinate_to_idx(i - 1, j, size)]
                            + x[Fluid::coordinate_to_idx(i, j + 1, size)]
                            + x[Fluid::coordinate_to_idx(i, j - 1, size)]))
                        * c_recip;
                }
            }
            Fluid::set_boundaries(b, x, size);
        }
    }

    fn project(
        velocities_x: &mut [f32],
        velocities_y: &mut [f32],
        p: &mut [f32],
        div: &mut [f32],
        size: &u32,
        iter: &u32,
    ) {
        for j in 1..size - 1 {
            for i in 1..size - 1 {
                div[Fluid::coordinate_to_idx(i, j, size)] = -0.5
                    * (velocities_x[Fluid::coordinate_to_idx(i + 1, j, size)]
                        - velocities_x[Fluid::coordinate_to_idx(i - 1, j, size)]
                        + velocities_y[Fluid::coordinate_to_idx(i, j + 1, size)]
                        - velocities_y[Fluid::coordinate_to_idx(i, j - 1, size)])
                    / *size as f32;

                p[Fluid::coordinate_to_idx(i, j, size)] = 0f32;
            }
        }

        Fluid::set_boundaries(0, div, size);
        Fluid::set_boundaries(0, p, size);
        Fluid::lin_solve(0, p, div, 1f32, 4f32, size, iter);

        for j in 1..size - 1 {
            for i in 1..size - 1 {
                velocities_x[Fluid::coordinate_to_idx(i, j, size)] -= 0.5
                    * (p[Fluid::coordinate_to_idx(i + 1, j, size)]
                        - p[Fluid::coordinate_to_idx(i - 1, j, size)])
                    * *size as f32;
                velocities_y[Fluid::coordinate_to_idx(i, j, size)] -= 0.5
                    * (p[Fluid::coordinate_to_idx(i, j + 1, size)]
                        - p[Fluid::coordinate_to_idx(i, j - 1, size)])
                    * *size as f32;
            }
        }

        Fluid::set_boundaries(1, velocities_x, size);
        Fluid::set_boundaries(2, velocities_y, size);
    }

    fn advect(
        boundary: u32,
        densities: &mut [f32],
        densities0: &[f32],
        velocities_x: &[f32],
        velocities_y: &[f32],
        size: &u32,
        dt: &f32,
    ) {
        let (mut i0, mut i1, mut j0, mut j1): (f32, f32, f32, f32);

        let dtx: f32 = *dt * (*size as f32 - 2f32);
        let dty: f32 = dtx;

        let (mut s0, mut s1, mut t0, mut t1): (f32, f32, f32, f32);
        let (mut x, mut y): (f32, f32);

        let size_float: f32 = *size as f32;

        for j in 1..size - 1 {
            for i in 1..size - 1 {
                x = i as f32 - dtx * velocities_x[Fluid::coordinate_to_idx(i, j, size)];
                y = j as f32 - dty * velocities_y[Fluid::coordinate_to_idx(i, j, size)];

                x = Fluid::constrain(x, 0.5, size_float - 1.0);
                y = Fluid::constrain(y, 0.5, size_float - 1.0);

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
                    densities[Fluid::coordinate_to_idx(i, j, size)] =
                        densities[Fluid::coordinate_to_idx(i - 1, j, size)];
                    break;
                }
                densities[Fluid::coordinate_to_idx(i, j, size)] = s0
                    * (t0 * densities0[Fluid::coordinate_to_idx(i0_int, j0_int, size)]
                        + t1 * densities0[Fluid::coordinate_to_idx(i0_int, j1_int, size)])
                    + s1 * (t0 * densities0[Fluid::coordinate_to_idx(i1_int, j0_int, size)]
                        + t1 * densities0[Fluid::coordinate_to_idx(i1_int, j1_int, size)]);
            }
        }
        Fluid::set_boundaries(boundary, densities, size);
    }

    fn step(&mut self) {
        Fluid::diffuse(
            1,
            &mut self.velocities_x0,
            &self.velocities_x,
            &self.fluid_configs.viscousity,
            &self.simulation_configs.size,
            &self.simulation_configs.dt,
            &self.simulation_configs.iter,
        );
        Fluid::diffuse(
            2,
            &mut self.velocities_y0,
            &self.velocities_y,
            &self.fluid_configs.viscousity,
            &self.simulation_configs.size,
            &self.simulation_configs.dt,
            &self.simulation_configs.iter,
        );

        Fluid::project(
            &mut self.velocities_x0,
            &mut self.velocities_y0,
            &mut self.velocities_x,
            &mut self.velocities_y,
            &self.simulation_configs.size,
            &self.simulation_configs.iter,
        );

        Fluid::advect(
            1,
            &mut self.velocities_x,
            &self.velocities_x0,
            &self.velocities_x0,
            &self.velocities_y0,
            &self.simulation_configs.size,
            &self.simulation_configs.dt,
        );
        Fluid::advect(
            2,
            &mut self.velocities_y,
            &self.velocities_y0,
            &self.velocities_x0,
            &self.velocities_y0,
            &self.simulation_configs.size,
            &self.simulation_configs.dt,
        );

        Fluid::project(
            &mut self.velocities_x,
            &mut self.velocities_y,
            &mut self.velocities_x0,
            &mut self.velocities_y0,
            &self.simulation_configs.size,
            &self.simulation_configs.iter,
        );

        Fluid::diffuse(
            0,
            &mut self.s,
            &self.density,
            &self.fluid_configs.diffusion,
            &self.simulation_configs.size,
            &self.simulation_configs.dt,
            &self.simulation_configs.iter,
        );

        Fluid::advect(
            0,
            &mut self.density,
            &self.s,
            &self.velocities_x,
            &self.velocities_y,
            &self.simulation_configs.size,
            &self.simulation_configs.dt,
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

        // for j in 0..N {
        //     for i in 0..N {
        //         if (fluid.density)
        //     }
        // }

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
        let t_f64: f64 = self.simulation_configs.t as f64;

        let angle: f64 = perlin.get([t_f64, t_f64]) * 6.28 * 2f64;
        let rand_x = rand::thread_rng().gen_range(0..self.simulation_configs.size);
        let rand_y = rand::thread_rng().gen_range(0..self.simulation_configs.size);

        let (center_point, rotating_point) = (
            point!(x: (self.simulation_configs.size/2) as f32, y: (self.simulation_configs.size/2) as f32),
            point!(x: rand_x as f32, y: rand_y as f32),
        );

        let ls = line_string![(x: (self.simulation_configs.size/2) as f32, y: (self.simulation_configs.size/2) as f32), (x: rand_x as f32, y: rand_y as f32)];

        //let ls = line_string![center_point, rotating_point];

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
        for i in 0..self.simulation_configs.iter {
            self.add_noise();
            self.step();

            let mut imgbuf =
                image::ImageBuffer::new(self.simulation_configs.size, self.simulation_configs.size);
            self.render_density(&mut imgbuf, i);
        }
    }
}
