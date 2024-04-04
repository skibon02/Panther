use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::images::get_gif;
use crate::render::objects::animated_image::AnimatedImage;
use crate::render::objects::r#box::Squad;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::records::RecordsScreen;
use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::FixedPosition;


pub struct StatsScreen {
    gl_mtx: Arc<Mutex<gl::Gl>>,
    bg_squad: Squad,
    screen_rendering: ScreenRendering,

    img: AnimatedImage,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl StatsScreen {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl_mtx.clone(), (0.5, 0.2, 0.9));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let img_pos = FixedPosition::new().width(1.0).bottom(1.0);
        let img = AnimatedImage::new(gl_mtx.clone(), get_gif("walking").unwrap(), img_pos);

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl_mtx.clone(), dims, circ_anim);

        StatsScreen {
            gl_mtx,
            bg_squad: squad,
            exit_request,
            start: Instant::now(),
            screen_rendering,
            img
        }
    }
}

impl ScreenTrait for StatsScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        ScreenManagementCmd::PushScreen(Box::new(RecordsScreen::new(self.gl_mtx.clone(), self.exit_request.clone())))
    }
    fn back(&mut self) -> ScreenManagementCmd {
        self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::None
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);
        let frame = (self.start.elapsed().as_millis() / 80) as usize % self.img.img_count;
        self.img.draw(texture_id, frame);

        self.screen_rendering.present();
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 0.5
    }
}