use equilibrium::app::app::App;
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
            File::create("logs.log").expect("Couldn't load logger"),
        ),
    ])
    .expect("Logger failed to load");

    let renderer = Renderer::default();

    let app = App::new(renderer);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
