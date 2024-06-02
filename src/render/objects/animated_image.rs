use std::sync::Arc;
use std::time::Instant;

use crate::render::gl;
use crate::render::gl::types::{GLint, GLuint};
use crate::render::images::ImageData;
use crate::render::objects::BoxProgram;
use crate::render::utils::position::FixedPosition;

pub struct AnimatedImage {
    gl: Arc<gl::Gl>,
    box_prog: BoxProgram,

    img_textures: Vec<GLuint>,
    pub img_count: usize,

    u_texture_loc: GLint,
    dims: (u32, u32),
    bounds: (f64, f64, f64, f64),

    img_period: f64,
    last_frame_time: Instant,
    cur_frame: usize
}

impl AnimatedImage {
    pub fn new(gl: Arc<gl::Gl>, imgs: Vec<ImageData>, pos: FixedPosition, img_period: f64) -> Self {
        unsafe {
            let dims = (imgs[0].width, imgs[1].height);
            let aspect_ratio = imgs[0].height as f64 / imgs[0].width as f64;
            let bounds = pos.get(aspect_ratio);

            let box_prog = BoxProgram::new(gl.clone(), bounds, include_bytes!("animated-image-frag.glsl"));

            let u_texture_loc = gl.GetUniformLocation(box_prog.program, b"tex\0".as_ptr() as *const _);
            gl.Uniform1i(u_texture_loc, 1);
            // info!("[img] pos: {:?}", bounds);

            let img_textures: Vec<_> = imgs.into_iter().map(|i| i.texture_id).collect();
            let img_count = img_textures.len();

            Self {
                gl,
                box_prog,

                img_textures,
                u_texture_loc,
                dims,
                img_count,
                img_period,
                last_frame_time: Instant::now(),
                cur_frame: 0,
                bounds
            }
        }
    }

    pub fn new_bg(gl: Arc<gl::Gl>, imgs: Vec<ImageData>, img_period: f64) -> Self {
        Self::new(gl, imgs, FixedPosition::new().width(1.0), img_period)
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.img_period = speed;
    }

    pub fn set_full_pos(&mut self, pos: FixedPosition) {
        let aspect_ratio = self.dims.1 as f64 / self.dims.0 as f64;
        let bounds = pos.get(aspect_ratio);
        self.box_prog.update_bounds(bounds);
    }

    pub fn set_pos(&mut self, x: f64, y: f64) {
        self.box_prog.update_pos((x, y));
    }

    pub fn move_pos(&mut self, x_diff: f64, y_diff: f64) {
        self.box_prog.move_pos((x_diff, y_diff));
    }

    #[profiling::function]
    pub fn draw(&mut self, texture_id: GLuint) {
        let _gl = &self.gl;

        if self.last_frame_time.elapsed().as_secs_f64() > self.img_period {
            self.last_frame_time = Instant::now();

            let frame = self.cur_frame + 1;
            self.cur_frame = frame % self.img_count;
        }

        self.box_prog.draw(texture_id, |gl| unsafe {
            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_2D, self.img_textures[self.cur_frame]);
        });
    }
}