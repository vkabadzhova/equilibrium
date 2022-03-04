use super::fluid_configs_menu::FluidUiSettings;
use super::simulation_configs_menu::SimulationUiSettings;
use super::viewport_configs_menu::ViewportUiSettings;
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
    /// Used for describing the [`ViewportUiSettings`] type
    Viewport(ViewportUiSettings),
}

impl Setting for SettingType {
    fn name(&self) -> &'static str {
        match self {
            SettingType::Fluid(fluid_ui_setting) => fluid_ui_setting.name(),
            SettingType::Simulation(simulation_ui_setting) => simulation_ui_setting.name(),
            SettingType::Viewport(viewport_ui_setting) => viewport_ui_setting.name(),
        }
    }

    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool) {
        match self {
            SettingType::Fluid(fluid_ui_setting) => fluid_ui_setting.show(ctx, open),
            SettingType::Simulation(simulation_ui_setting) => simulation_ui_setting.show(ctx, open),
            SettingType::Viewport(viewport_ui_setting) => viewport_ui_setting.show(ctx, open),
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
        Self::from_settings(
            vec![
                SettingType::Simulation(
                    super::simulation_configs_menu::SimulationUiSettings::default(),
                ),
                SettingType::Fluid(super::fluid_configs_menu::FluidUiSettings::default()),
                SettingType::Viewport(super::viewport_configs_menu::ViewportUiSettings::default()),
            ],
            true,
        )
    }
}

impl SettingsMenu {
    /// Configure the GUI
    ///
    /// The function creates [`SettingsMenu`] from given vector of checkbox menus which
    /// are located in the most right part of the application (see the application interface
    /// and the picture below)
    ///
    /// * `settings_menu` - collection of all the menus
    /// * `should_open_first` - the first setting can be automatically opened every
    /// time the application is started. This parameter defines if this should be the case.
    ///
    /// <a href="https://imgbb.com/"><img src="https://i.ibb.co/0qNLXV0/checkbox-menus.png" alt="checkbox-menus" border="0"></a><br />
    pub fn from_settings(settings_menu: Vec<SettingType>, should_open_first: bool) -> Self {
        let mut open = BTreeSet::new();
        if should_open_first {
            open.insert(
                super::simulation_configs_menu::SimulationUiSettings::default()
                    .name()
                    .to_owned(),
            );
        }

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

#[cfg(test)]
mod tests {
    use crate::app::widgets::widgets_menu::FluidUiSettings;
    use crate::app::widgets::widgets_menu::SettingType;
    use crate::app::widgets::Setting;

    #[test]
    fn settingtype_name_works() {
        let fluid_ui_setting = FluidUiSettings::default();
        let fluid_setting_type = SettingType::Fluid(fluid_ui_setting);
        assert_eq!(fluid_setting_type.name(), fluid_ui_setting.name());
    }
}
