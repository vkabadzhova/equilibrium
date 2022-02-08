/// Major configurations in order to run the simulation
#[derive(Copy, Clone)]
pub struct SimulationConfigs {
    /// The size of each step
    pub delta_t: f32,
    /// Number of frames to be simulated
    pub frames: i64,
    /// The size of the fluid. A square container is used
    pub size: u32,
}

impl Default for SimulationConfigs {
    fn default() -> SimulationConfigs {
        SimulationConfigs {
            delta_t: 0.02,
            frames: 16,
            size: 128,
        }
    }
}

impl SimulationConfigs {
    /// Creates new SimulationConfigs struct
    pub fn new(delta_t: f32, frames: i64, fluid_container_size: u32) -> Self {
        SimulationConfigs {
            delta_t: delta_t,
            frames: frames,
            size: fluid_container_size,
        }
    }
}

/// Struct describing general fluid-related configurations
#[derive(Copy, Clone)]
pub struct FluidConfigs {
    /// Fluid's diffusion
    pub diffusion: f32,
    /// Fluid's viscousity
    pub viscousity: f32,
    /// Shows if random perlin noise is enabled
    pub has_perlin_noise: bool,
}

impl Default for FluidConfigs {
    fn default() -> FluidConfigs {
        FluidConfigs {
            diffusion: 0.0,
            viscousity: 0.001,
            has_perlin_noise: true,
        }
    }
}
