extern crate image;
extern crate nalgebra_glm as glm;
extern crate rand;

use geo::algorithm::rotate::RotatePoint;
use geo::{line_string, point};
use noise::{NoiseFn, Perlin};
use rand::Rng;

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
            s: vec![0.0; (&init.size * &init.size) as usize],
            density: vec![0.0; (&init.size * &init.size) as usize],
            Vx: vec![0.0; (&init.size * &init.size) as usize],
            Vy: vec![0.0; (&init.size * &init.size) as usize],
            Vx0: vec![0.0; (&init.size * &init.size) as usize],
            Vy0: vec![0.0; (&init.size * &init.size) as usize],
            fluid_configs: init,
        }
    }

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
    }

    fn add_density(&mut self, x: u32, y: u32, amount: f32) {
        dbg!("-----------------------------------------");
        dbg!(" # Fluid::ADD_DENSITY");
        dbg!("-----------------------------------------");
        dbg!("Added density => x:{}, y:{}, amount:{}", &x, &y, &amount);
        let idx: usize = Self::IX(x, y);
        self.density[idx] += amount;
    }

    fn add_velocity(&mut self, x: u32, y: u32, amountX: f32, amountY: f32) {
        dbg!("-----------------------------------------");
        dbg!(" # Fluid::ADD_VELOCITY");
        dbg!("-----------------------------------------");
        dbg!(
            "Added velocity => x:{}, y:{}, amountX:{}, amountY:{}",
            &x,
            &y,
            &amountX,
            &amountY
        );
        let idx: usize = Self::IX(x, y);
        self.Vx[idx] += amountX;
        self.Vy[idx] += amountY;
    }

    // TODO: make it a test
    fn check_if_density(imgbuf: &mut image::RgbImage, img_name: &String) {
        dbg!("-----------------------------------------");
        dbg!(" # Fluid::CHECK_IF_DENSITY");
        dbg!("-----------------------------------------");
        let imgx = N;
        let imgy = N;
        let mut num_different_pixels = 0u32;
        let mut num_same_pixels = 0u32;
        let first_pixel: &image::Rgb<u8> = imgbuf.get_pixel(0, 0);

        for x in 0..imgx {
            for y in 0..imgy {
                let cx = y as f32 - 1.5;
                let cy = x as f32 - 1.5;

                let pixel = imgbuf.get_pixel(x, y);
                if pixel != first_pixel {
                    dbg!("different pixel found! x:{}, y:{}", x, y);
                    num_different_pixels += 1;
                } else if pixel == first_pixel {
                    num_same_pixels += 1;
                }
            }
        }
        dbg!("Num different pixels: {}", num_different_pixels);
        dbg!("Num different pixels: {}", num_same_pixels);
    }

    fn renderD(&self, imgbuf: &mut image::RgbImage, frame_number: u32) {
        dbg!("-----------------------------------------");
        dbg!(" # Fluid::RENDER_D");
        dbg!(" # Frame number: {}", frame_number);
        dbg!("-----------------------------------------");
        // TODO: arg: image buffer
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = self.density[Fluid::IX(x, y)];
            if (density != 0.0) {
                dbg!(x, y, density);
            }
            *pixel = image::Rgb([(density * 255.0) as u8, 200, density as u8]);
        }
        let img_name = format!("rendered_images/density{}.jpg", frame_number);
        // Self::check_if_density(imgbuf, &img_name);
        imgbuf.save(img_name).unwrap();
    }
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
        dbg!("-----------------------------------------");
        dbg!(" # FluidSimulator::DIFFUSE");
        dbg!("-----------------------------------------");
        let new_N_float: f32 = (N - 2) as f32;
        let a: f32 = dt * diff * new_N_float * new_N_float;
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
        dbg!("-----------------------------------------");
        dbg!(" # FluidSimulatioo::PROJECT");
        dbg!("-----------------------------------------");
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
        dbg!("-----------------------------------------");
        dbg!(" # FluidSimulator::ADVECT");
        dbg!("-----------------------------------------");
        let (mut i0, mut i1, mut j0, mut j1): (f32, f32, f32, f32);

        let dtx: f32 = dt * (N as f32 - 2f32);
        let dty: f32 = dtx;

        let (mut s0, mut s1, mut t0, mut t1): (f32, f32, f32, f32);
        let (mut x, mut y): (f32, f32);

        let Nfloat: f32 = N as f32;
        dbg!("Nfloat: {}", Nfloat);

        // DOUBLE CHECK THIS
        for j in 1..N - 1 {
            for i in 1..N - 1 {
                x = i as f32 - dtx * velocX[Fluid::IX(i, j)];
                y = j as f32 - dty * velocY[Fluid::IX(i, j)];
                dbg!(
                    "x: {}, y:{}, velocX: {}, velocY:{}",
                    x,
                    y,
                    velocX[Fluid::IX(i, j)],
                    velocY[Fluid::IX(i, j)]
                );

                //TODO: Refactor - the same code twice
                x = match x {
                    d if d < 0.5 => 0.5,
                    // d if d > Nfloat + 0.5 => Nfloat + 0.5,
                    d if d >= Nfloat => Nfloat - 1.0,
                    _ => x,
                };

                i0 = x.floor();
                i1 = i0 + 1.0;

                y = match y {
                    d if d < 0.5 => 0.5,
                    // d if d > Nfloat + 0.5 => Nfloat + 0.5,
                    d if d >= Nfloat => Nfloat - 1.0,
                    _ => y,
                };

                j0 = y.floor();
                j1 = j0 + 1.0;

                s1 = x - i0;
                s0 = 1.0 - s1;
                t1 = y - j0;
                t0 = 1.0 - t1;

                let (i0_int, i1_int) = (i0 as u32, i1 as u32);
                let (j0_int, j1_int) = (j0 as u32, j1 as u32);

                dbg!(
                    "i: {}, j:{}, i0: {}, j0:{}, i1: {}, j1:{}",
                    i, j, i0, j0, i1, j1
                );

                if i1 >= Nfloat || j1 >= Nfloat {
                    //TODO: Refactor - works only for 0 degrees
                    d[Fluid::IX(i, j)] = d[Fluid::IX(i - 1, j)];
                    break;
                }
                d[Fluid::IX(i, j)] = s0
                    * (t0 * d0[Fluid::IX(i0_int, j0_int)] + t1 * d0[Fluid::IX(i0_int, j1_int)])
                    + s1 * (t0 * d0[Fluid::IX(i1_int, j0_int)]
                        + t1 * d0[Fluid::IX(i1_int, j1_int)]);
            }
        }
        FluidSimulator::set_boundaries(b, d);
    }

    fn step(&self, fluid: &mut Fluid) {
        dbg!("-----------------------------------------");
        dbg!(" # FluidSimulator::STEP");
        dbg!("-----------------------------------------");
        // self.diffuse(
        //     1,
        //     &mut fluid.Vx0,
        //     &fluid.Vx,
        //     &fluid.fluid_configs.viscousity,
        //     &fluid.fluid_configs.dt,
        // );
        // self.diffuse(
        //     2,
        //     &mut fluid.Vy0,
        //     &fluid.Vy,
        //     &fluid.fluid_configs.viscousity,
        //     &fluid.fluid_configs.dt,
        // );

        // self.project(&mut fluid.Vx0, &mut fluid.Vy0, &mut fluid.Vx, &mut fluid.Vy);

        // self.advect(
        //     1,
        //     &mut fluid.Vx,
        //     &fluid.Vx0,
        //     &fluid.Vx0,
        //     &fluid.Vy0,
        //     &fluid.fluid_configs.dt,
        // );
        // self.advect(
        //     2,
        //     &mut fluid.Vy,
        //     &fluid.Vy0,
        //     &fluid.Vx0,
        //     &fluid.Vy0,
        //     &fluid.fluid_configs.dt,
        // );

        // self.project(&mut fluid.Vx, &mut fluid.Vy, &mut fluid.Vx0, &mut fluid.Vy0);

        // self.diffuse(
        //     0,
        //     &mut fluid.s,
        //     &fluid.density,
        //     &fluid.fluid_configs.diffusion,
        //     &fluid.fluid_configs.dt,
        // );
        // self.advect(
        //     0,
        //     &mut fluid.density,
        //     &fluid.s,
        //     &fluid.Vx,
        //     &fluid.Vy,
        //     &fluid.fluid_configs.dt,
        // );
    }

    fn init_densitities_velocities(&mut self, width: u32, height: u32, fluid: &mut Fluid) {
        let cx: u32 = (0.5 * width as f32).floor() as u32;
        let cy: u32 = (0.5 * height as f32).floor() as u32;
        dbg!("cx:{}, cy:{}", cx, cy);

        for x in 0..=2 {
            for y in 0..=2 {
                let x_coef: i32 = x - 1;
                let y_coef: i32 = y - 1;
                dbg!(
                    "Add density => cx:{}, x_coef:{}, cy:{}, y_coef:{}",
                    cx,
                    x_coef,
                    cy,
                    y_coef
                );

                // Safe because sign is reversed after processing u32 -> i32 -> u32
                fluid.add_density(
                    // TODO: When cx == 0 & x_coef = -1, gets out of scope
                    (cx as i32 + x_coef) as u32,
                    (cy as i32 + y_coef) as u32,
                    (rand::thread_rng().gen_range(0..100) / 100) as f32,
                );
            }
        }

        // let perlin = Perlin::new();
        // let t_f64: f64 = self.t as f64;

        //TODO: Refactor with 2pi:
        //let angle: f64 = perlin.get([t_f64, t_f64]) * glm::two_pi() * 2;
        // let angle: f64 = perlin.get([t_f64, t_f64]) * 6.28 * 2f64;
        // dbg!("Rotation angle: {}", angle);

        // let ls = line_string![(x: (N / 2) as f32, y: (N / 2) as f32), (x: cx as f32, y: cy as f32)];
        // let rotated =
        //     ls.rotate_around_point(angle as f32, point!(x: (N / 2) as f32, y: (N / 2) as f32));
        // dbg!(
        //     "Rotated point => {}, {}",
        //     rotated[1].x as f32, rotated[1].y as f32
        // );

        // self.t += 0.01;

        // fluid.add_velocity(cx, cy, rotated[1].x as f32 * 0.2, rotated[1].y as f32 * 0.2);

        self.t += 0.01;

        fluid.add_velocity(cx, cy, cx as f32 * 0.2, cy as f32 * 0.2);
    }

    fn init_densitity(fluid: &mut Fluid) {
        for j in 0..N {
            for i in 0..N {
                fluid.add_density(i, j, 0.0);
            }
        }

        for j in N / 2 - 20..=N / 2 + 20 {
            for i in N / 2 - 20..=N / 2 + 20 {
                fluid.add_density(i, j, 0.9);
            }
        }

        // for j in 0..N {
        //     for i in 0..N {
        //         if (fluid.density)
        //     }
        // }

    }

    fn init_velocities(fluid: &mut Fluid) {
        for j in 0..N {
            for i in 0..N {
                fluid.add_velocity(i, j, 2 as f32 * 0.2, 3 as f32 * 0.2);
            }
        }
    }

    pub fn fluid_simulation(&mut self) {
        dbg!("-----------------------------------------");
        dbg!(" # FluidSimulator::FLUID_SIMULATION");
        dbg!("-----------------------------------------");

        //Set up
        let mut fluid = Fluid::new(FluidConfig {
            dt: 0.2,
            //TODO: SET DIFFUSION!
            diffusion: 0f32,
            viscousity: 0.0000001,
            size: 128,
        });

        // FluidSimulator::init_velocities(&mut fluid);
        FluidSimulator::init_densitity(&mut fluid);

        // draw
        for i in 0..self.iter {
            dbg!("Iteration: {}", i);

            // self.init_densitities_velocities(width, height, &mut fluid);

            self.step(&mut fluid);
            let mut imgbuf = image::ImageBuffer::new(N, N);
            fluid.renderD(&mut imgbuf, i);
        }
    }
}
