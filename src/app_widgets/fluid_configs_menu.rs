use crate::app_widgets::Setting;
use eframe::egui;

/// Menu with settings for the fluid
pub struct FluidConfigsMenu {}

impl Default for FluidConfigsMenu {
    fn default() -> Self {
        Self {}
    }
}

impl Setting for FluidConfigsMenu {
    fn name(&self) -> &'static str {
        "Fluid Settings"
    }

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool) {}
}
