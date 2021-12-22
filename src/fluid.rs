extern crate image;
extern crate nalgebra_glm as glm;
extern crate rand;

use geo::algorithm::rotate::RotatePoint;
use geo::{line_string, point};
use noise::{NoiseFn, Perlin};
use rand::Rng;

static N: u32 = 128;

pub struct FluidConfig {
    diffusion: f32,
    viscousity: f32,
    size: u32,
}

pub struct Fluid {
    pub fluid_configs: FluidConfig,
    s: Vec<f32>,
    density: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    velocities_x0: Vec<f32>,
    velocities_y0: Vec<f32>,
}

impl Fluid {
    pub fn new(init: FluidConfig) -> Fluid {
        Fluid {
            s: vec![0.0; (&init.size * &init.size) as usize],
            density: vec![0.0; (&init.size * &init.size) as usize],
            velocities_x: vec![0.0; (&init.size * &init.size) as usize],
            velocities_y: vec![0.0; (&init.size * &init.size) as usize],
            velocities_x0: vec![0.0; (&init.size * &init.size) as usize],
            velocities_y0: vec![0.0; (&init.size * &init.size) as usize],
            fluid_configs: init,
        }
    }

    /// Returns the variable's value constrained between from and to (both inclusive)
    /// # Arguments
    /// * `var` - The variable whoose value to be processed
    /// * `from` - the minimal value that can be hold (inclusive)
    /// * `to` - the maximum value that can be hold (inclusive)
    pub fn constrain<T: PartialOrd>(var: T, from: T, to: T) -> T {
        let var = match var {
            d if d < from => from,
            d if d > to => to,
            _ => var,
        };
        var
    }

    fn coordinate_to_idx(&self, x: u32, y: u32) -> usize {
        let x = &Fluid::constrain(x, 0, self.fluid_configs.size - 1);
        let y = &Fluid::constrain(y, 0, self.fluid_configs.size - 1);
        (x + (y * self.fluid_configs.size)) as usize
    }

    fn add_density(&mut self, x: u32, y: u32, amount: f32) {
        let idx: usize = Self::coordinate_to_idx(x, y);
        self.density[idx] += amount;
        self.s[idx] += amount;
    }

    fn add_velocity(&mut self, x: u32, y: u32, amount_x: f32, amount_y: f32) {
        let idx: usize = Self::coordinate_to_idx(x, y);
        self.velocities_x[idx] += amount_x;
        self.velocities_y[idx] += amount_y;
    }

    fn render_density(&self, imgbuf: &mut image::RgbImage, frame_number: u32) {
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = self.density[Fluid::coordinate_to_idx(x, y)];
            *pixel = image::Rgb([(density * 255.0) as u8, 200, density as u8]);
        }
        let img_name = format!("rendered_images/density{}.jpg", frame_number);
        imgbuf.save(img_name).unwrap();
    }
}

pub struct FluidSimulator {
    pub dt: f32,
    pub iter: u32,
    pub scale: u32,
    pub t: f32,
    pub size: u32,
}

impl Default for FluidSimulator {
    fn default() -> FluidSimulator {
        FluidSimulator {
            dt: 0.02,
            iter: 16,
            scale: 4,
            t: 0f32,
            size: 128,
        }
    }
}

impl FluidSimulator {
    fn set_boundaries(b: u32, x: &mut Vec<f32>) {
        for i in 1..N - 1 {
            x[Fluid::coordinate_to_idx(i, 0)] = if b == 2 {
                -x[Fluid::coordinate_to_idx(i, 1)]
            } else {
                x[Fluid::coordinate_to_idx(i, 1)]
            };
            x[Fluid::coordinate_to_idx(i, N - 1)] = if b == 2 {
                -x[Fluid::coordinate_to_idx(i, N - 2)]
            } else {
                x[Fluid::coordinate_to_idx(i, N - 2)]
            };
        }

        for j in 1..N - 1 {
            x[Fluid::coordinate_to_idx(0, j)] = if b == 1 {
                -x[Fluid::coordinate_to_idx(1, j)]
            } else {
                x[Fluid::coordinate_to_idx(1, j)]
            };
            x[Fluid::coordinate_to_idx(N - 1, j)] = if b == 1 {
                -x[Fluid::coordinate_to_idx(N - 2, j)]
            } else {
                x[Fluid::coordinate_to_idx(N - 2, j)]
            };
        }

        x[Fluid::coordinate_to_idx(0, 0)] =
            0.5 * (x[Fluid::coordinate_to_idx(1, 0)] + x[Fluid::coordinate_to_idx(0, 1)]);
        x[Fluid::coordinate_to_idx(0, N - 1)] =
            0.5 * (x[Fluid::coordinate_to_idx(1, N - 1)] + x[Fluid::coordinate_to_idx(0, N - 2)]);
        x[Fluid::coordinate_to_idx(N - 1, 0)] =
            0.5 * (x[Fluid::coordinate_to_idx(N - 2, 0)] + x[Fluid::coordinate_to_idx(N - 1, 1)]);
        x[Fluid::coordinate_to_idx(N - 1, N - 1)] = 0.5
            * (x[Fluid::coordinate_to_idx(N - 2, N - 1)]
                + x[Fluid::coordinate_to_idx(N - 1, N - 2)]);
    }

