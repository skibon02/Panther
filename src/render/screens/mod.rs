pub mod main;
pub mod records;
pub mod stats;

use std::mem;
use std::sync::{Arc, Mutex};
use log::{error, info};
use crate::render::{check_gl_errors, create_shader, gl};
use crate::render::gl::{BLEND, COLOR, ONE_MINUS_SRC_ALPHA, SRC_ALPHA, UNPACK_ALIGNMENT};
use crate::render::gl::types::{GLint, GLsizei, GLsizeiptr, GLuint};
use crate::render::objects::SQUAD_VERTEX_DATA;
use crate::render::utils::circle_animation::CircleAnimation;

pub enum ScreenManagementCmd {
    None,
    PushScreen(Box<dyn ScreenTrait>),
    PopScreen
}
pub trait ScreenTrait {
    fn start_scroll(&mut self, pos: (f64, f64)) -> bool {
        true
    }
    fn scroll(&mut self, pos: (f64, f64)) {
        // info!("YAY scroll!!!! {:?}", pos);
    }
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        info!("YAY press!!!! {:?}", pos);

        ScreenManagementCmd::None
    }
    fn back(&mut self) -> ScreenManagementCmd {
        ScreenManagementCmd::None
    }
    fn draw(&mut self);

    fn is_expanded(&self) -> bool {
        false
    }
}

pub struct ScreenRendering {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    fbo: GLuint,
    gl_mtx: Arc<Mutex<gl::Gl>>,
    texture: GLuint,
    dims: (u32, u32),

    circle: GLint,
    circle_anim: CircleAnimation
}


const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("present-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("present-frag.glsl");

impl ScreenRendering {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, dims: (u32, u32), mut circle_anim: CircleAnimation) -> Self {
        let gl = gl_mtx.lock().unwrap();

        unsafe {
            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            let program = gl.CreateProgram();

            gl.PixelStorei(UNPACK_ALIGNMENT, 1);

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            gl.Enable(BLEND);
            gl.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (SQUAD_VERTEX_DATA.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                SQUAD_VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                0,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as GLuint);


            let mut fbo = 0;
            gl.GenFramebuffers(1, &mut fbo);


            // Generate a texture ID
            let mut texture = std::mem::zeroed();
            gl.GenTextures(1, &mut texture);

            // Bind the texture
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, texture);

            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            let data = vec![0u8; (dims.0 * dims.1 * 4) as usize]; // may be long
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                dims.0 as i32,
                dims.1 as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );

            let texture_loc = gl.GetUniformLocation(program, b"u_texture\0".as_ptr() as *const _);
            gl.Uniform1i(texture_loc, 0);

            let circle = gl.GetUniformLocation(program, b"u_circle\0".as_ptr() as *const _);
            gl.Uniform3f(circle, 0.0, 0.0, 0.0);

            let y_ratio = gl.GetUniformLocation(program, b"y_ratio\0".as_ptr() as *const _);
            gl.Uniform1f(y_ratio, dims.1 as f32 / dims.0 as f32);
            circle_anim.start();


            drop(gl);
            Self {
                program,
                vao,
                vbo,
                fbo,
                gl_mtx,
                texture,
                dims,

                circle,
                circle_anim
            }
        }
    }

    pub fn texture_id(&self) -> GLuint {
        self.texture
    }
    pub fn clear_texture(&self) {
        let gl = self.gl_mtx.lock().unwrap();

        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.texture, 0);
            if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete");
            }

            let clear_color = [0.0, 0.0, 0.0, 0.0];
            gl.ClearColor(clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn present(&self) {
        let gl = self.gl_mtx.lock().unwrap();

        unsafe {
            gl.UseProgram(self.program);
            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);// because we use this texture in rendering

            let circ_params = self.circle_anim.cur();
            gl.Uniform3f(self.circle, circ_params.0, circ_params.1, circ_params.2);


            gl.DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}

impl Drop for ScreenRendering {
    fn drop(&mut self) {
        let gl = self.gl_mtx.lock().unwrap();

        unsafe {
            gl.DeleteProgram(self.program);
            gl.DeleteVertexArrays(1, &self.vao);
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteFramebuffers(1, &self.fbo);
            gl.DeleteTextures(1, &self.texture);
        }
    }
}