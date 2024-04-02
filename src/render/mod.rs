use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use glutin::display::{Display, GlDisplay};
use log::{error, info};
use winit::dpi::PhysicalPosition;
use crate::render::screens::main::MainScreen;
use crate::render::screens::ScreenTrait;

pub mod utils;
pub mod objects;
pub mod screens;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    let len = source.len() as gl::types::GLint;
    gl.ShaderSource(shader, 1, [source.as_ptr().cast()].as_ptr(), &len);
    gl.CompileShader(shader);
    shader
}

pub fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

pub fn check_gl_errors(gl: &gl::Gl) {
    unsafe {
        let mut err = gl.GetError();
        while err != gl::NO_ERROR {
            error!("GL error: {}", err);
            err = gl.GetError();
        }
    }
}
pub static SURFACE_WIDTH: AtomicU32 = AtomicU32::new(0);
pub static SURFACE_HEIGHT: AtomicU32 = AtomicU32::new(0);
pub fn get_surface_y_ratio() -> f64 {
    let width = SURFACE_WIDTH.load(Ordering::Relaxed);
    let height = SURFACE_HEIGHT.load(Ordering::Relaxed);
    if width == 0 {
        return 0.0;
    }
    height as f64 / width as f64

}
#[derive(Debug)]
pub enum MyInputEvent {
    Back,
    TouchEvent(u64, PhysicalPosition<f64>, winit::event::TouchPhase),
}

pub struct AppState {
    screens: Vec<Box<dyn ScreenTrait>>,
    exit_request: Arc<AtomicBool>,
    gl: Option<Arc<Mutex<gl::Gl>>>,
}

impl AppState {
    pub fn new(exit_request: Arc<AtomicBool>) -> Self {
        AppState {
            screens: Vec::new(),
            exit_request,
            gl: None
        }
    }

    // called once on resume
    pub fn ensure_renderer(&mut self, gl_display: &Display, dims: (u32, u32)) {
        let gl = self.gl.get_or_insert_with(|| {
            info!("[AppState] Initializing GL...");

            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            Arc::new(Mutex::new(gl))
        });

        SURFACE_WIDTH.store(dims.0, Ordering::Relaxed);
        SURFACE_HEIGHT.store(dims.1, Ordering::Relaxed);

        //nice place to create first screen
        if self.screens.len() == 0 {
            self.screens.push(Box::new(MainScreen::new(gl.clone(), self.exit_request.clone())));
        }
    }

    // called repeatedly just before draw, to determine, should we draw
    pub fn renderer_ready(&self) -> bool {
        self.screens.len() > 0
    }

    pub fn get_input_screen(&mut self) -> Option<&mut Box<dyn ScreenTrait>> {
        if self.screens.len() > 0 {
            let i = self.screens.len() - 1;
            Some(&mut self.screens[i])
        } else {
            None
        }
    }

    // called repeatedly from outside
    pub fn draw(&mut self) {

        unsafe {

            let gl = self.gl.as_ref().unwrap().lock().unwrap();
            gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        let mut screens_len = self.screens.len();
        let mut i = 0;
        while i < screens_len {
            self.screens[i].draw();
            if self.screens[i].is_expanded() && i > 0 {
                info!("[ScreenStack]Screen {} is expanded, dropping back screens...", i);
                let preserve_screens = self.screens.split_off(i);
                self.screens = preserve_screens;
                screens_len = self.screens.len();
                i = 1;
            }
            else {
                i += 1;
            }
        }
        check_gl_errors(&self.gl.as_ref().unwrap().lock().unwrap());
    }

    pub fn pop_screen(&mut self) {
        info!("[ScreenStack] Popping top screen");
        self.screens.pop();
        if self.screens.len() == 0 {
            self.exit_request.store(true, Ordering::Relaxed);
        }
    }

    pub fn push_screen(&mut self, screen: Box<dyn ScreenTrait>) {
        info!("[ScreenStack] Pushing new screen");
        self.screens.push(screen);
    }
}