// Headless rendering harness for screenshot tests.
//
// Uses Bevy 0.15 without WinitPlugin, rendering to an off-screen Image.
// This avoids the macOS requirement that EventLoop must be on the main thread.

use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::sprite::Material2dPlugin;

use bms_render::distance_field_material::DistanceFieldMaterial;
use bms_render::font_map::FontMap;
use bms_render::skin_renderer::{setup_skin, skin_render_system};
use bms_render::state_provider::SkinStateProvider;
use bms_render::texture_map::TextureMap;
use bms_skin::image_handle::ImageHandle;
use bms_skin::skin::Skin;

use std::path::Path;

/// Handle to the off-screen render target image.
#[derive(Resource, Clone)]
struct RenderTargetHandle(Handle<Image>);

/// Temporary resource to hold uploaded textures before setup_skin is called.
#[derive(Resource, Default)]
struct PendingTextures {
    entries: Vec<(ImageHandle, Handle<Image>, f32, f32)>,
}

/// Render test harness for capturing headless screenshots.
pub struct RenderTestHarness {
    app: App,
    next_handle_id: u32,
}

impl RenderTestHarness {
    /// Creates a new headless rendering harness.
    pub fn new(width: u32, height: u32) -> Self {
        let mut app = App::new();

        // Use DefaultPlugins but disable WinitPlugin to avoid EventLoop creation.
        // On macOS, winit requires EventLoop on the main thread, but tests run
        // on worker threads. We render to an off-screen Image instead.
        app.add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::winit::WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..default()
                }),
        );

        // Register DistanceFieldMaterial (required by skin_render_system parameter)
        app.add_plugins(Material2dPlugin::<DistanceFieldMaterial>::default());
        app.add_systems(Update, skin_render_system);

        // Finalize all plugins — this calls Plugin::finish() on every registered
        // plugin, which inserts critical resources like CapturedScreenshots.
        // Without this, app.update() alone will panic on missing resources.
        app.finish();
        app.cleanup();

        // Create off-screen render target image
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let mut render_image = Image::new(
            size,
            TextureDimension::D2,
            vec![0u8; (width * height * 4) as usize],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        render_image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::COPY_SRC
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING;

        let image_handle = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(render_image);

        // Spawn camera targeting the off-screen image
        app.world_mut().spawn((
            Camera2d,
            Camera {
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
        ));

        app.world_mut()
            .insert_resource(RenderTargetHandle(image_handle));

        Self {
            app,
            next_handle_id: 0,
        }
    }

    /// Upload an RGBA image into Bevy's asset system.
    pub fn upload_image(&mut self, rgba: &image::RgbaImage) {
        let id = self.next_handle_id;
        self.next_handle_id += 1;

        let size = Extent3d {
            width: rgba.width(),
            height: rgba.height(),
            depth_or_array_layers: 1,
        };
        let bevy_image = Image::new(
            size,
            TextureDimension::D2,
            rgba.as_raw().clone(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        let handle = self
            .app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(bevy_image);

        if !self.app.world().contains_resource::<PendingTextures>() {
            self.app
                .world_mut()
                .insert_resource(PendingTextures::default());
        }
        self.app
            .world_mut()
            .resource_mut::<PendingTextures>()
            .entries
            .push((
                ImageHandle(id),
                handle,
                rgba.width() as f32,
                rgba.height() as f32,
            ));
    }

    /// Set up the skin with the given skin data and state provider.
    pub fn setup_skin(&mut self, skin: Skin, state_provider: Box<dyn SkinStateProvider>) {
        let mut texture_map = TextureMap::new();
        if let Some(pending) = self.app.world_mut().remove_resource::<PendingTextures>() {
            for (img_handle, bevy_handle, w, h) in pending.entries {
                texture_map.insert(img_handle, bevy_handle, w, h);
            }
        }

        let font_map = FontMap::new();
        let mut commands = self.app.world_mut().commands();
        setup_skin(&mut commands, skin, texture_map, font_map, state_provider);
        self.app.world_mut().flush();
    }

    /// Run pre-roll frames, capture a screenshot, and save to disk.
    pub fn capture_frame(&mut self, output_path: &Path) {
        // Run pre-roll frames to let rendering pipeline stabilize
        for _ in 0..6 {
            self.app.update();
        }

        // Capture from the off-screen render target
        let handle = self.app.world().resource::<RenderTargetHandle>().0.clone();
        self.app
            .world_mut()
            .commands()
            .spawn(Screenshot::image(handle))
            .observe(save_to_disk(output_path.to_path_buf()));
        self.app.world_mut().flush();

        // Run frames to complete capture and disk write
        for _ in 0..6 {
            self.app.update();
        }
    }
}
