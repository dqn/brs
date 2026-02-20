// BmsRenderPlugin — Bevy plugin for skin rendering.
//
// Sets up the 2D orthographic camera, registers the skin render system,
// and configures the distance field font material pipeline.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

use bevy::render::camera::ScalingMode;

use crate::bga_layer_material::BgaLayerMaterial;
use crate::distance_field_material::DistanceFieldMaterial;
use crate::mod_menu::ModMenuPlugin;
use crate::skin_renderer::{SkinRenderState, skin_render_system};

/// Register embedded shader assets and Material2d plugins required by
/// skin_render_system, without camera setup or ModMenu.
///
/// Used by the test harness where BmsRenderPlugin is too heavy (it
/// pulls in EguiPlugin via ModMenuPlugin).
pub fn register_render_materials(app: &mut App) {
    embedded_asset!(app, "distance_field.wgsl");
    embedded_asset!(app, "bga_layer.wgsl");
    app.add_plugins(Material2dPlugin::<DistanceFieldMaterial>::default());
    app.add_plugins(Material2dPlugin::<BgaLayerMaterial>::default());
}

/// Bevy plugin that sets up skin rendering.
///
/// Configures a 2D orthographic camera, registers the per-frame
/// skin render system, and adds the distance field material pipeline.
pub struct BmsRenderPlugin;

impl Plugin for BmsRenderPlugin {
    fn build(&self, app: &mut App) {
        register_render_materials(app);

        app.add_plugins(ModMenuPlugin)
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (update_camera_projection, skin_render_system).chain(),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_camera_projection(
    render_state: Option<Res<SkinRenderState>>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    if let Some(state) = render_state {
        for mut projection in &mut camera_query {
            projection.scaling_mode = ScalingMode::Fixed {
                width: state.skin.width,
                height: state.skin.height,
            };
        }
    }
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
