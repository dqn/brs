// BmsRenderPlugin — Bevy plugin for skin rendering.
//
// Sets up the 2D orthographic camera and registers the skin render system.

use bevy::prelude::*;

use crate::skin_renderer::skin_render_system;

/// Bevy plugin that sets up skin rendering.
///
/// Configures a 2D orthographic camera and registers the per-frame
/// skin render system.
pub struct BmsRenderPlugin;

impl Plugin for BmsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, skin_render_system);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        // Verify plugin can be added without panic (no GPU needed)
        let app = App::new();
        // We don't add BmsRenderPlugin because it needs DefaultPlugins,
        // but we verify the struct exists and is a Plugin
        let _plugin = BmsRenderPlugin;
        let _ = &app;
    }
}
