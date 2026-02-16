// Runtime window management: fullscreen toggle (F6), startup monitor selection,
// and ModMenu window settings apply.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::{Monitor, MonitorSelection, PresentMode, WindowMode, WindowPosition};
use bms_config::DisplayMode;
use tracing::{info, warn};

use crate::{BrsConfig, StateUiResources};

/// Format a monitor name in Java-compatible format: "Name [x, y]".
pub fn format_monitor_name(monitor: &Monitor) -> String {
    let name = monitor.name.as_deref().unwrap_or("Unknown");
    let pos = monitor.physical_position;
    format!("{name} [{}, {}]", pos.x, pos.y)
}

/// Resolve a config `monitor_name` string to a Bevy `MonitorSelection`.
///
/// Matching priority:
/// 1. Empty string -> `MonitorSelection::Current`
/// 2. Exact format match ("Name [x, y]") -> `MonitorSelection::Entity`
/// 3. Name-only prefix match (fallback) -> `MonitorSelection::Entity`
/// 4. No match -> `MonitorSelection::Current` + warn log
pub fn resolve_monitor_selection(
    monitor_name: &str,
    monitors: &Query<(Entity, &Monitor)>,
) -> MonitorSelection {
    if monitor_name.is_empty() {
        return MonitorSelection::Current;
    }

    // Exact format match
    for (entity, monitor) in monitors.iter() {
        if format_monitor_name(monitor) == monitor_name {
            return MonitorSelection::Entity(entity);
        }
    }

    // Name-only fallback: match just the monitor name portion
    for (entity, monitor) in monitors.iter() {
        if monitor.name.as_deref() == Some(monitor_name) {
            return MonitorSelection::Entity(entity);
        }
    }

    warn!(
        "Monitor '{}' not found, using current monitor",
        monitor_name
    );
    MonitorSelection::Current
}

