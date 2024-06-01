use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::fonts::get_font;
use crate::render::images::{get_gif, get_image};
use crate::render::objects::animated_image::AnimatedImage;
use crate::render::objects::image::Image;
use crate::render::objects::r#box::Squad;
use crate::render::objects::tab::Tab;
use crate::render::objects::textbox::TextBox;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};


use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::{FixedPosition, FreePosition};


use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jdouble;
use lazy_static::lazy_static;
use log::{info, warn};
use crate::render::screens::active_training::GPS_DATA;
use crate::render::screens::main::{MainScreen, stop_location_updates};
use crate::render::screens::records::push_new_record;

pub struct PausedScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,

    tab: Squad,

    screen_rendering: ScreenRendering,
    exit_request: Arc<AtomicBool>,

    tittle: TextBox,
    exit_but: TextBox,
    continue_but: TextBox,

    exit_bg: Squad,
    continue_bg: Squad,

    is_pause: bool
}

impl PausedScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let bg_squad = Squad::new_bg_alpha(gl.clone(), (0.0, 0.0, 0.0, 0.5));

        unsafe {
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let tab = Squad::new(gl.clone(), (0.4, 0.5, 0.9, 1.0),
            FreePosition::new().bottom(1.1).left(0.1).width(0.8).height(0.5));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        let font = get_font("queensides").unwrap();

        let tittle = TextBox::new(gl.clone(), font.clone(), "Paused...".to_string(), (0.15, 1.4), 2.0, 2);
        let exit_but = TextBox::new(gl.clone(), font.clone(), "Finish".to_string(), (0.15, 1.15), 1.2, 1);
        let continue_but = TextBox::new(gl.clone(), font.clone(), "Continue".to_string(), (0.55, 1.15), 1.2, 1);

        let exit_bg = Squad::new(gl.clone(), (0.8, 0.2, 0.2, 1.0),
                FreePosition::new().left(0.10).bottom(1.1).width(0.4).height(0.18));
        let continue_bg = Squad::new(gl.clone(), (0.2, 0.8, 0.2, 1.0),
                FreePosition::new().left(0.5).bottom(1.1).width(0.4).height(0.18));

        PausedScreen {
            gl,
            bg_squad,
            tab,

            exit_request,
            screen_rendering,

            tittle,
            exit_but,
            continue_but,

            exit_bg,
            continue_bg,

            is_pause: false
        }
    }

    fn paused(&mut self) {
        self.is_pause = !self.is_pause;
    }
}

impl ScreenTrait for PausedScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        // continue button
        if pos.0 > 0.5 && pos.0 < 0.9 && pos.1 > 1.1 && pos.1 < 1.28 {
            let mut gps_data = GPS_DATA.lock().unwrap();
            gps_data.resume();
            return ScreenManagementCmd::PopScreen;
        }
        // exit button
        if pos.0 > 0.1 && pos.0 < 0.5 && pos.1 > 1.1 && pos.1 < 1.28 {
            let mut gps_data = GPS_DATA.lock().unwrap();
            gps_data.pause();
            push_new_record(&gps_data);
            stop_location_updates();
            return ScreenManagementCmd::PushScreen(Box::new(MainScreen::new(self.gl.clone(), self.exit_request.clone())));
        }

        ScreenManagementCmd::None
    }
    fn back(&mut self) -> ScreenManagementCmd {
        let mut gps_data = GPS_DATA.lock().unwrap();
        gps_data.resume();
        ScreenManagementCmd::PopScreen
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);
        self.tab.draw(texture_id);

        self.exit_bg.draw(texture_id);
        self.continue_bg.draw(texture_id);

        self.tittle.draw(texture_id);
        self.exit_but.draw(texture_id);
        self.continue_but.draw(texture_id);

        self.screen_rendering.present();
    }
    fn scroll(&mut self, _pos: (f64, f64)) {

    }
    fn is_expanded(&self) -> bool {
        false
    }
}