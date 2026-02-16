// Window Settings menu — resolution, display mode, VSync runtime control.
//
// Allows in-game adjustment of window settings through the ModMenu.
// Changes are applied to the Bevy window and persisted to config by
// `apply_window_settings_system` in the brs crate.

use bms_config::{Config, DisplayMode, Resolution};

const DISPLAY_MODES: &[DisplayMode] = &[
    DisplayMode::Window,
    DisplayMode::Fullscreen,
    DisplayMode::Borderless,
];

const RESOLUTIONS: &[Resolution] = &[
    Resolution::Sd,
    Resolution::Svga,
    Resolution::Xga,
    Resolution::Hd,
    Resolution::Quadvga,
    Resolution::Fwxga,
    Resolution::Sxgaplus,
    Resolution::Hdplus,
    Resolution::Uxga,
    Resolution::Wsxgaplus,
    Resolution::Fullhd,
    Resolution::Wuxga,
    Resolution::Qxga,
    Resolution::Wqhd,
    Resolution::Ultrahd,
];

#[derive(Debug, Clone)]
pub struct WindowSettingsState {
    pub selected_resolution: Resolution,
    pub selected_display_mode: DisplayMode,
    pub use_resolution: bool,
    pub window_width: i32,
    pub window_height: i32,
    pub vsync: bool,
    pub pending_apply: bool,
}

impl Default for WindowSettingsState {
    fn default() -> Self {
        let config = Config::default();
        Self {
            selected_resolution: config.resolution,
            selected_display_mode: config.displaymode,
            use_resolution: config.use_resolution,
            window_width: config.window_width,
            window_height: config.window_height,
            vsync: config.vsync,
            pending_apply: false,
        }
    }
}

impl WindowSettingsState {
    pub fn load_from_config(&mut self, config: &Config) {
        self.selected_resolution = config.resolution;
        self.selected_display_mode = config.displaymode;
        self.use_resolution = config.use_resolution;
        self.window_width = config.window_width;
        self.window_height = config.window_height;
        self.vsync = config.vsync;
        self.pending_apply = false;
    }
}

pub fn render(ctx: &egui::Context, open: &mut bool, state: &mut WindowSettingsState) {
    egui::Window::new("Window Settings")
        .open(open)
        .resizable(false)
        .show(ctx, |ui| {
            // Display mode
            egui::ComboBox::from_label("Display Mode")
                .selected_text(format!("{:?}", state.selected_display_mode))
                .show_ui(ui, |ui| {
                    for &mode in DISPLAY_MODES {
                        ui.selectable_value(
                            &mut state.selected_display_mode,
                            mode,
                            format!("{mode:?}"),
                        );
                    }
                });

            ui.separator();

            // Resolution
            ui.checkbox(&mut state.use_resolution, "Use preset resolution");

            if state.use_resolution {
                egui::ComboBox::from_label("Resolution")
                    .selected_text(state.selected_resolution.to_string())
                    .show_ui(ui, |ui| {
                        for &res in RESOLUTIONS {
                            ui.selectable_value(
                                &mut state.selected_resolution,
                                res,
                                res.to_string(),
                            );
                        }
                    });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(
                        egui::DragValue::new(&mut state.window_width)
                            .range(Resolution::Sd.width()..=Resolution::Ultrahd.width()),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    ui.add(
                        egui::DragValue::new(&mut state.window_height)
                            .range(Resolution::Sd.height()..=Resolution::Ultrahd.height()),
                    );
                });
            }

            ui.separator();

            ui.checkbox(&mut state.vsync, "VSync");

            ui.separator();

            if ui.button("Apply").clicked() {
                state.pending_apply = true;
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pending_apply_is_false() {
        let state = WindowSettingsState::default();
        assert!(!state.pending_apply);
    }

    #[test]
    fn load_from_config() {
        let mut config = Config::default();
        config.resolution = Resolution::Fullhd;
        config.displaymode = DisplayMode::Fullscreen;
        config.use_resolution = true;
        config.window_width = 1920;
        config.window_height = 1080;
        config.vsync = true;

        let mut state = WindowSettingsState::default();
        state.load_from_config(&config);

        assert_eq!(state.selected_resolution, Resolution::Fullhd);
        assert_eq!(state.selected_display_mode, DisplayMode::Fullscreen);
        assert!(state.use_resolution);
        assert_eq!(state.window_width, 1920);
        assert_eq!(state.window_height, 1080);
        assert!(state.vsync);
        assert!(!state.pending_apply);
    }

    #[test]
    fn load_from_config_resets_pending_apply() {
        let config = Config::default();
        let mut state = WindowSettingsState::default();
        state.pending_apply = true;
        state.load_from_config(&config);
        assert!(!state.pending_apply);
    }
}
