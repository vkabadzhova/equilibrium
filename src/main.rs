mod fluid;
use crate::fluid::{Fluid, FluidConfig, SimulationConfig};

fn main() {
    let fluid_config = FluidConfig {
        ..Default::default()
    };

    let simulation_config = SimulationConfig {
        ..Default::default()
    };

    let mut fluid = Fluid::new(fluid_config, simulation_config);
    fluid.simulate();
    /*
    let mut simulator: fluid::FluidSimulator = fluid::FluidSimulator {
        ..Default::default()
    };
    simulator.fluid_simulation();
    println!("Hello world!");
    */
}
