// Headless rendering harness for screenshot tests.
//
// Uses Bevy 0.15 with a hidden window + Screenshot component + save_to_disk
// observer to capture frames.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use bms_render::font_map::FontMap;
use bms_render::skin_renderer::{setup_skin, skin_render_system};
use bms_render::state_provider::SkinStateProvider;
use bms_render::texture_map::TextureMap;
use bms_skin::image_handle::ImageHandle;
use bms_skin::skin::Skin;

use std::path::Path;

/// Render test harness for capturing headless screenshots.
pub struct RenderTestHarness {
    app: App,
    next_handle_id: u32,
}

impl RenderTestHarness {
    /// Creates a new rendering harness with a hidden window.
    pub fn new(width: u32, height: u32) -> Self {
        let mut app = App::new();

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (width as f32, height as f32).into(),
                        visible: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..default()
                }),
        );

        app.add_systems(Update, skin_render_system);

        Self {
            app,
            next_handle_id: 0,
        }
    }

    /// Upload an RGBA image into Bevy's asset system and return an ImageHandle.
    pub fn upload_image(&mut self, rgba: &image::RgbaImage) -> ImageHandle {
        let id = self.next_handle_id;
        self.next_handle_id += 1;

        let size = bevy::render::render_resource::Extent3d {
            width: rgba.width(),
            height: rgba.height(),
            depth_or_array_layers: 1,
        };
        let bevy_image = Image::new(
            size,
            bevy::render::render_resource::TextureDimension::D2,
            rgba.as_raw().clone(),
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
                | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
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

        ImageHandle(id)
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

        // Spawn camera
        self.app.world_mut().spawn(Camera2d);

        // Set up skin entities and resource
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

        // Spawn a Screenshot entity with save_to_disk observer
        self.app
            .world_mut()
            .commands()
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(output_path.to_path_buf()));
        self.app.world_mut().flush();

        // Run frames to complete capture and disk write
        for _ in 0..6 {
            self.app.update();
        }
    }
}

/// Temporary resource to hold uploaded textures before setup_skin is called.
#[derive(Resource, Default)]
struct PendingTextures {
    entries: Vec<(ImageHandle, Handle<Image>, f32, f32)>,
}