/// Convert `DisplayMode` config value to Bevy `WindowMode`.
pub fn display_mode_to_window_mode(mode: DisplayMode, monitor: MonitorSelection) -> WindowMode {
    match mode {
        DisplayMode::Fullscreen => WindowMode::Fullscreen(monitor),
        DisplayMode::Borderless => WindowMode::BorderlessFullscreen(monitor),
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

/// PostStartup system: apply monitor selection after Bevy creates monitor entities.
///
/// On startup, the initial window uses `MonitorSelection::Current` because monitor
/// entities don't exist yet. This system runs once after startup to resolve the
/// configured monitor and update the window accordingly.
pub fn apply_monitor_selection_system(
    config: Res<BrsConfig>,
    monitors: Query<(Entity, &Monitor)>,
    mut windows: Query<&mut Window>,
) {
    if config.0.monitor_name.is_empty() {
        return;
    }

    let selection = resolve_monitor_selection(&config.0.monitor_name, &monitors);

    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    match config.0.displaymode {
        DisplayMode::Fullscreen | DisplayMode::Borderless => {
            window.mode = display_mode_to_window_mode(config.0.displaymode, selection);
            info!(
                monitor = %config.0.monitor_name,
                "Applied monitor selection for fullscreen/borderless"
            );
        }
        DisplayMode::Window => {
            window.position = WindowPosition::Centered(selection);
            info!(
                monitor = %config.0.monitor_name,
                "Centered window on selected monitor"
            );
        }
    }
}

/// Update system: apply window settings from ModMenu when pending.
pub fn apply_window_settings_system(
    mut mod_menu: ResMut<bms_render::mod_menu::ModMenuState>,
    mut windows: Query<&mut Window>,
    mut config: ResMut<BrsConfig>,
    monitors: Query<(Entity, &Monitor)>,
    ui_res: Res<StateUiResources>,
) {
    if !mod_menu.window_settings.pending_apply {
        return;
    }
    mod_menu.window_settings.pending_apply = false;

    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    let ws = &mod_menu.window_settings;
    let selection = resolve_monitor_selection(&config.0.monitor_name, &monitors);

    // Apply display mode
    window.mode = display_mode_to_window_mode(ws.selected_display_mode, selection);

    // Apply resolution (only meaningful for windowed mode)
    if ws.selected_display_mode == DisplayMode::Window {
        let (w, h) = if ws.use_resolution {
            (
                ws.selected_resolution.width(),
                ws.selected_resolution.height(),
            )
        } else {
            (ws.window_width, ws.window_height)
        };
        window.resolution = bevy::window::WindowResolution::new(w as f32, h as f32);
    }

    // Apply VSync
    window.present_mode = vsync_to_present_mode(ws.vsync);

    // Persist to config
    config.0.displaymode = ws.selected_display_mode;
    config.0.resolution = ws.selected_resolution;
    config.0.use_resolution = ws.use_resolution;
    let (w, h) = if ws.use_resolution {
        (
            ws.selected_resolution.width(),
            ws.selected_resolution.height(),
        )
    } else {
        (ws.window_width, ws.window_height)
    };
    config.0.window_width = w;
    config.0.window_height = h;
    config.0.vsync = ws.vsync;

    if let Err(e) = config.0.write(&ui_res.config_paths.config) {
        warn!("Failed to save config after window settings change: {e}");
    } else {
        info!(
            mode = ?ws.selected_display_mode,
            resolution = ?ws.selected_resolution,
            vsync = ws.vsync,
            "Applied window settings from ModMenu"
        );
    }
}

/// Bevy system: F6 toggles fullscreen <-> windowed mode.
///
/// Skips when Alt is held (Java compatibility) or when egui has keyboard focus.
pub fn window_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut config: ResMut<BrsConfig>,
    mod_menu: Res<bms_render::mod_menu::ModMenuState>,
    ui_res: Res<StateUiResources>,
    monitors: Query<(Entity, &Monitor)>,
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

    let selection = resolve_monitor_selection(&config.0.monitor_name, &monitors);

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

    window.mode = display_mode_to_window_mode(new_mode, selection);

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
        warn!("Failed to save config after screen mode switch: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode_to_window_mode_window() {
        assert_eq!(
            display_mode_to_window_mode(DisplayMode::Window, MonitorSelection::Current),
            WindowMode::Windowed,
        );
    }

    #[test]
    fn test_display_mode_to_window_mode_fullscreen() {
        assert_eq!(
            display_mode_to_window_mode(DisplayMode::Fullscreen, MonitorSelection::Current),
            WindowMode::Fullscreen(MonitorSelection::Current),
        );
    }

    #[test]
    fn test_display_mode_to_window_mode_borderless() {
        assert_eq!(
            display_mode_to_window_mode(DisplayMode::Borderless, MonitorSelection::Current),
            WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        );
    }

    #[test]
    fn test_display_mode_to_window_mode_with_primary() {
        assert_eq!(
            display_mode_to_window_mode(DisplayMode::Fullscreen, MonitorSelection::Primary),
            WindowMode::Fullscreen(MonitorSelection::Primary),
        );
    }

    #[test]
    fn test_vsync_to_present_mode() {
        assert_eq!(vsync_to_present_mode(true), PresentMode::AutoVsync);
        assert_eq!(vsync_to_present_mode(false), PresentMode::AutoNoVsync);
    }

    #[test]
    fn test_format_monitor_name() {
        let monitor = Monitor {
            name: Some("HDMI-1".to_string()),
            physical_position: IVec2::new(0, 0),
            physical_width: 1920,
            physical_height: 1080,
            refresh_rate_millihertz: Some(60000),
            scale_factor: 1.0,
            video_modes: vec![],
        };
        assert_eq!(format_monitor_name(&monitor), "HDMI-1 [0, 0]");
    }

    #[test]
    fn test_format_monitor_name_with_offset() {
        let monitor = Monitor {
            name: Some("DP-1".to_string()),
            physical_position: IVec2::new(1920, 0),
            physical_width: 2560,
            physical_height: 1440,
            refresh_rate_millihertz: Some(144000),
            scale_factor: 1.0,
            video_modes: vec![],
        };
        assert_eq!(format_monitor_name(&monitor), "DP-1 [1920, 0]");
    }

    #[test]
    fn test_format_monitor_name_unknown() {
        let monitor = Monitor {
            name: None,
            physical_position: IVec2::new(0, 0),
            physical_width: 1920,
            physical_height: 1080,
            refresh_rate_millihertz: None,
            scale_factor: 1.0,
            video_modes: vec![],
        };
        assert_eq!(format_monitor_name(&monitor), "Unknown [0, 0]");
    }
}
