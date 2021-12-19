// static N: u32 = 128;

pub struct FluidConfig {
    size: u32,
    dt: f32,
    diffusion: f32,
    viscousity: f32,
}

pub struct FluidCube {
    pub fluid_configs: FluidConfig,
    s: Vec<f32>,
    density: Vec<f32>,
    Vx: Vec<f32>,
    Vy: Vec<f32>,
    Vx0: Vec<f32>,
    Vy0: Vec<f32>,
}

impl FluidCube {
    pub fn new(init: FluidConfig) -> FluidCube {
        FluidCube {
            s: vec![0.0; (&init.size * &init.size) as usize],
            density: vec![0.0; (&init.size * &init.size) as usize],
            Vx: vec![0.0; (&init.size * &init.size) as usize],
            Vy: vec![0.0; (&init.size * &init.size) as usize],
            Vx0: vec![0.0; (&init.size * &init.size) as usize],
            Vy0: vec![0.0; (&init.size * &init.size) as usize],
            fluid_configs: init,
        }
    }

    pub fn constrain<T>(var: T, from: T, to: T) -> T 
    where T: PartialOrd
    {
        let var = match var {
            d if d < from => from,
            d if d > to => to,
            _ => var
        };
        var
    }


    pub fn IX(&self, x: &u32, y: &u32) -> usize {
        let x = &FluidCube::constrain(*x, 0, self.fluid_configs.size - 1);
        let y = &FluidCube::constrain(*y, 0, self.fluid_configs.size - 1);
        (x + (y * self.fluid_configs.size)) as usize
    }

    pub fn add_density(&mut self, x: &u32, y: &u32, amount: &f32) {
        let N = self.fluid_configs.size;
        self.density[self.IX(&x, &y)] += amount;
    }

    pub fn add_velocity(&mut self, x: &u32, y: &u32, amountX: &f32, amountY: &f32) {
        let N = self.fluid_configs.size;
        let index = self.IX(&x, &y);

        self.Vx[index] += amountX;
        self.Vy[index] += amountY;
    }

    pub fn step(&mut self) {
        // let N = &self.fluid_configs.size;
        // let visc = &self.fluid_configs.viscousity;
        // let diff = &self.fluid_configs.diffusion;
        // let dt = &self.fluid_configs.dt;
        // let Vx = &self.Vx;
        // let Vy = &self.Vy;
        // let Vx0 =  &self.Vx0;
        // let Vy0 = &self.Vy0;
        // let s = &self.s;
        // let density = &self.density;

        // self.diffuse_x();
        // self.diffuse_y();

        // self.project(&mut Vx0, &mut Vy0, &mut Vx, &mut Vy, 4, N);

        // self.advect(1, &mut Vx, Vx0, Vx0, Vy0, dt, N);
        // self.advect(2, &mut Vy, Vy0, Vx0, Vy0, dt, N);

        // self.project(&mut Vx, &mut Vy, &mut Vx0, &mut Vy0, 4, N);

        // self.diffuse_density();
        // self.advect(0, density, s, Vx, Vy, dt, N);

        self.diffuse_x();
        self.diffuse_y();

        self.project(&self.Vx0, &mut Vy0, &mut Vx, &mut Vy, 4, N);

        self.advect(1, &mut Vx, Vx0, Vx0, Vy0, dt, N);
        self.advect(2, &mut Vy, Vy0, Vx0, Vy0, dt, N);

        self.project(&mut Vx, &mut Vy, &mut Vx0, &mut Vy0, 4, N);

        self.diffuse_density();
    }

    pub fn set_bnd(&self, b: u32, x: &mut Vec<f32>, N: &u32) {
        for i in 1..=N - 1 {
            x[self.IX(&i, &0)] = if b == 2 {
                -x[self.IX(&i, &1)]
            } else {
                x[self.IX(&i, &1)]
            };
            x[self.IX(&i, &(N - 1))] = if b == 2 {
                -x[self.IX(&i, &(N - 2))]
            } else {
                x[self.IX(&i, &(N - 2))]
            };
        }
        for j in 1..=N - 1 {
            x[self.IX(&0, &j)] = if b == 1 {
                -x[self.IX(&1, &j)]
            } else {
                x[self.IX(&1, &j)]
            };
            x[self.IX(&(N - 1), &j)] = if b == 1 {
                -x[self.IX(&(N - 2), &j)]
            } else {
                x[self.IX(&(N - 2), &j)]
            };
        }

        //TODO: why 0.33f?
        x[self.IX(&0, &0)] = 0.33f32 * (x[self.IX(&1, &0)] + x[self.IX(&0, &1)]);
        x[self.IX(&0, &(N - 1))] =
            0.33f32 * (x[self.IX(&1, &(N - 1))] + x[self.IX(&0, &(N - 2))]);
        x[self.IX(&(N - 1), &0)] =
            0.33f32 * (x[self.IX(&(N - 2), &0)] + x[self.IX(&(N - 1), &1)]);
        x[self.IX(&(N - 1), &(N - 1))] =
            0.33f32 * (x[self.IX(&(N - 2), &(N - 1))] + x[self.IX(&(N - 1), &(N - 2))]);
    }

