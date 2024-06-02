use std::ffi::{c_void, CStr, CString};
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use glutin::display::{Display, GlDisplay};
use log::{error, info, warn};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use crate::render::fonts::load_fonts;
use crate::render::gl::UNPACK_ALIGNMENT;
use crate::render::images::load_images;
use crate::render::screens::main::MainScreen;
use crate::render::screens::{ScreenManagementCmd, ScreenTrait};
use crate::render::screens::records::{Records, RECORDS_LIST};

pub mod utils;
pub mod objects;
pub mod screens;
mod images;
mod fonts;

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
    gl: Option<Arc<gl::Gl>>,
}

extern "system" fn gl_debug_callback(
    source: gl::types::GLenum,
    ty: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut c_void,
) {
    unsafe {
        let message_str = CStr::from_ptr(message).to_string_lossy();
        error!(
            "OpenGL Debug Message:\nSource: {:?}\nType: {:?}\nID: {}\nSeverity: {:?}\nMessage: {}",
            source, ty, id, severity, message_str
        );
    }
}


pub const ANDROID_DATA_PATH: &str = "/data/user/0/com.skygrel.panther/files";
impl AppState {
    pub fn new(exit_request: Arc<AtomicBool>) -> Self {
        let records_path = format!("{}/records.json", ANDROID_DATA_PATH);

        //read test content if file exists
        info!("Loading records state from file {}...", records_path);
        if let Ok(file) = File::open(&records_path) {
            info!("File opened successfully! Loading records...");
            if let Ok(records) = serde_json::from_reader(file) {
                let mut records_list = RECORDS_LIST.lock();
                *records_list = records;
            } else {
                warn!("Deserialization failed! Creating empty records object...");
            }
        }
        else {
            warn!("File open failed! Creating empty records object...");
        }

        AppState {
            screens: Vec::new(),
            exit_request,
            gl: None
        }
    }

    /// Called once on resume. Checks if gl is initialized, create it from gl_display if not
    pub fn ensure_renderer(&mut self, gl_display: &Display, dims: PhysicalSize<u32>) {
        let gl = self.gl.get_or_insert_with(|| {
            info!("[AppState] Initializing GL...");

            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            unsafe {
                gl.PixelStorei(UNPACK_ALIGNMENT, 1);

                gl.Enable(gl::BLEND);
                gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                gl.Enable(gl::DEBUG_OUTPUT);
                gl.DebugMessageCallback(Some(gl_debug_callback), std::ptr::null());
            }

            load_images(&gl);
            load_fonts(&gl);

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                info!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                info!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                info!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            Arc::new(gl)
        });

        SURFACE_WIDTH.store(dims.width, Ordering::Relaxed);
        SURFACE_HEIGHT.store(dims.height, Ordering::Relaxed);

        //nice place to create first screen
        if self.screens.is_empty() {
            self.screens.push(Box::new(MainScreen::new(gl.clone(), self.exit_request.clone())));
        }
    }

    // called repeatedly just before draw, to determine, should we draw
    pub fn renderer_ready(&self) -> bool {
        !self.screens.is_empty()
    }

    pub fn get_input_screen(&mut self) -> Option<&mut Box<dyn ScreenTrait>> {
        if !self.screens.is_empty() {
            let i = self.screens.len() - 1;
            Some(&mut self.screens[i])
        } else {
            None
        }
    }

    #[profiling::function]
    pub fn update(&mut self) -> ScreenManagementCmd {
        // call input screen's update method
        if let Some(screen) = self.get_input_screen() {
            screen.update()
        } else {
            ScreenManagementCmd::None
        }
    }

    // called repeatedly from outside
    pub fn draw(&mut self) {
        puffin::profile_function!();
        unsafe {
            let gl = self.gl.as_ref().unwrap();
            gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        let mut screens_len = self.screens.len();
        let mut i = 0;
        while i < screens_len {
            profiling::scope!("Drawing screen", format!("iteration {}", i).as_str());
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
        check_gl_errors(self.gl.as_ref().unwrap());
    }

    pub fn pop_screen(&mut self) {
        info!("[ScreenStack] Popping top screen");
        self.screens.pop();
        if self.screens.is_empty() {
            self.exit_request.store(true, Ordering::Relaxed);
        }
    }

    pub fn push_screen(&mut self, screen: Box<dyn ScreenTrait>) {
        info!("[ScreenStack] Pushing new screen");
        self.screens.push(screen);
    }
}