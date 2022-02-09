use super::fluid_configs_menu::FluidUiSettings;
use super::simulation_configs_menu::SimulationUiSettings;
use super::Setting;
use eframe::egui;
use std::collections::BTreeSet;

/// Enum describing the various widgets' types. This is what unifies all the widgets
/// and is used fot storing them in collections.
pub enum SettingType {
    /// Used for describing the [`FluidUiSettings`] type
    Fluid(FluidUiSettings),
    /// Used for describing the [`SimulationUiSettings`] type
    Simulation(SimulationUiSettings),
}

impl Setting for SettingType {
    fn name(&self) -> &'static str {
        match self {
            SettingType::Fluid(fluid_ui_setting) => fluid_ui_setting.name(),
            SettingType::Simulation(simulation_ui_setting) => simulation_ui_setting.name(),
        }
    }

    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool) {
        match self {
            SettingType::Fluid(fluid_ui_setting) => fluid_ui_setting.show(ctx, open),
            SettingType::Simulation(simulation_ui_setting) => simulation_ui_setting.show(ctx, open),
        };
    }
}

/// Main window with settings from which the other menus can be pulled out via checkboxes
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct SettingsMenu {
    /// a collection of all settings widgets with checkboxes
    #[cfg_attr(feature = "serde", serde(skip))]
    pub settings_menu: Vec<SettingType>,

    open: BTreeSet<String>,
}

impl Default for SettingsMenu {
    fn default() -> Self {
        Self::from_settings(vec![
            SettingType::Simulation(
                super::simulation_configs_menu::SimulationUiSettings::default(),
            ),
            SettingType::Fluid(super::fluid_configs_menu::FluidUiSettings::default()),
        ])
    }
}

impl SettingsMenu {
    /// The starting window
    pub fn from_settings(settings_menu: Vec<SettingType>) -> Self {
        let mut open = BTreeSet::new();
        open.insert(
            super::simulation_configs_menu::SimulationUiSettings::default()
                .name()
                .to_owned(),
        );

        Self {
            settings_menu,
            open,
        }
    }

    /// Make checkboxes
    pub fn checkboxes(&mut self, ui: &mut egui::Ui) {
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
    pub fn windows(&mut self, ctx: &egui::CtxRef) {
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
