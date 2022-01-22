use equilibrium::app::app::App;
use equilibrium::simulation::configs::{FluidConfigs, SimulationConfigs};
use equilibrium::simulation::fluid::Fluid;
use equilibrium::simulation::renderer::Renderer;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Error,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("../logs.log").unwrap(),
        ),
    ])
    .unwrap();

    let fluid_configs = FluidConfigs::default();

    let simulation_configs = SimulationConfigs::default();

    let fluid = Fluid::new(fluid_configs, simulation_configs);
    let renderer = Renderer::new(fluid);

    let app = App::new(renderer);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
