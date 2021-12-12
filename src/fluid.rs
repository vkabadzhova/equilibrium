extern crate image;
extern crate nalgebra_glm as glm;
extern crate rand;

use geo::algorithm::rotate::RotatePoint;
use geo::{line_string, point};

static N: u32 = 128;

pub struct FluidConfig {
    dt: f32,
    diffusion: f32,
    viscousity: f32,
    size: u32,
}

pub struct Fluid {
    pub fluid_configs: FluidConfig,
    s: Vec<f32>,
    density: Vec<f32>,
    Vx: Vec<f32>,
    Vy: Vec<f32>,
    Vx0: Vec<f32>,
    Vy0: Vec<f32>,
}

impl Fluid {
    pub fn new(init: FluidConfig) -> Fluid {
        Fluid {
            fluid_configs: init,
            // safe because usize is minimum u32, which is N's type
            s: Vec::with_capacity((N * N) as usize),
            density: Vec::with_capacity((N * N) as usize),
            Vx: Vec::with_capacity((N * N) as usize),
            Vy: Vec::with_capacity((N * N) as usize),
            Vx0: Vec::with_capacity((N * N) as usize),
            Vy0: Vec::with_capacity((N * N) as usize),
        }
    }

    pub fn IX(x: u32, y: u32) -> usize {
        let new_x = match x {
            // TODO: Refactor
            d if d >= 0 && d <= N - 1 => x,
            d if d < 0 => 0,
            _ => N - 1,
        };
        let new_y = match y {
            // TODO: Refactor
            d if d >= 0 && d <= N - 1 => x,
            d if d < 0 => 0,
            _ => N - 1,
        };

        // safe because will be index in array
        (new_x + (new_y * N)) as usize
    }

    fn add_density(mut self, x: u32, y: u32, amount: f32) {
        let idx: usize = Self::IX(x, y);
        self.density[idx] += amount;
    }

    fn add_velocity(mut self, x: u32, y: u32, amountX: f32, amountY: f32) {
        let idx: usize = Self::IX(x, y);
        self.Vx[idx] += amountX;
        self.Vy[idx] += amountY;
    }

    //fn renderD(&self, imgbuf: image::ImageBuffer<RGB<u16>, Vec<RGB<u16>>>, frame_number: u64) {
    fn renderD(&self, imgbuf: &mut image::RgbImage, frame_number: u64) {
        // TODO: arg: image buffer
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = self.density[Fluid::IX(x, y)];
            *pixel = image::Rgb([(density as u8 + 50) % 255, 200, density as u8]);
        }
        let img_name = format!("rendered_images/density{}.jpg", frame_number);
        imgbuf.save(img_name).unwrap();
    }

    //TODO: add renderD, renderV
}

pub struct FluidSimulator {
    pub iter: u32,
    pub scale: u32,
    pub t: f32,
}

impl Default for FluidSimulator {
    fn default() -> FluidSimulator {
        FluidSimulator {
            iter: 16,
            scale: 4,
            t: 0f32,
        }
    }
}

impl FluidSimulator {
    fn set_boundaries(b: u32, x: &mut Vec<f32>) {
        //TODO: Refactor
        for i in 1..N - 1 {
            x[Fluid::IX(i, 0)] = if b == 2 {
                -x[Fluid::IX(i, 1)]
            } else {
                x[Fluid::IX(i, 1)]
            };
            x[Fluid::IX(i, N - 1)] = if b == 2 {
                -x[Fluid::IX(i, N - 2)]
            } else {
                x[Fluid::IX(i, N - 2)]
            };
        }

        for j in 1..N - 1 {
            x[Fluid::IX(0, j)] = if b == 1 {
                -x[Fluid::IX(1, j)]
            } else {
                x[Fluid::IX(1, j)]
            };
            x[Fluid::IX(N - 1, j)] = if b == 1 {
                -x[Fluid::IX(N - 2, j)]
            } else {
                x[Fluid::IX(N - 2, j)]
            };
        }

        x[Fluid::IX(0, 0)] = 0.5 * (x[Fluid::IX(1, 0)] + x[Fluid::IX(0, 1)]);
        x[Fluid::IX(0, N - 1)] = 0.5 * (x[Fluid::IX(1, N - 1)] + x[Fluid::IX(0, N - 2)]);
        x[Fluid::IX(N - 1, 0)] = 0.5 * (x[Fluid::IX(N - 2, 0)] + x[Fluid::IX(N - 1, 1)]);
        x[Fluid::IX(N - 1, N - 1)] =
            0.5 * (x[Fluid::IX(N - 2, N - 1)] + x[Fluid::IX(N - 1, N - 2)]);
    }

