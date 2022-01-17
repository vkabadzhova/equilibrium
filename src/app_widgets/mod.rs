/// The main menu window from which other settings can be pulled out via checkboxes
pub mod widgets_menu;
/// Menu with simulation settings such as number of frames, velocity of simulation, etc.
pub mod simulation_configs_menu;
/// Menu with settings for the fluid
pub mod fluid_configs_menu;

use eframe::egui;

/// Something to view in the settings windows
pub trait View {
    /// The "window" of the widget which one can move around
    fn ui(&mut self, ui: &mut egui::Ui);
}


/// Something to view
pub trait Setting {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool);
}
