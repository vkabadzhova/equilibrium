mod fluid;
use equilibrium::fluid::{Fluid, FluidConfig, SimulationConfig};
use equilibrium::renderer::Renderer;

fn main() {
    let fluid_config = FluidConfig::default();

    let simulation_config = SimulationConfig::default();

    let mut fluid = Fluid::new(fluid_config, simulation_config);

    let mut renderer = Renderer::new();
    renderer.simulate(&mut fluid);
}
