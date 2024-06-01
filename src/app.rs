use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use glutin::prelude::*;

use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::Window;
use crate::render::{AppState, get_surface_y_ratio, SURFACE_WIDTH};
use crate::render::screens::ScreenManagementCmd;


pub enum TouchState {
    //start, distance, send_move
    MovingStart(PhysicalPosition<f64>, f64, bool), // moving less than distance 50px
    Moving(PhysicalPosition<f64>, bool), // moving more than 50ms
}

pub struct App {
    gl_context: Option<PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    gl_config: Option<Config>,

    // Window must be dropped at the end.
    window: Option<Window>,

    surface_dims: PhysicalSize<u32>,

    app_state: AppState,

    exit_request: Arc<AtomicBool>,
    touch_state: BTreeMap<u64, TouchState>,
}

impl App {
    pub fn new(exit_request: Arc<AtomicBool>) -> Self {
        Self {
            window: None,
            gl_context: None,
            gl_surface: None,
            gl_config: None,
            app_state: AppState::new(exit_request.clone()),
            exit_request,
            touch_state: BTreeMap::new(),
            surface_dims: PhysicalSize::new(0, 0)
        }
    }
}

impl App {
    fn create_window(&mut self, active_event_loop: &ActiveEventLoop) {
        // Only Windows requires the window to be present before creating the display.
        // Other platforms don't really need one.
        let window_attributes = Window::default_attributes()
            .with_transparent(true)
            .with_title("Panther tracker");

        // The template will match only the configurations supporting rendering
        // to windows.
        //
        // XXX We force transparency only on macOS, given that EGL on X11 doesn't
        // have it, but we still want to show window. The macOS situation is like
        // that, because we can query only one config at a time on it, but all
        // normal platforms will return multiple configs, so we can find the config
        // with transparency ourselves inside the `reduce`.
        let template =
            ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(active_event_loop, template, gl_config_picker)
            .expect("Failed to create Window.");

        if let Some(window) = &window {
            let size  = window.inner_size();
            self.surface_dims = size;
        }

        println!("Picked a config with {} samples", gl_config.num_samples());

        let raw_window_handle = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());

        // XXX The display could be obtained from any object created by it, so we can
        // query it from the config.
        let gl_display = gl_config.display();

        // The context creation part.
        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(raw_window_handle);

        // There are also some old devices that support neither modern OpenGL nor GLES.
        // To support these we can try and create a 2.1 context.
        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_display.create_context(&gl_config, &context_attributes).unwrap_or_else(|_| {
                gl_display.create_context(&gl_config, &fallback_context_attributes).unwrap_or_else(
                    |_| {
                        gl_display
                            .create_context(&gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    },
                )
            })
        };

