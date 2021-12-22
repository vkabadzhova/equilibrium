mod fluid;

fn main() {
    let fluid_config = FluidConfig {
        ..Default::default()
    }

    let simulation_config = SimulationConfig {
        ..Default::default()
    }

    let mut fluid = Fluid::new();

    let mut simulator: fluid::FluidSimulator = fluid::FluidSimulator {
        ..Default::default()
    };
    simulator.fluid_simulation();
    println!("Hello world!");
}
