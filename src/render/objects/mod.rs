use std::mem;
use std::sync::Arc;

use crate::render::{check_gl_errors, create_shader, get_surface_y_ratio, gl};
use crate::render::gl::Gles2;
use crate::render::gl::types::{GLsizei, GLsizeiptr, GLuint};


pub mod image;
pub mod r#box;
pub mod animated_image;
pub mod textbox;
pub mod start_animation;
pub mod tab;


#[rustfmt::skip]
pub static SQUAD_VERTEX_DATA: [f32; 12] = [
    -1.0, -1.0,
    1.0,  1.0,
    1.0, -1.0,
    -1.0, -1.0,
    -1.0,  1.0,
    1.0,  1.0,
];

pub struct BoxProgram {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    fbo: GLuint,
    gl: Arc<gl::Gl>,

    y_offset: f64,
    bounds: (f64, f64, f64, f64),
}

impl BoxProgram {
    pub fn new(gl: Arc<gl::Gl>, bounds: (f64, f64, f64, f64), frag_shader: &[u8]) -> Self {
        unsafe {

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, include_bytes!("common-box-vert.glsl"));
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, frag_shader);

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            let mut fbo = 0;
            gl.GenFramebuffers(1, &mut fbo);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            // dummy data
            let vert_data = [0.0, 0.0, 0.0, 0.0,
                1.0, 1.0, 1.0, 1.0,
                1.0, 0.0, 1.0, 0.0];
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vert_data.len() * mem::size_of::<f32>()) as GLsizeiptr,
                vert_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let ratio_location = gl.GetUniformLocation(program, b"y_ratio\0".as_ptr() as *const _);
            let ratio = get_surface_y_ratio();
            gl.Uniform1f(ratio_location, ratio as f32);

            check_gl_errors(&gl);
            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                4 * mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as GLuint);
            check_gl_errors(&gl);

            let tex_attrib = gl.GetAttribLocation(program, b"texcoord\0".as_ptr() as *const _);
            if tex_attrib != -1 {
                gl.VertexAttribPointer(
                    tex_attrib as GLuint,
                    2,
                    gl::FLOAT,
                    0,
                    4 * mem::size_of::<f32>() as GLsizei,
                    (2 * mem::size_of::<f32>()) as *const _,
                );
                gl.EnableVertexAttribArray(tex_attrib as GLuint);
            }
            check_gl_errors(&gl);

            let mut res = Self {
                program,
                vao,
                vbo,
                fbo,
                gl,
                bounds,
                y_offset: 0.0
            };

            res.update_bounds(bounds);

            res
        }
    }

    #[profiling::function]
    pub fn update_bounds(&mut self, bounds: (f64, f64, f64, f64)) {
        self.bounds = bounds;
        let gl = &self.gl;

        let left = bounds.0;
        let bottom = bounds.1 + self.y_offset;
        let right = left + bounds.2;
        let top = bottom + bounds.3;

        unsafe {
            let vert_data = [left as f32, bottom as f32, 0.0, 0.0,
                right as f32, top as f32, 1.0, 1.0,
                right as f32, bottom as f32, 1.0, 0.0,
                left as f32, bottom as f32, 0.0, 0.0,
                left as f32, top as f32, 0.0, 1.0,
                right as f32, top as f32, 1.0, 1.0];

            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vert_data.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vert_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
        }
    }

    pub fn set_pos_y_offset(&mut self, offset: f64) {
        self.y_offset = offset;
        self.update_bounds(self.bounds);
    }

    pub fn update_pos(&mut self, pos: (f64, f64)) {
        let mut bounds = self.bounds;
        bounds.0 = pos.0;
        bounds.1 = pos.1;
        self.update_bounds(bounds);
    }

    pub fn move_pos(&mut self, pos: (f64, f64)) {
        let mut bounds = self.bounds;
        bounds.0 += pos.0;
        bounds.1 += pos.1;
        self.update_bounds(bounds);
    }

    #[profiling::function]
    pub fn draw(&self, target_texture: GLuint, draw_fun: impl FnOnce(&Gles2)) {
        let gl = &self.gl;
        unsafe {
            gl.UseProgram(self.program);

            gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, target_texture, 0);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            draw_fun(gl);

            gl.DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}

impl Drop for BoxProgram {
    fn drop(&mut self) {
        let gl = &self.gl;

        unsafe {
            gl.DeleteProgram(self.program);
            gl.DeleteVertexArrays(1, &self.vao);
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteFramebuffers(1, &self.fbo);
        }
    }
}