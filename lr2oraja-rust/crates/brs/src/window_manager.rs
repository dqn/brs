// Runtime window management: fullscreen toggle (F6) and startup config.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
use bms_config::DisplayMode;
use tracing::info;

use crate::{BrsConfig, StateUiResources};

/// Convert `DisplayMode` config value to Bevy `WindowMode`.
pub fn display_mode_to_window_mode(mode: DisplayMode) -> WindowMode {
    match mode {
        DisplayMode::Fullscreen => WindowMode::Fullscreen(MonitorSelection::Current),
        DisplayMode::Borderless => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        DisplayMode::Window => WindowMode::Windowed,
    }
}

/// Convert `vsync` config flag to Bevy `PresentMode`.
pub fn vsync_to_present_mode(vsync: bool) -> PresentMode {
    if vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    }
}

/// Bevy system: F6 toggles fullscreen ↔ windowed mode.
///
/// Skips when Alt is held (Java compatibility) or when egui has keyboard focus.
pub fn window_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut config: ResMut<BrsConfig>,
    mod_menu: Res<bms_render::mod_menu::ModMenuState>,
    ui_res: Res<StateUiResources>,
) {
    if mod_menu.wants_keyboard {
        return;
    }

    if !keyboard.just_pressed(KeyCode::F6) {
        return;
    }

    // Skip when Alt is held (Java: MainController.java:732)
    if keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight) {
        return;
    }

    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    let new_mode = match window.mode {
        WindowMode::Windowed => {
            info!("Switching to fullscreen");
            DisplayMode::Fullscreen
        }
        _ => {
            info!("Switching to windowed");
            DisplayMode::Window
        }
    };

    window.mode = display_mode_to_window_mode(new_mode);

    // Restore config resolution when returning to windowed mode
    if new_mode == DisplayMode::Window {
        window.resolution = bevy::window::WindowResolution::new(
            config.0.window_width as f32,
            config.0.window_height as f32,
        );
    }

    // Persist display mode change
    config.0.displaymode = new_mode;
    if let Err(e) = config.0.write(&ui_res.config_paths.config) {
        tracing::warn!("Failed to save config after screen mode switch: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode_to_window_mode_window() {
        assert_eq!(
            display_mode_to_window_mode(DisplayMode::Window),
            WindowMode::Windowed,
        );
    }

    #[test]
    fn test_display_mode_to_window_mode_fullscreen() {
        assert_eq!(
            display_mode_to_window_mode(DisplayMode::Fullscreen),
            WindowMode::Fullscreen(MonitorSelection::Current),
        );
    }

    #[test]
    fn test_display_mode_to_window_mode_borderless() {
        assert_eq!(
            display_mode_to_window_mode(DisplayMode::Borderless),
            WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        );
    }

    #[test]
    fn test_vsync_to_present_mode() {
        assert_eq!(vsync_to_present_mode(true), PresentMode::AutoVsync);
        assert_eq!(vsync_to_present_mode(false), PresentMode::AutoNoVsync);
    }
}
