use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use crate::render::{ create_shader, gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::gl::{BLEND, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use crate::render::gl::types::{GLsizei, GLsizeiptr, GLuint};
use crate::render::objects::SQUAD_VERTEX_DATA;


const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("box-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("box-frag.glsl");

pub struct Squad {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    fbo: GLuint,
    gl_mtx: Arc<Mutex<gl::Gl>>,
}


impl Squad {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, color: (f32, f32, f32), bounds: (f32, f32, f32, f32)) -> Self {
        unsafe {
            let gl = gl_mtx.lock().unwrap();

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            gl.Enable(BLEND);
            gl.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);

            let mut fbo = 0;
            gl.GenFramebuffers(1, &mut fbo);

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


            let ratio_location = gl.GetUniformLocation(program, b"y_ratio\0".as_ptr() as *const _);
            let dims = (SURFACE_WIDTH.load(Ordering::Relaxed) as f32, SURFACE_HEIGHT.load(Ordering::Relaxed) as f32);
            gl.Uniform1f(ratio_location, dims.1 / dims.0);

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                2 * mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as GLuint);

            let color_location = gl.GetUniformLocation(program, b"color\0".as_ptr() as *const _);
            gl.Uniform3f(color_location, color.0, color.1, color.2);

            let bounds_location = gl.GetUniformLocation(program, b"bounds\0".as_ptr() as *const _);
            gl.Uniform4f(bounds_location, bounds.0, bounds.1, bounds.2, bounds.3);

            mem::drop(gl);
            Self {
                program,
                vao,
                vbo,
                gl_mtx,
                fbo,
            }
        }
    }

    pub fn new_bg(gl_mtx: Arc<Mutex<gl::Gl>>, color: (f32, f32, f32)) -> Self {
        Self::new(gl_mtx, color, (0.0, 1.0, 0.0, 1.0))
    }

    pub fn draw(&mut self, texture_id: GLuint) {

        let gl = self.gl_mtx.lock().unwrap();


        // Check if the framebuffer is complete
        // let status = unsafe { gl.CheckFramebufferStatus(gl::FRAMEBUFFER) };
        // if status != gl::FRAMEBUFFER_COMPLETE {
        //     panic!("Failed to create framebuffer");
        // }

        unsafe {
            gl.UseProgram(self.program);

            gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            // let params = self.circ_anim.cur();
            // gl.Uniform3f(self.circle, params.0, params.1, params.2);

            gl.DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}


impl Drop for Squad {
    fn drop(&mut self) {
        let gl = self.gl_mtx.lock().unwrap();
        unsafe {
            gl.DeleteProgram(self.program);
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}
