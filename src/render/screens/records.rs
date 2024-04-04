use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::images::{get_gif, get_image, PANTHER_HD};
use crate::render::objects::animated_image::AnimatedImage;
use crate::render::objects::image::Image;
use crate::render::objects::r#box::Squad;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::main::MainScreen;
use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::FixedPosition;


pub struct RecordsScreen {
    gl_mtx: Arc<Mutex<gl::Gl>>,
    bg_squad: Squad,
    bg_img: AnimatedImage,

    screen_rendering: ScreenRendering,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl RecordsScreen {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl_mtx.clone(), (0.6, 0.8, 0.2));

        let img_pos = FixedPosition::new().width(0.5).left(0.5).bottom(0.0);
        let img = AnimatedImage::new(gl_mtx.clone(), get_gif("running").unwrap(), img_pos);

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl_mtx.clone(), dims, circ_anim);

        RecordsScreen {
            gl_mtx,
            bg_squad: squad,
            bg_img: img,

            exit_request,
            start: Instant::now(),
            screen_rendering
        }
    }
}

impl ScreenTrait for RecordsScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        ScreenManagementCmd::PushScreen(Box::new(MainScreen::new(self.gl_mtx.clone(), self.exit_request.clone())))
    }
    fn back(&mut self) -> ScreenManagementCmd {
        self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::None
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);
        let img_frame = (self.start.elapsed().as_millis() / 50) as usize % self.bg_img.img_count;
        self.bg_img.draw(texture_id, img_frame);

        self.screen_rendering.present();
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 0.5
    }
}