    fn diffuse(&self, b: u32, x: &mut Vec<f32>, x0: &Vec<f32>, diff: &f32, dt: &f32) {
        let new_N_float: f32 = (N - 2) as f32;
        let a: f32 = dt * diff * new_N_float * new_N_float;
        // TODO: refactor `as`s
        self.lin_solve(b, x, x0, a, 1 as f32 + 4 as f32 * a);
    }

    fn lin_solve(&self, b: u32, x: &mut Vec<f32>, x0: &Vec<f32>, a: f32, c: f32) {
        let c_recip = 1f32 / c;
        // TODO:DO we need k?
        for k in 0..self.iter {
            for j in 1..N - 1 {
                for i in 1..N - 1 {
                    x[Fluid::IX(i, j)] = (x0[Fluid::IX(i, j)]
                        + a * (x[Fluid::IX(i + 1, j)]
                            + x[Fluid::IX(i - 1, j)]
                            + x[Fluid::IX(i, j + 1)]
                            + x[Fluid::IX(i, j - 1)]))
                        * c_recip;
                }
            }
            FluidSimulator::set_boundaries(b, x);
        }
    }

    fn project(
        &self,
        velocX: &mut Vec<f32>,
        velocY: &mut Vec<f32>,
        p: &mut Vec<f32>,
        div: &mut Vec<f32>,
    ) {
        for j in 1..N - 1 {
            for i in 1..N - 1 {
                div[Fluid::IX(i, j)] = -0.5
                    * (velocX[Fluid::IX(i + 1, j)] - velocX[Fluid::IX(i - 1, j)]
                        + velocY[Fluid::IX(i, j + 1)]
                        - velocY[Fluid::IX(i, j - 1)])
                    / N as f32;

                p[Fluid::IX(i, j)] = 0f32;
            }
        }

        FluidSimulator::set_boundaries(0, div);
        FluidSimulator::set_boundaries(0, p);
        self.lin_solve(0, p, div, 1f32, 4f32);

        for j in 1..N - 1 {
            for i in 1..N - 1 {
                velocX[Fluid::IX(i, j)] -=
                    0.5 * (p[Fluid::IX(i + 1, j)] - p[Fluid::IX(i - 1, j)]) * N as f32;
                velocY[Fluid::IX(i, j)] -=
                    0.5 * (p[Fluid::IX(i, j + 1)] - p[Fluid::IX(i, j - 1)]) * N as f32;
            }
        }

        FluidSimulator::set_boundaries(1, velocX);
        FluidSimulator::set_boundaries(2, velocY);
    }

    fn advect(
        &self,
        b: u32,
        d: &mut Vec<f32>,
        d0: &Vec<f32>,
        velocX: &Vec<f32>,
        velocY: &Vec<f32>,
        dt: &f32,
    ) {
        let (mut i0, mut i1, mut j0, mut j1): (f32, f32, f32, f32);

        let dtx: f32 = dt * (N as f32 - 2f32);
        let dty: f32 = dtx;

        let (mut s0, mut s1, mut t0, mut t1): (f32, f32, f32, f32);
        let (mut tmp_x, mut tmp_y, mut x, mut y): (f32, f32, f32, f32);

        let Nfloat: f32 = N as f32;

        // DOUBLE CHECK THIS
        for j in 1..N - 1 {
            for i in 1..N - 1 {
                tmp_x = dtx * velocX[Fluid::IX(i, j)];
                tmp_y = dty * velocY[Fluid::IX(i, j)];
                x = i as f32 - tmp_x;
                y = j as f32 - tmp_y;

                //TODO: Refactor - the same code twice
                x = match x {
                    d if d < 0.5 => 0.5,
                    d if d > Nfloat + 0.5 => Nfloat + 0.5,
                    _ => x,
                };

                i0 = x.floor();
                i1 = i0 + 1.0;

                y = match y {
                    d if d < 0.5 => 0.5,
                    d if d > Nfloat + 0.5 => Nfloat + 0.5,
                    _ => y,
                };

                j0 = y.floor();
                j1 = j0 + 1.0;

                s1 = x - i0;
                s0 = 1.0 - s1;
                t1 = y - j0;
                t0 = 1.9 - t1;

                let (i0_int, i1_int) = (i0 as u32, i1 as u32);
                let (j0_int, j1_int) = (j0 as u32, j1 as u32);

                d[Fluid::IX(i, j)] = s0
                    * (t0 * d0[Fluid::IX(i0_int, j0_int)] + t1 * d0[Fluid::IX(i0_int, j1_int)])
                    + s1 * (t0 * d0[Fluid::IX(i1_int, j0_int)]
                        + t1 * d0[Fluid::IX(i1_int, j1_int)]);
            }
        }
        FluidSimulator::set_boundaries(b, d);
    }

