use std::mem;
use std::sync::Arc;
use log::debug;
use crate::render::{create_shader, get_surface_y_ratio, gl};
use crate::render::fonts::FontData;
use crate::render::gl::Gles2;
use crate::render::gl::types::{GLint, GLsizei, GLsizeiptr, GLuint};

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("textbox-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("textbox-frag.glsl");

pub struct TextBox {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    fbo: GLuint,
    gl: Arc<gl::Gl>,
    font_table: FontData,

    pos: (f32, f32),
    text: String,
    scale: f32,
    vert_buf: Vec<f32>,
    style: u32,

    prev_y_offs: f32
}

fn build_vertex_buffer(gl: &Gles2, pos: &(f32, f32), scale: f32, vbo: GLuint, font_table: &FontData, text: String) -> Vec<f32> {
    let mut temp_buf = vec![];

    let mut prev_char = None;

    let mut cursor_pos_x = pos.0;
    let mut cursor_pos_y = pos.1;
    for c in text.chars() {
        match c {
            '\n' => {
                //move cursor to new line
                cursor_pos_x = pos.0;

                //use height value of 'n'
                let glyph_params = font_table.glyph_params.get(&'h').unwrap();

                cursor_pos_y -= (glyph_params.texture_rect.height() + font_table.line_gap) * scale * 1.2;
                prev_char = None;
            }
            ' ' => {
                //just move cursor
                //use advance value of 'n'
                let glyph_params = font_table.glyph_params.get(&'n').unwrap();
                cursor_pos_x += glyph_params.h_advance * scale;
                prev_char = None;
            }
            _ => {
                let glyph_params = font_table.glyph_params.get(&c).unwrap();

                debug!("Char: {}. h_advance: {}, h_side_bearing: {}, v_side_bearing: {}, v_advance: {}", c,
            glyph_params.h_advance, glyph_params.h_side_bearing, glyph_params.v_side_bearing, glyph_params.v_advance);

                let raster_rect = glyph_params.texture_rect;

                let x = raster_rect.min.x;
                let y = raster_rect.min.y;
                let w = raster_rect.width();
                let h = raster_rect.height();

                let cell_sz_x = w * scale;
                let cell_sz_y = h * scale;

                let prev_char = prev_char.replace(c);
                let additional_kerning = if let Some(p) = prev_char {
                    font_table.kern_space(p, c)
                } else {
                    0.0
                };

                let glyph_advance = (glyph_params.h_advance + additional_kerning) * scale;

                let glyph_x_fixed = cursor_pos_x + glyph_params.h_side_bearing * scale;
                let glyph_y_fixed = cursor_pos_y + glyph_params.v_side_bearing * scale;

                temp_buf.extend_from_slice(&[
                    glyph_x_fixed + cell_sz_x, glyph_y_fixed, x + w, y + h,
                    glyph_x_fixed + cell_sz_x, glyph_y_fixed + cell_sz_y, x + w, y,
                    glyph_x_fixed, glyph_y_fixed + cell_sz_y, x, y,

                    glyph_x_fixed + cell_sz_x, glyph_y_fixed, x + w, y + h,
                    glyph_x_fixed, glyph_y_fixed + cell_sz_y, x, y,
                    glyph_x_fixed, glyph_y_fixed, x, y + h,
                ]);

                cursor_pos_x += glyph_advance;
                debug!("new cursor x: {}", cursor_pos_x);
            }
        }
    }

    unsafe {
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (temp_buf.len() * mem::size_of::<f32>()) as GLsizeiptr,
            temp_buf.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
    }
    temp_buf
}

impl TextBox {
    pub fn new(gl: Arc<gl::Gl>, font: FontData, text: String, pos: (f32, f32), scale: f32, style: u32) -> Self {
        unsafe {
            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

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
            let vert_buf = build_vertex_buffer(&gl, &pos, scale, vbo, &font, text.clone());


            let ratio_location = gl.GetUniformLocation(program, b"y_ratio\0".as_ptr() as *const _);
            let ratio = get_surface_y_ratio();
            gl.Uniform1f(ratio_location, ratio as f32);

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

            let texcoord_attrib = gl.GetAttribLocation(program, b"texcoord\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                texcoord_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                4 * mem::size_of::<f32>() as GLsizei,
                (2 * mem::size_of::<f32>()) as *const _,
            );
            gl.EnableVertexAttribArray(texcoord_attrib as GLuint);

            let tex_location = gl.GetUniformLocation(program, b"tex\0".as_ptr() as *const _);
            gl.Uniform1i(tex_location, 1);

            let style_location = gl.GetUniformLocation(program, b"u_style\0".as_ptr() as *const _);
            gl.Uniform1i(style_location, style as GLint);

            Self {
                program,
                vao,
                vbo,
                gl,
                fbo,
                font_table: font,
                pos,
                text,

                scale,
                vert_buf,
                style,

                prev_y_offs: 0.0
            }
        }
    }

    pub fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
        self.vert_buf = build_vertex_buffer(&self.gl, &self.pos, self.scale, self.vbo, &self.font_table, self.text.clone());
    }

    pub fn set_pos_y_offs(&mut self, pos_y_offs: f64) {
        let movement = pos_y_offs as f32 - self.prev_y_offs;
        self.prev_y_offs = pos_y_offs as f32;

        let mut vert_buf = self.vert_buf.clone();
        let gl = &self.gl;

        for i in (1..vert_buf.len()).step_by(4) {
            let mut y = vert_buf[i];
            y += movement;
            vert_buf[i] = y;
        }

        unsafe {
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vert_buf.len() * mem::size_of::<f32>()) as GLsizeiptr,
                vert_buf.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
        }

        self.vert_buf = vert_buf;
    }

    pub fn set_text(&mut self, text: String) {
        if text == self.text {
            return;
        }
        self.text = text;
        self.vert_buf = build_vertex_buffer(&self.gl, &self.pos, self.scale, self.vbo, &self.font_table, self.text.clone());
    }

    pub fn draw(&mut self, texture_id: GLuint) {
        let gl = &self.gl;

        unsafe {
            gl.UseProgram(self.program);

            gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_2D, self.font_table.texture_id);

            // let params = self.circ_anim.cur();
            // gl.Uniform3f(self.circle, params.0, params.1, params.2);

            gl.DrawArrays(gl::TRIANGLES, 0, self.vert_buf.len() as GLsizei / 4);
        }
    }
}


impl Drop for TextBox {
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
