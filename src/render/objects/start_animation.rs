use std::sync::Arc;
use std::time::Instant;
use log::info;
use crate::render::{get_surface_y_ratio, gl};
use crate::render::gl::types::{GLint, GLuint};
use crate::render::images::get_gif;
use crate::render::objects::BoxProgram;
use crate::render::utils::position::FreePosition;

pub struct StartAnimation {
    gl: Arc<gl::Gl>,
    box_prog: BoxProgram,

    start: Instant,
    animation_start: Option<Instant>,

    t_loc: GLint,

    img_textures: Vec<GLuint>,
    pub img_count: usize,

    u_texture_loc: GLint,

    img_period: f64,
    last_frame_time: Instant,
    cur_frame: usize
}

impl StartAnimation {
    pub fn new(gl: Arc<gl::Gl>, pos: FreePosition) -> Self {
        unsafe {
            let imgs = get_gif("running").unwrap();

            let squad = BoxProgram::new(gl.clone(), pos.get(), include_bytes!("start-animation-frag.glsl"));

            let u_texture_loc = gl.GetUniformLocation(squad.program, b"tex\0".as_ptr() as *const _);
            gl.Uniform1i(u_texture_loc, 1);

            let img_textures: Vec<_> = imgs.into_iter().map(|i| i.texture_id).collect();
            let img_count = img_textures.len();

            let t_loc = gl.GetUniformLocation(squad.program, b"t\0".as_ptr() as *const _);

            Self {
                gl,
                box_prog: squad,
                animation_start: None,
                start: Instant::now(),

                t_loc,


                img_textures,
                u_texture_loc,
                img_count,
                img_period: 0.03,
                last_frame_time: Instant::now(),
                cur_frame: 0,
            }
        }
    }

    pub fn new_bg(gl: Arc<gl::Gl>) -> Self {
        Self::new(gl, FreePosition::new().width(1.0).height(get_surface_y_ratio()))
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.img_period = speed;
    }

    pub fn launch(&mut self) {
        info!("Start pressed!");
        self.animation_start = Some(Instant::now());
    }

    pub fn is_finished(&mut self) -> bool {
        self.animation_start.map(|t|t.elapsed().as_secs_f32() > 8.0).unwrap_or(false)
    }

    pub fn draw(&mut self, texture_id: GLuint) {
        let anim_time = if let Some(start) = self.animation_start {
            start.elapsed().as_secs_f32()
        }
        else {
            self.start.elapsed().as_secs_f32().sin() * 0.1 + 0.1
        };

        let mut anim_times = [anim_time - 0.1, anim_time - 0.066, anim_time - 0.033, anim_time];
        anim_times.iter_mut().for_each(|v| *v = v.clamp(0.0, 3.0));

        anim_times.iter_mut().for_each(|v| {
            match *v {
                0.0..=1.0 => {
                    *v = v.powf(0.8)
                }
                1.0..=3.0 => {
                    *v = (*v - 1.0).powf(0.8) + 1.0
                }
                _ => {
                    *v = (*v - 3.0).powf(0.8) + 2.0
                }
            }
        });



        if self.last_frame_time.elapsed().as_secs_f64() > self.img_period {
            self.last_frame_time = Instant::now();

            let frame = self.cur_frame + 1;
            self.cur_frame = frame % self.img_count;
        }

        self.box_prog.draw(texture_id, |gl| unsafe {
            gl.UseProgram(self.box_prog.program);
            gl.Uniform4f(self.t_loc, anim_times[3], anim_times[2], anim_times[1], anim_times[0]);

            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_2D, self.img_textures[self.cur_frame]);
        });
    }
}