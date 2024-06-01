
use std::sync::{Arc};
use image::GenericImageView;

use crate::render::{gl};

use crate::render::gl::types::{GLuint};
use crate::render::images::ImageData;
use crate::render::objects::{BoxProgram};
use crate::render::utils::position::FixedPosition;


pub struct Image {
    gl: Arc<gl::Gl>,
    box_prog: BoxProgram,

    img_texture: GLuint,
    color: Option<(f64, f64, f64)>,
}

impl Image {
    pub fn new(gl: Arc<gl::Gl>, img: ImageData, pos: FixedPosition, color: Option<(f64, f64, f64)>) -> Self {
        unsafe {
            let aspect_ratio = img.height as f64 / img.width as f64;
            let bounds = pos.get(aspect_ratio);

            let box_prog = BoxProgram::new(gl.clone(), bounds, include_bytes!("image-frag.glsl"));

            let tex_location = gl.GetUniformLocation(box_prog.program, b"tex\0".as_ptr() as *const _);
            gl.Uniform1i(tex_location, 1);


            let tex_location = gl.GetUniformLocation(box_prog.program, b"u_color\0".as_ptr() as *const _);
            if color.is_some() {
                let (r, g, b) = color.unwrap();
                gl.Uniform3f(tex_location, r as f32, g as f32, b as f32);
            }
            else {
                gl.Uniform3f(tex_location, 1.0, 1.0, 1.0);
            }
            // info!("[img] pos: {:?}", bounds);

            Self {
                gl,
                img_texture: img.texture_id,
                box_prog,
                color
            }
        }
    }

    pub fn new_bg(gl: Arc<gl::Gl>, img: ImageData, color: Option<(f64, f64, f64)>) -> Self {
        Self::new(gl, img, FixedPosition::new().width(1.0), color)
    }

    pub fn set_pos_y_offset(&mut self, offset: f64) {
        self.box_prog.set_pos_y_offset(offset);
    }

    pub fn draw(&mut self, texture_id: GLuint) {
        self.box_prog.draw(texture_id, |gl| unsafe {
            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_2D, self.img_texture);
        });
    }
}