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

    fn step(&self) {
        //TODO: algorithm
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
            t: 0.0,
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
        x[Fluid::IX(N - 1, N - 1)] = 0.5 * (x[Fluid::IX(N - 2, N - 1)] + x[Fluid::IX(N - 1, N - 2)]);
    }



    fn diffuse(&self, b: i32, x: &mut Vec<f32>, x0: &Vec<f32>, diff: &f32, dt: &f32) {
        let new_N_float: f32 = (N - 2) as f32;
        let a: f32 = dt * diff * new_N_float * new_N_float;
        // TODO: refactor `as`s
        self.lin_solve(b, x, x0, a, 1 as f32 + 4 as f32 * a);
    }

    fn lin_solve(&self, b: i32, x: &mut Vec<f32>, x0: &Vec<f32>, a: f32, c: f32) {
        let c_recip = 1.0 / c;
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
        }
    }

    fn project(&self, velocX: &mut Vec<f32>, velocY: &mut Vec<f32>, p: &mut Vec<f32>, div: &mut Vec<f32>) {
        for j in 1..N-1 {
            for i in 1..N-1 {
                div[Fluid::IX(i, j)] = -0.5 * (
                    velocX[Fluid::IX(i+1,j)]
                    - velocX[Fluid::IX(i-1, j)]
                    + velocY[Fluid::IX(i, j+1)]
                    - velocY[Fluid::IX(i, j-1)]
                    ) / N as f32;

                p[Fluid::IX(i, j)] = 0.0;
            }
        }

        FluidSimulator::set_boundaries(0, div);
        FluidSimulator::set_boundaries(0, p);
        self.lin_solve(0, p, div, 1.0, 4.0);

        for j in 1..N-1 {
            for i in 1..N-1 { 
                velocX[Fluid::IX(i, j)] -= 0.5 * ( p[Fluid::IX(i+1, j)]
                    -p[Fluid::IX(i-1, j)]) * N as f32;
                velocY[Fluid::IX(i, j)] -= 0.5 * ( p[Fluid::IX(i, j+1)]
                    -p[Fluid::IX(i, j-1)]) * N as f32;
            }
        }

        FluidSimulator::set_boundaries(1, velocX);
        FluidSimulator::set_boundaries(2, velocY);
    }

    fn advect(
        &self,
        b: u32,
        d: &Vec<f32>,
        d0: &Vec<f32>,
        velocX: &Vec<f32>,
        velocY: &Vec<f32>,
        dt: &f32,
    ) {
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
            &fluid.Vx,
            &fluid.Vx0,
            &fluid.Vx0,
            &fluid.Vy0,
            &fluid.fluid_configs.dt,
        );
        self.advect(
            2,
            &fluid.Vy,
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
            &fluid.density,
            &fluid.s,
            &fluid.Vx,
            &fluid.Vy,
            &fluid.fluid_configs.dt,
        );
    }

    pub fn fluid_simulation(&self) {
        let t: f32 = 0.0;

        let fluid = Fluid::new(FluidConfig {
            dt: 0.2,
            diffusion: 0.0,
            viscousity: 0.0000001,
            size: 128,
        });
    }
}
