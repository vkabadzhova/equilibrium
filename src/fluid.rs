static N: u32 = 128;

pub struct FluidConfig {
    dt: f32,
    diffusion: f32,
    viscousity: f32,
    size: u32,
} 

pub struct Fluid {
    fluid_configs: FluidConfig,
    s: Vec<f32>, 
}


impl Fluid {
    pub fn new(init: FluidConfig) -> Fluid {
       Fluid {
           fluid_configs: init,
           s: Vec::new(),
       }

    }

    fn IX(x: u32, y: u32) -> u32 {
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

        new_x + (new_y * N)
    }

    fn step(&self) {
        //TODO: algorithm            

    }


    fn add_density(&self, x: u32, y: u32, amount: f32) {
        let idx: u32 = Self::IX(x, y);
    }
}

pub fn fluid_simulation() {
    let iter: u32 = 16;
    let SCALE: u32 = 4;

    let fluid = Fluid::new(
        FluidConfig {
        dt: 0.2,
        diffusion: 0.0,
        viscousity: 0.0000001,
        size: 128
        },
    );
}
