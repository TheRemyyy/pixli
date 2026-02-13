//! Application and main game loop.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::audio::Audio;
use crate::ecs::World;
use crate::error::{Error, Result};
use crate::input::{Input, MouseButton};
use crate::physics::Physics;
use crate::renderer::Renderer;
use crate::time::Time;
use crate::window::{Window as EngineWindow, WindowConfig};

/// Game state passed to each system every frame.
pub struct GameState<'a> {
    pub world: &'a mut World,
    pub input: &'a Input,
    pub time: &'a Time,
    pub renderer: &'a mut Renderer,
    pub physics: &'a mut Physics,
    pub audio: &'a Audio,
    pub window: &'a EngineWindow,
}

/// System function type
pub type System = Box<dyn FnMut(&mut GameState)>;

/// Startup system function type
pub type StartupSystem = Box<dyn FnOnce(&mut World, &mut Renderer)>;

/// Application builder
pub struct App {
    config: WindowConfig,
    startup_systems: Vec<StartupSystem>,
    systems: Vec<System>,
}

impl App {
    pub fn new() -> Self {
        Self {
            config: WindowConfig::default(),
            startup_systems: Vec::new(),
            systems: Vec::new(),
        }
    }

    /// Set window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    /// Set fullscreen
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.config.fullscreen = fullscreen;
        self
    }

    /// Set vsync
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.config.vsync = vsync;
        self
    }

    /// Add startup system (runs once at start)
    pub fn add_startup_system<F>(mut self, system: F) -> Self
    where
        F: FnOnce(&mut World, &mut Renderer) + 'static,
    {
        self.startup_systems.push(Box::new(system));
        self
    }

    /// Add system (runs every frame)
    pub fn add_system<F>(mut self, system: F) -> Self
    where
        F: FnMut(&mut GameState) + 'static,
    {
        self.systems.push(Box::new(system));
        self
    }

    /// Run the application. Returns `Err` if initialization fails (e.g. no GPU, window creation).
    pub fn run(self) -> Result<()> {
        env_logger::init();

        let event_loop = EventLoop::new().map_err(|e| Error::EventLoop(e.to_string()))?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let init_error = Rc::new(RefCell::new(None));
        let mut app_state = AppState::new(
            self.config,
            self.startup_systems,
            self.systems,
            init_error.clone(),
        );
        let _ = event_loop.run_app(&mut app_state);

        if let Some(e) = init_error.borrow_mut().take() {
            return Err(e);
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal application state.
struct AppState {
    config: WindowConfig,
    startup_systems: Option<Vec<StartupSystem>>,
    systems: Vec<System>,
    init_error: Rc<RefCell<Option<Error>>>,

    // Runtime state
    world: World,
    input: Input,
    time: Time,
    renderer: Renderer,
    physics: Physics,
    audio: Audio,
    engine_window: EngineWindow,

    // GPU state
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,

    last_frame: Instant,
    last_title_update: Instant,
    title_frame_count: u32,
    initialized: bool,
}

impl AppState {
    fn new(
        config: WindowConfig,
        startup_systems: Vec<StartupSystem>,
        systems: Vec<System>,
        init_error: Rc<RefCell<Option<Error>>>,
    ) -> Self {
        Self {
            config,
            startup_systems: Some(startup_systems),
            systems,
            init_error,
            world: World::new(),
            input: Input::new(),
            time: Time::new(),
            renderer: Renderer::new(),
            physics: Physics::new(),
            audio: Audio::new(),
            engine_window: EngineWindow::default(),
            window: None,
            surface: None,
            device: None,
            queue: None,
            surface_config: None,
            last_frame: Instant::now(),
            last_title_update: Instant::now(),
            title_frame_count: 0,
            initialized: false,
        }
    }

    fn set_init_error_and_exit(&self, event_loop: &ActiveEventLoop, err: Error) {
        *self.init_error.borrow_mut() = Some(err);
        event_loop.exit();
    }

    fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        let Some(_window) = &self.window else { return };
        let Some(device) = &self.device else { return };
        let Some(queue) = &self.queue else { return };
        let Some(config) = &self.surface_config else {
            return;
        };

        // Initialize renderer
        self.renderer.init(
            device.clone(),
            queue.clone(),
            config.format,
            config.width,
            config.height,
        );

        // Run startup systems
        if let Some(startup_systems) = self.startup_systems.take() {
            for system in startup_systems {
                system(&mut self.world, &mut self.renderer);
            }
        }

        // Upload all meshes
        for entity in self
            .world
            .query::<(&crate::math::Transform, &crate::renderer::Mesh)>()
            .iter()
        {
            if let Some(mesh) = self.world.get::<crate::renderer::Mesh>(entity) {
                self.renderer.upload_mesh(mesh);
            }
        }

        self.initialized = true;
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title(&self.config.title)
                .with_inner_size(PhysicalSize::new(self.config.width, self.config.height))
                .with_resizable(self.config.resizable);

            let window = match event_loop.create_window(window_attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    self.set_init_error_and_exit(event_loop, Error::Window(e.to_string()));
                    return;
                }
            };

            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
                ..Default::default()
            });

            let surface = match instance.create_surface(window.clone()) {
                Ok(s) => s,
                Err(e) => {
                    self.set_init_error_and_exit(
                        event_loop,
                        Error::Surface(format!("create surface: {}", e)),
                    );
                    return;
                }
            };

            let adapter =
                match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })) {
                    Some(a) => a,
                    None => {
                        self.set_init_error_and_exit(event_loop, Error::NoAdapter);
                        return;
                    }
                };

            log::info!("Using adapter: {:?}", adapter.get_info());

            let (device, queue) = match pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )) {
                Ok(pair) => pair,
                Err(e) => {
                    self.set_init_error_and_exit(event_loop, Error::DeviceRequest(e.to_string()));
                    return;
                }
            };

            let device = Arc::new(device);
            let queue = Arc::new(queue);

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let present_mode = if self.config.vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::Immediate
            };

            let size = window.inner_size();
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &surface_config);

            // Update engine window
            self.engine_window.width = size.width;
            self.engine_window.height = size.height;
            self.engine_window.scale_factor = window.scale_factor();

            self.window = Some(window);
            self.surface = Some(surface);
            self.device = Some(device);
            self.queue = Some(queue);
            self.surface_config = Some(surface_config);

            self.initialize();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let (Some(device), Some(surface), Some(config)) =
                    (&self.device, &self.surface, &mut self.surface_config)
                {
                    config.width = size.width.max(1);
                    config.height = size.height.max(1);
                    surface.configure(device, config);
                    self.renderer.resize(config.width, config.height);
                    self.engine_window.width = config.width;
                    self.engine_window.height = config.height;
                }
            }

            WindowEvent::Focused(focused) => {
                self.engine_window.focused = focused;
                if !focused {
                    // Release mouse on focus loss
                    if let Some(window) = &self.window {
                        let _ = window.set_cursor_grab(CursorGrabMode::None);
                        window.set_cursor_visible(true);
                        self.input.set_mouse_captured(false);
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                match state {
                    ElementState::Pressed => self.input.on_key_pressed(key_code),
                    ElementState::Released => self.input.on_key_released(key_code),
                }

                // ESC to release mouse or exit
                if key_code == KeyCode::Escape && state == ElementState::Pressed {
                    if self.input.is_mouse_captured() {
                        if let Some(window) = &self.window {
                            let _ = window.set_cursor_grab(CursorGrabMode::None);
                            window.set_cursor_visible(true);
                            self.input.set_mouse_captured(false);
                        }
                    } else {
                        event_loop.exit();
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let button = MouseButton::from(button);
                match state {
                    ElementState::Pressed => {
                        self.input.on_mouse_button_pressed(button);

                        // Capture mouse on click
                        if !self.input.is_mouse_captured() {
                            if let Some(window) = &self.window {
                                let _ = window
                                    .set_cursor_grab(CursorGrabMode::Confined)
                                    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
                                window.set_cursor_visible(false);
                                self.input.set_mouse_captured(true);
                            }
                        }
                    }
                    ElementState::Released => self.input.on_mouse_button_released(button),
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .on_mouse_moved(position.x as f32, position.y as f32);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.input.on_scroll(dx, dy);
            }

            WindowEvent::RedrawRequested => {
                // Calculate delta time
                let now = Instant::now();
                let delta = (now - self.last_frame).as_secs_f32();
                self.last_frame = now;
                self.time.update(delta);

                // Update physics first (use real delta so speed is FPS-independent)
                self.physics.update(&mut self.world, self.time.delta());

                // Run systems (input, sync camera from player, collectibles, etc.)
                {
                    let mut state = GameState {
                        world: &mut self.world,
                        input: &self.input,
                        time: &self.time,
                        renderer: &mut self.renderer,
                        physics: &mut self.physics,
                        audio: &self.audio,
                        window: &self.engine_window,
                    };

                    for system in &mut self.systems {
                        system(&mut state);
                    }
                }

                // Render
                if let Some(surface) = &self.surface {
                    match surface.get_current_texture() {
                        Ok(output) => {
                            let view = output
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());
                            self.renderer.render(&self.world, &view);
                            output.present();
                        }
                        Err(wgpu::SurfaceError::Lost) => {
                            if let (Some(device), Some(config)) =
                                (&self.device, &self.surface_config)
                            {
                                surface.configure(device, config);
                            }
                        }
                        Err(wgpu::SurfaceError::Outdated) => {
                            if let (Some(device), Some(config)) =
                                (&self.device, &self.surface_config)
                            {
                                surface.configure(device, config);
                            }
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::trace!(
                                "Surface get_current_texture timeout (e.g. vsync), skip frame"
                            );
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("GPU out of memory, exiting");
                            event_loop.exit();
                        }
                    }
                }

                // Update window title with FPS (throttled to ~0.25s to avoid syscall every frame).
                if let Some(window) = &self.window {
                    self.title_frame_count += 1;
                    let elapsed = self.last_title_update.elapsed().as_secs_f32();
                    if elapsed >= 0.25 {
                        let fps = (self.title_frame_count as f32 / elapsed) as u32;
                        let title = format!("{} | FPS: {}", self.config.title, fps);
                        window.set_title(&title);
                        self.title_frame_count = 0;
                        self.last_title_update = Instant::now();
                    }
                }

                // Clear input state for next frame
                self.input.update();
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.input.is_mouse_captured() {
                self.input.on_mouse_delta(delta.0 as f32, delta.1 as f32);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
