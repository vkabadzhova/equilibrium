mod fluid;
use equilibrium::app::App;
use equilibrium::fluid::{Fluid, FluidConfig, SimulationConfig};
use equilibrium::renderer::Renderer;

fn main() {
    let fluid_config = FluidConfig::default();

    let simulation_config = SimulationConfig::default();

    let mut fluid = Fluid::new(fluid_config, simulation_config);
    let mut renderer = Renderer::new(fluid);

    let app = App::new(renderer);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
