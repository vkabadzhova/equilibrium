mod fluid;
use equilibrium::app::App;
use equilibrium::fluid::{Fluid, FluidConfig, SimulationConfig};

fn main() {
    let fluid_config = FluidConfig::default();

    let simulation_config = SimulationConfig::default();

    let mut fluid = Fluid::new(fluid_config, simulation_config);
    /*
        let mut renderer = Renderer::new();
        renderer.start(&mut fluid);
    */
    let app = App::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
