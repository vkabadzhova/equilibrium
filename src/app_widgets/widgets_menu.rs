use super::Setting;
use eframe::egui::{CtxRef, Ui};
use std::collections::BTreeSet;

// ----------------------------------------------------------------------------

/// Main window with settings from which the other menus can be pulled out via checkboxes
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct SettingsMenu {
    #[cfg_attr(feature = "serde", serde(skip))]
    settings_menu: Vec<Box<dyn Setting>>,

    open: BTreeSet<String>,
}

impl Default for SettingsMenu {
    fn default() -> Self {
        Self::from_settings(vec![
            Box::new(super::simulation_configs_menu::SimulationConfigsMenu::default()),
            Box::new(super::fluid_configs_menu::FluidConfigsMenu::default()),
        ])
    }
}

impl SettingsMenu {
    /// The starting window
    pub fn from_settings(settings_menu: Vec<Box<dyn Setting>>) -> Self {
        let mut open = BTreeSet::new();
        open.insert(
            super::simulation_configs_menu::SimulationConfigsMenu::default()
                .name()
                .to_owned(),
        );

        Self {
            settings_menu,
            open,
        }
    }

    /// Make checkboxes
    pub fn checkboxes(&mut self, ui: &mut Ui) {
        let Self {
            settings_menu,
            open,
        } = self;
        for setting in settings_menu {
            let mut is_open = open.contains(setting.name());
            ui.checkbox(&mut is_open, setting.name());
            set_open(open, setting.name(), is_open);
        }
    }

    /// Open a window
    pub fn windows(&mut self, ctx: &CtxRef) {
        let Self {
            settings_menu,
            open,
        } = self;
        for setting in settings_menu {
            let mut is_open = open.contains(setting.name());
            setting.show(ctx, &mut is_open);
            set_open(open, setting.name(), is_open);
        }
    }
}

fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

fn show_menu_bar(ui: &mut Ui) {
    trace!(ui);
    use eframe::egui::*;

    menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Organize windows").clicked() {
                ui.ctx().memory().reset_areas();
                ui.close_menu();
            }
            if ui
                .button("Reset egui memory")
                .on_hover_text("Forget scroll, positions, sizes etc")
                .clicked()
            {
                *ui.ctx().memory() = Default::default();
                ui.close_menu();
            }
        });
    });
}
