use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::images::get_gif;
use crate::render::objects::animated_image::AnimatedImage;
use crate::render::objects::r#box::Squad;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::main::MainScreen;
use crate::render::screens::records::RecordsScreen;
use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::FixedPosition;


pub struct StatsScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,
    screen_rendering: ScreenRendering,

    img: AnimatedImage,

    exit_request: Arc<AtomicBool>,
    start: Instant,

    cur_color: (f32, f32, f32),
}

impl StatsScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let cur_color = (0.5, 0.2, 0.9);
        let squad = Squad::new_bg(gl.clone(), cur_color);

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let img_pos = FixedPosition::new().width(1.0).bottom(1.0);
        let img = AnimatedImage::new(gl.clone(), get_gif("walking").unwrap(), img_pos, 0.1);

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        StatsScreen {
            gl,
            bg_squad: squad,
            exit_request,
            start: Instant::now(),
            screen_rendering,
            img,
            cur_color
        }
    }
}

impl ScreenTrait for StatsScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        ScreenManagementCmd::PushScreen(Box::new(RecordsScreen::new(self.gl.clone(), self.exit_request.clone())))
    }
    fn back(&mut self) -> ScreenManagementCmd {
        // self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::PushScreen(Box::new(MainScreen::new(self.gl.clone(), self.exit_request.clone())))
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);
        self.img.draw(texture_id);

        self.screen_rendering.present();
    }
    fn scroll(&mut self, pos: (f64, f64)) {
        self.cur_color.2 -= pos.0 as f32 / 2.0;
        self.cur_color.0 -= pos.1 as f32 / 2.0;

        self.cur_color.2 = self.cur_color.2.clamp(0.0, 1.0);
        self.cur_color.0 = self.cur_color.0.clamp(0.0, 1.0);

        self.bg_squad.set_color(self.cur_color);
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 0.5
    }
}