    fn diffuse(&self, b: u32, x: &mut Vec<f32>, x0: &Vec<f32>, diffusion: &f32) {
        let size_float: f32 = (&self.size - 2) as f32;
        let a: f32 = &self.dt * diffusion * size_float * size_float;
        self.lin_solve(b, x, x0, a, 1 as f32 + 4 as f32 * a);
    }

    fn lin_solve(&self, b: u32, x: &mut Vec<f32>, x0: &Vec<f32>, a: f32, c: f32) {
        let c_recip = 1f32 / c;
        for _k in 0..self.iter {
            for j in 1..&self.size - 1 {
                for i in 1..&self.size - 1 {
                    x[Fluid::coordinate_to_idx(i, j)] = (x0[Fluid::coordinate_to_idx(i, j)]
                        + a * (x[Fluid::coordinate_to_idx(i + 1, j)]
                            + x[Fluid::coordinate_to_idx(i - 1, j)]
                            + x[Fluid::coordinate_to_idx(i, j + 1)]
                            + x[Fluid::coordinate_to_idx(i, j - 1)]))
                        * c_recip;
                }
            }
            FluidSimulator::set_boundaries(b, x);
        }
    }

    fn project(
        &self,
        velocities_x: &mut Vec<f32>,
        velocities_y: &mut Vec<f32>,
        p: &mut Vec<f32>,
        div: &mut Vec<f32>,
    ) {
        for j in 1..&self.size - 1 {
            for i in 1..&self.size - 1 {
                div[Fluid::coordinate_to_idx(i, j)] = -0.5
                    * (velocities_x[Fluid::coordinate_to_idx(i + 1, j)]
                        - velocities_x[Fluid::coordinate_to_idx(i - 1, j)]
                        + velocities_y[Fluid::coordinate_to_idx(i, j + 1)]
                        - velocities_y[Fluid::coordinate_to_idx(i, j - 1)])
                    / self.size as f32;

                p[Fluid::coordinate_to_idx(i, j)] = 0f32;
            }
        }

        FluidSimulator::set_boundaries(0, div);
        FluidSimulator::set_boundaries(0, p);
        self.lin_solve(0, p, div, 1f32, 4f32);

        for j in 1..self.size - 1 {
            for i in 1..self.size - 1 {
                velocities_x[Fluid::coordinate_to_idx(i, j)] -= 0.5
                    * (p[Fluid::coordinate_to_idx(i + 1, j)]
                        - p[Fluid::coordinate_to_idx(i - 1, j)])
                    * self.size as f32;
                velocities_y[Fluid::coordinate_to_idx(i, j)] -= 0.5
                    * (p[Fluid::coordinate_to_idx(i, j + 1)]
                        - p[Fluid::coordinate_to_idx(i, j - 1)])
                    * self.size as f32;
            }
        }

        FluidSimulator::set_boundaries(1, velocities_x);
        FluidSimulator::set_boundaries(2, velocities_y);
    }