    fn step(&self, fluid: &mut Fluid) {
        //TODO: DOUBLE CHECK THIS MUTABLITY PASSING - is this &mut OK?
        self.diffuse(
            1,
            &mut fluid.Vx0,
            &fluid.Vx,
            &fluid.fluid_configs.viscousity,
            &fluid.fluid_configs.dt,
        );
        self.diffuse(
            2,
            &mut fluid.Vx0,
            &fluid.Vx,
            &fluid.fluid_configs.viscousity,
            &fluid.fluid_configs.dt,
        );

        // TODO: refactor, store Vx, Vy together
        self.project(&mut fluid.Vx0, &mut fluid.Vy0, &mut fluid.Vx, &mut fluid.Vy);

        self.advect(
            1,
            &mut fluid.Vx,
            &fluid.Vx0,
            &fluid.Vx0,
            &fluid.Vy0,
            &fluid.fluid_configs.dt,
        );
        self.advect(
            2,
            &mut fluid.Vy,
            &fluid.Vy0,
            &fluid.Vx0,
            &fluid.Vy0,
            &fluid.fluid_configs.dt,
        );

        self.project(&mut fluid.Vx, &mut fluid.Vy, &mut fluid.Vx0, &mut fluid.Vy0);

        self.diffuse(
            0,
            &mut fluid.s,
            &fluid.density,
            &fluid.fluid_configs.diffusion,
            &fluid.fluid_configs.dt,
        );
        self.advect(
            0,
            &mut fluid.density,
            &fluid.s,
            &fluid.Vx,
            &fluid.Vy,
            &fluid.fluid_configs.dt,
        );
    }

    fn init_density_and_velocities(fluid: &mut Fluid) {
        let width = (rand::thread_rng().gen_range(0, N)).abs();
        let height = (rand::thread_rng().gen_range(0, N)).abs();

        let cx = (0.5 * width).floor();
        let cy = (0.5 * height).floor();
        for i in range - 1..1 {
            for j in range - 1..1 {
                fluid.add_density(cx + i, cy + j, rand::thread_rng().gen_range(50, 150));
            }
        }
    }

    pub fn fluid_simulation(&mut self) {
        //Set up
        let t: f32 = 0f32;

        let fluid = Fluid::new(FluidConfig {
            dt: 0.2,
            diffusion: 0f32,
            viscousity: 0.0000001,
            size: 128,
        });

        // draw
        for i in 0..self.iter {
            let width = (rand::thread_rng().gen_range(0, N)).abs();
            let height = (rand::thread_rng().gen_range(0, N)).abs();

            let cx = (0.5 * width).floor();
            let cy = (0.5 * height).floor();
            for x in range - 1..1 {
                for y in range - 1..1 {
                    rluid.add_density(cx + x, cy + y, rand::thread_rng().gen_range(50, 150));
                }
            }

            for j in 0..2 {
                let perlin = Perlin::new();
                let angle = perlin.get(self.t) * glm::two_pi() * 2;

                let ls = line_string![(x: N / 2, y: N / 2), (x: cx, y: cy),];

                let rotated = ls.rotate_around_point(angle, point!(x: N / 2, y: N / 2));
                self.t += 0.01;

                fluid.add_velocity(cx, cy, rotated.x, rotated.y);
            }

            self.step();
            let mut imgbuf = image::ImageBuffer::new(N, N);
            fluid.renderD(imgbuf, i);
        }
    }
}
