use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{check_gl_errors, gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::objects::r#box::Squad;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::stats::StatsScreen;
use crate::render::utils::circle_animation::CircleAnimation;


pub struct MainScreen {
    gl_mtx: Arc<Mutex<gl::Gl>>,
    bg_squad: Squad,
    screen_rendering: ScreenRendering,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl MainScreen {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl_mtx.clone(), (0.4, 0.5, 0.9));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl_mtx.clone(), dims, circ_anim);

        MainScreen {
            gl_mtx,
            bg_squad: squad,
            exit_request,
            start: Instant::now(),
            screen_rendering
        }
    }
}

impl ScreenTrait for MainScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        ScreenManagementCmd::PushScreen(Box::new(StatsScreen::new(self.gl_mtx.clone(), self.exit_request.clone())))
    }
    fn back(&mut self) -> ScreenManagementCmd {
        self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::None
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);

        self.screen_rendering.present();
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 0.5
    }
}