    pub fn lin_solve(&self, b: u32, x: &mut Vec<f32>, x0: &Vec<f32>, a: f32, c: f32, iter: u32, N: u32) {
        let cRecip = 1.0 / c;
        for k in 0..iter {
            for j in 1..N - 1 {
                for i in 1..N - 1 {
                    x[self.IX(&i, &j)] = (x0[self.IX(&i, &j)]
                        + a * (x[self.IX(&(i + 1), &j)]
                            + x[self.IX(&(i - 1), &j)]
                            + x[self.IX(&i, &(j + 1))]
                            + x[self.IX(&i, &(j - 1))]))
                        * cRecip;
                }
            }
            self.set_bnd(b, x, &N);
        }
    }

    pub fn diffuse_x(
        &self,
    ) {
        let a: f32 = &self.fluid_configs.dt * &self.fluid_configs.diffusion * (&self.fluid_configs.size - 2) as f32 * (self.fluid_configs.size - 2) as f32;
        self.lin_solve(1, &mut self.Vx0, &mut self.Vx, a, 1 as f32 + a * 6 as f32, 4, self.fluid_configs.size);
    }

    pub fn diffuse_y(
        &self,
    ) {
        let a: f32 = &self.fluid_configs.dt * &self.fluid_configs.diffusion * (&self.fluid_configs.size - 2) as f32 * (self.fluid_configs.size - 2) as f32;
        self.lin_solve(2, &mut self.Vy0, &mut self.Vy, a, 1 as f32 + a * 6 as f32, 4, self.fluid_configs.size);
    }

    pub fn diffuse_density(
        &self,
    ) {
        let a: f32 = &self.fluid_configs.dt * &self.fluid_configs.diffusion * (&self.fluid_configs.size - 2) as f32 * (self.fluid_configs.size - 2) as f32;
        self.lin_solve(0, &mut self.s, &mut self.density, a, 1 as f32 + a * 6 as f32, 4, self.fluid_configs.size);
    }

    // pub fn diffuse_density(
    //     &self,
    //     b: u32,
    //     x: &mut Vec<f32>,
    //     x0: &Vec<f32>,
    //     diff: &f32,
    //     dt: &f32,
    //     iter: u32,
    //     N: &u32,
    // ) {
    //     let a = dt * diff * (N - 2) as f32 * (self.fluid_configs.size - 2) as f32;
    //     self.lin_solve(b, x, x0, a, 1 as f32 + a * 6 as f32, iter, *N);
    // }

    pub fn project(
        &self, 
        velocX: &mut Vec<f32>,
        velocY: &mut Vec<f32>,
        p: &mut Vec<f32>,
        div: &mut Vec<f32>,
        iter: u32,
        N: &u32,
    ) {
        for j in 1..N - 1 {
            for i in 1..N - 1 {
                div[self.IX(&i, &j)] = -0.5f32
                    * (velocX[self.IX(&(i + 1), &j)] - velocX[self.IX(&(i - 1), &j)]
                        + velocY[self.IX(&i, &(j + 1))]
                        - velocY[self.IX(&i, &(j - 1))])
                    / *N as f32;

                p[self.IX(&i, &j)] = 0.0;
            }
        }
        self.set_bnd(0, div, N);
        self.set_bnd(0, p, N);
        self.lin_solve(0, p, div, 1.0, 6.0, iter, *N);

        for j in 1..N - 1 {
            for i in 1..N - 1 {
                velocX[self.IX(&i, &j)] -= 0.5f32
                    * (p[self.IX(&(i + 1), &j)] - p[self.IX(&(i - 1), &j)])
                    * *N as f32;
                velocY[self.IX(&i, &j)] -= 0.5f32
                    * (p[self.IX(&i, &(j + 1))] - p[self.IX(&i, &(j - 1))])
                    * *N as f32;
            }
        }
        self.set_bnd(1, velocX, N);
        self.set_bnd(2, velocY, N);
    }

    pub fn advect(
        self, 
        b: u32,
        d: &mut Vec<f32>,
        d0: &Vec<f32>,
        velocX: &Vec<f32>,
        velocY: &Vec<f32>,
        dt: &f32,
        N: &u32,
    ) {
        let (mut i0, mut i1, mut j0, mut j1): (f32, f32, f32, f32);

        let dtx = dt * (*N as f32 - 2.0);
        let dty = dt * (*N as f32 - 2.0);

        let (mut s0, mut s1, mut t0, mut t1): (f32, f32, f32, f32);
        let (mut x, mut y): (f32, f32);

        let Nfloat = *N as f32;
        let (mut ifloat, mut jfloat): (f32, f32);

        for j in 1..N - 1 {
            jfloat = j as f32;
            for i in 1..N - 1 {
                ifloat = i as f32;
                x = ifloat - dtx * velocX[self.IX(&i, &j)];
                y = jfloat - dty * velocY[self.IX(&i, &j)];

                x = FluidCube::constrain(x, 0.5, Nfloat + 0.5);
                y = FluidCube::constrain(y, 0.5, Nfloat + 0.5);

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

                d[self.IX(&i, &j)] = 
                    s0 * (t0 * d0[self.IX(&i0_int, &j0_int)] + t1 * d0[self.IX(&i0_int, &j1_int)]) + 
                    s1 * (t0 * d0[self.IX(&i1_int, &j0_int)] + t1 * d0[self.IX(&i1_int, &j1_int)]);
            }
        }
        self.set_bnd(b, d, N);
    }

    //Vx, Vx0, Vx0, Vy0
    pub fn advect_x(&mut self) {

    }
}