    fn advect(
        &self,
        boundary: u32,
        densities: &mut Vec<f32>,
        densities0: &Vec<f32>,
        velocities_x: &Vec<f32>,
        velocities_y: &Vec<f32>,
    ) {
        let (mut i0, mut i1, mut j0, mut j1): (f32, f32, f32, f32);

        let dtx: f32 = &self.dt * (self.size as f32 - 2f32);
        let dty: f32 = dtx;

        let (mut s0, mut s1, mut t0, mut t1): (f32, f32, f32, f32);
        let (mut x, mut y): (f32, f32);

        let size_float: f32 = self.size as f32;

        for j in 1..self.size - 1 {
            for i in 1..self.size - 1 {
                x = i as f32 - dtx * velocities_x[Fluid::coordinate_to_idx(i, j)];
                y = j as f32 - dty * velocities_y[Fluid::coordinate_to_idx(i, j)];

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
                    densities[Fluid::coordinate_to_idx(i, j)] =
                        densities[Fluid::coordinate_to_idx(i - 1, j)];
                    break;
                }
                densities[Fluid::coordinate_to_idx(i, j)] = s0
                    * (t0 * densities0[Fluid::coordinate_to_idx(i0_int, j0_int)]
                        + t1 * densities0[Fluid::coordinate_to_idx(i0_int, j1_int)])
                    + s1 * (t0 * densities0[Fluid::coordinate_to_idx(i1_int, j0_int)]
                        + t1 * densities0[Fluid::coordinate_to_idx(i1_int, j1_int)]);
            }
        }
        FluidSimulator::set_boundaries(boundary, densities);
    }

    fn step(&self, fluid: &mut Fluid) {
        self.diffuse(
            1,
            &mut fluid.velocities_x0,
            &fluid.velocities_x,
            &fluid.fluid_configs.viscousity,
        );
        self.diffuse(
            2,
            &mut fluid.velocities_y0,
            &fluid.velocities_y,
            &fluid.fluid_configs.viscousity,
        );

        self.project(
            &mut fluid.velocities_x0,
            &mut fluid.velocities_y0,
            &mut fluid.velocities_x,
            &mut fluid.velocities_y,
        );

        self.advect(
            1,
            &mut fluid.velocities_x,
            &fluid.velocities_x0,
            &fluid.velocities_x0,
            &fluid.velocities_y0,
        );
        self.advect(
            2,
            &mut fluid.velocities_y,
            &fluid.velocities_y0,
            &fluid.velocities_x0,
            &fluid.velocities_y0,
        );

        self.project(
            &mut fluid.velocities_x,
            &mut fluid.velocities_y,
            &mut fluid.velocities_x0,
            &mut fluid.velocities_y0,
        );

        self.diffuse(
            0,
            &mut fluid.s,
            &fluid.density,
            &fluid.fluid_configs.diffusion,
        );

        self.advect(
            0,
            &mut fluid.density,
            &fluid.s,
            &fluid.velocities_x,
            &fluid.velocities_y,
        );

        fluid.s = fluid.density.clone();
    }

    fn init_densitity(fluid: &mut Fluid) {
        for j in 0..fluid.fluid_configs.size {
            for i in 0..fluid.fluid_configs.size {
                fluid.add_density(i, j, 0.0);
            }
        }

        for j in fluid.fluid_configs.size / 2 - 10..=fluid.fluid_configs.size / 2 + 10 {
            for i in fluid.fluid_configs.size / 2 - 10..=fluid.fluid_configs.size / 2 + 10 {
                fluid.add_density(i, j, 0.9);
            }
        }
    }

    fn init_velocities(fluid: &mut Fluid) {
        for j in 0..fluid.fluid_configs.size {
            for i in 0..fluid.fluid_configs.size {
                fluid.add_velocity(i, j, 1 as f32, 1 as f32);
            }
        }
    }

    fn add_noise(&mut self, fluid: &mut Fluid) {
        let perlin = Perlin::new();
        let t_f64: f64 = self.t as f64;

        let angle: f64 = perlin.get([t_f64, t_f64]) * 6.28 * 2f64;
        let rand_x = rand::thread_rng().gen_range(0..self.size);
        let rand_y = rand::thread_rng().gen_range(0..self.size);

        let (center_point, rotating_point) = (
            point!(x: (self.size/2) as f32, y: (self.size/2) as f32),
            point!(x: rand_x as f32, y: rand_y as f32),
        );

        let ls = line_string![(x: (self.size/2) as f32, y: (self.size/2) as f32), (x: rand_x as f32, y: rand_y as f32)];

        // let ls =
        //     line_string![center_point, rotating_point];

        let rotated = ls.rotate_around_point(angle as f32, center_point);

        self.t += 0.01;

        fluid.add_velocity(
            center_point.x().trunc() as u32,
            center_point.y().trunc() as u32,
            rotated[1].x as f32 * 2.0,
            rotated[1].y as f32 * 2.0,
        );
    }

    pub fn fluid_simulation(&mut self) {
        //Set up
        let mut fluid = Fluid::new(FluidConfig {
            diffusion: 0f32,
            viscousity: 0.0000001,
            size: 128,
        });

        FluidSimulator::init_velocities(&mut fluid);
        FluidSimulator::init_densitity(&mut fluid);

        // draw
        for i in 0..self.iter {
            self.add_noise(&mut fluid);
            self.step(&mut fluid);

            let mut imgbuf = image::ImageBuffer::new(self.size, self.size);
            fluid.render_density(&mut imgbuf, i);
        }
    }
}