        self.window = window;
        self.gl_config = Some(gl_config);
        self.gl_context = Some(not_current_gl_context.treat_as_possibly_current());
    }

    pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        if let Some((gl_context, gl_surface)) =
            &self.gl_context.as_ref().zip(self.gl_surface.as_ref())
        {
            gl_surface.resize(
                gl_context,
                NonZeroU32::new(new_size.width).unwrap(),
                NonZeroU32::new(new_size.height).unwrap(),
            );
            self.surface_dims = new_size;

            let gl_display = self.gl_config.as_ref().unwrap().display();
            self.app_state.ensure_renderer(&gl_display, new_size);
        }
    }

    pub fn queue_redraw(&self) {
        if let Some(window) = &self.window {
            // log::debug!("Making Redraw Request");
            window.request_redraw();
        }
    }

    pub fn resume(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Resumed, creating render state...");

        let (gl_config, window) =
            if let Some(state) = self.gl_config.as_ref().zip(self.window.as_ref()) {
                state
            } else {
                self.create_window(event_loop);

                (self.gl_config.as_ref().unwrap(), self.window.as_ref().unwrap())
            };

        let attrs = window.build_surface_attributes(Default::default()).unwrap();
        let gl_surface =
            unsafe { gl_config.display().create_window_surface(gl_config, &attrs).unwrap() };

        let gl_context = self.gl_context.as_ref().unwrap();

        // Make it current.
        gl_context.make_current(&gl_surface).unwrap();

        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        self.app_state.ensure_renderer(&gl_config.display(), self.surface_dims);

        // Try setting vsync.
        if let Err(res) = gl_surface
            .set_swap_interval(gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            eprintln!("Error setting vsync: {res:?}");
        }

        self.gl_surface = Some(gl_surface);

        // self.queue_redraw();
    }

    ///
    pub fn handle_redraw_request(&mut self) {
        if let Some(ref surface) = self.gl_surface {
            if let Some(ctx) = &self.gl_context {
                if self.app_state.renderer_ready() {
                    match self.app_state.update() {
                        ScreenManagementCmd::PopScreen => {
                            self.app_state.pop_screen();
                        }
                        ScreenManagementCmd::PushScreen(screen) => {
                            self.app_state.push_screen(screen);
                        }
                        _ => {}
                    }
                    self.app_state.draw();


                    if let Err(err) = surface.swap_buffers(ctx) {
                        log::error!("Failed to swap buffers after render: {}", err);
                    }
                }
                self.queue_redraw();
            }
        }
    }

    pub fn handle_suspend(&mut self) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");

        // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
        // the window back to the system.
        let gl_context = self.gl_context.take().unwrap();
        gl_context.make_not_current().unwrap();
    }

    /// can potentially call exit
    pub fn handle_back_button(&mut self) {
        if let Some(screen) = self.app_state.get_input_screen() {
            match screen.back() {
                ScreenManagementCmd::PopScreen => {
                    self.app_state.pop_screen();
                }
                ScreenManagementCmd::PushScreen(screen) => {
                    self.app_state.push_screen(screen);
                }
                _ => {}
            }
        }
        else {
            log::warn!("Back button pressed, but no screen to send it to");
        }
    }

    pub fn handle_close_request(&mut self) {
        self.exit_request.store(true, Ordering::Relaxed);
    }

    pub fn handle_touch(&mut self, id: u64, location: PhysicalPosition<f64>, phase: winit::event::TouchPhase) {
        if let Some(screen) = self.app_state.get_input_screen() {
            let screen_width = SURFACE_WIDTH.load(Ordering::Relaxed) as f64;
            let y_ratio = get_surface_y_ratio();
            match phase {
                winit::event::TouchPhase::Started => {
                    let should_send_move = screen.start_scroll((location.x / screen_width, y_ratio - location.y / screen_width));
                    self.touch_state.insert(id, TouchState::MovingStart(location, 0.0, should_send_move));
                }
                winit::event::TouchPhase::Moved => {
                    if let Some(touch_state) = self.touch_state.get_mut(&id) {
                        match *touch_state {
                            TouchState::MovingStart(prev_pos, distance, should_send_move) => {
                                //trigger to switch to moving state
                                let diff = (location.x - prev_pos.x, location.y - prev_pos.y);
                                if should_send_move {
                                    screen.scroll((diff.0 / screen_width, -diff.1 / screen_width));
                                }
                                if distance > 50.0 {
                                    *touch_state = TouchState::Moving(location, should_send_move);
                                }
                                else {
                                    //just update location
                                    *touch_state = TouchState::MovingStart(location, distance + diff.0.abs() + diff.1.abs(), should_send_move);
                                }
                            }
                            TouchState::Moving(prev_pos, should_send_move) => {
                                let diff = (location.x - prev_pos.x, location.y - prev_pos.y);
                                if should_send_move {
                                    screen.scroll((diff.0 / screen_width, -diff.1 / screen_width));
                                }
                                //just update location
                                *touch_state = TouchState::Moving(location, should_send_move);
                            }
                        }
                    }
                }
                winit::event::TouchPhase::Ended => {
                    if let Some(touch_state) = self.touch_state.remove(&id) {
                        if let TouchState::MovingStart(_, _, _) = touch_state {
                            match screen.press((location.x / screen_width, y_ratio - location.y / screen_width)) {
                                ScreenManagementCmd::PushScreen(screen) => {
                                    self.app_state.push_screen(screen);
                                }
                                ScreenManagementCmd::PopScreen => {
                                    self.app_state.pop_screen();
                                }
                                _ => {}

                            }
                        }
                    }
                }
                winit::event::TouchPhase::Cancelled => {
                    self.touch_state.remove(&id); // just cancel
                }
            }
        }
        else {
            log::warn!("Touch event, but no screen to send it to");
        }
    }
}

pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}