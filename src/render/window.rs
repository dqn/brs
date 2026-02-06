use anyhow::{Result, anyhow};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// Configuration for window creation.
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "brs".to_string(),
            width: 1280,
            height: 720,
            resizable: false,
        }
    }
}

/// Manages the application window and event loop.
pub struct WindowManager {
    window: Window,
}

impl WindowManager {
    /// Create a new window with the given configuration.
    /// Must be called on the main thread.
    pub fn new(event_loop: &ActiveEventLoop, config: &WindowConfig) -> Result<Self> {
        let attrs = Window::default_attributes()
            .with_title(&config.title)
            .with_inner_size(PhysicalSize::new(config.width, config.height))
            .with_resizable(config.resizable);

        let window = event_loop
            .create_window(attrs)
            .map_err(|e| anyhow!("failed to create window: {e}"))?;

        Ok(Self { window })
    }

    /// Get a reference to the underlying window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get current window inner size.
    pub fn inner_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }
}

/// Application handler that bridges winit events to the game loop.
/// Users should implement GameLoop and pass it to run_app.
pub trait GameLoop {
    /// Called once when the window is created and GPU is ready.
    fn init(&mut self, window: &Window) -> Result<()>;
    /// Called each frame to update game state.
    fn update(&mut self);
    /// Called each frame to render.
    fn render(&mut self) -> Result<()>;
    /// Called when the window should close. Return true to allow closing.
    fn should_close(&self) -> bool;
    /// Called on window resize.
    fn on_resize(&mut self, width: u32, height: u32);
    /// Called for input events.
    fn on_input(&mut self, event: &WindowEvent);
}

/// Run the application with a winit event loop.
pub fn run_app<G: GameLoop + 'static>(config: WindowConfig, game: G) -> Result<()> {
    let event_loop = EventLoop::new().map_err(|e| anyhow!("failed to create event loop: {e}"))?;

    struct App<G: GameLoop> {
        game: G,
        config: WindowConfig,
        window: Option<Window>,
    }

    impl<G: GameLoop> ApplicationHandler for App<G> {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_none() {
                let attrs = Window::default_attributes()
                    .with_title(&self.config.title)
                    .with_inner_size(PhysicalSize::new(self.config.width, self.config.height))
                    .with_resizable(self.config.resizable);

                match event_loop.create_window(attrs) {
                    Ok(window) => {
                        if let Err(e) = self.game.init(&window) {
                            tracing::error!("failed to initialize game: {e}");
                            event_loop.exit();
                            return;
                        }
                        self.window = Some(window);
                    }
                    Err(e) => {
                        tracing::error!("failed to create window: {e}");
                        event_loop.exit();
                    }
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            event: WindowEvent,
        ) {
            match &event {
                WindowEvent::CloseRequested => {
                    if self.game.should_close() {
                        event_loop.exit();
                    }
                }
                WindowEvent::Resized(size) => {
                    self.game.on_resize(size.width, size.height);
                }
                WindowEvent::RedrawRequested => {
                    self.game.update();
                    if let Err(e) = self.game.render() {
                        tracing::error!("render error: {e}");
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
            self.game.on_input(&event);
        }
    }

    let mut app = App {
        game,
        config,
        window: None,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|e| anyhow!("event loop error: {e}"))?;

    Ok(())
}
