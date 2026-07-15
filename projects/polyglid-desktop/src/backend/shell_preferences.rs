use polyglid_core::services::SettingsService;
use polyglid_core::store::WorkspaceStore;

use super::DesktopBackend;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShellPreferences {
    pub(crate) sidebar_visible: bool,
    pub(crate) bottom_panel_visible: bool,
    pub(crate) sidebar_width: f64,
    pub(crate) bottom_panel_height: f64,
}

impl Default for ShellPreferences {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            bottom_panel_visible: true,
            sidebar_width: 280.0,
            bottom_panel_height: 210.0,
        }
    }
}

impl DesktopBackend {
    pub(crate) fn save_shell_preferences(&self, value: &ShellPreferences) -> Result<(), String> {
        let settings = self.settings()?;
        settings.set_setting("ui.sidebar_visible", bool_text(value.sidebar_visible))?;
        settings.set_setting(
            "ui.bottom_panel_visible",
            bool_text(value.bottom_panel_visible),
        )?;
        settings.set_setting("ui.sidebar_width", &value.sidebar_width.round().to_string())?;
        settings.set_setting(
            "ui.bottom_panel_height",
            &value.bottom_panel_height.round().to_string(),
        )
    }

    fn settings(&self) -> Result<SettingsService, String> {
        WorkspaceStore::new(&self.database_path).map(SettingsService::new)
    }

    pub(super) fn load_shell_preferences(&self) -> Result<ShellPreferences, String> {
        let settings = self.settings()?;
        let defaults = ShellPreferences::default();
        Ok(ShellPreferences {
            sidebar_visible: setting_bool(
                &settings,
                "ui.sidebar_visible",
                defaults.sidebar_visible,
            )?,
            bottom_panel_visible: setting_bool(
                &settings,
                "ui.bottom_panel_visible",
                defaults.bottom_panel_visible,
            )?,
            sidebar_width: setting_number(
                &settings,
                "ui.sidebar_width",
                defaults.sidebar_width,
                180.0,
                480.0,
            )?,
            bottom_panel_height: setting_number(
                &settings,
                "ui.bottom_panel_height",
                defaults.bottom_panel_height,
                120.0,
                520.0,
            )?,
        })
    }
}

fn setting_bool(service: &SettingsService, key: &str, fallback: bool) -> Result<bool, String> {
    Ok(service
        .get_setting(key)?
        .map_or(fallback, |value| value == "true"))
}

fn setting_number(
    service: &SettingsService,
    key: &str,
    fallback: f64,
    minimum: f64,
    maximum: f64,
) -> Result<f64, String> {
    Ok(service
        .get_setting(key)?
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(fallback)
        .clamp(minimum, maximum))
}

fn bool_text(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}
