mod fluid;
use crate::fluid::{Fluid, FluidConfig, SimulationConfig};

fn main() {
    let fluid_config = FluidConfig::default();

    let simulation_config = SimulationConfig::default();

    let mut fluid = Fluid::new(fluid_config, simulation_config);
    fluid.simulate();
}
