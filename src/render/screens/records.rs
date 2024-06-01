use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use log::{info, warn};
use parking_lot::{Mutex, MutexGuard};
use crate::render::{ANDROID_DATA_PATH, gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::fonts::get_font;
use crate::render::images::get_image;

use crate::render::objects::image::Image;
use crate::render::objects::r#box::Squad;
use crate::render::objects::textbox::TextBox;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::active_training::GpsData;
use crate::render::screens::main::MainScreen;
use crate::render::screens::stats::StatsScreen;
use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::{FixedPosition, FreePosition};


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Record {
    timestamp: f64,
    distance: f64,
    time: f64,
    speed: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Records {
    records: Vec<Record>,
    pub total_distance: f64,
    pub total_time: f64,
    pub avg_speed: f64,
}

pub fn push_new_record(gps_data: &MutexGuard<GpsData>) {
    let mut records = RECORDS_LIST.lock();

    //UNIX EPOCH
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let record = Record {
        timestamp: now,
        distance: gps_data.total_distance(),
        time: gps_data.total_time(),
        speed: gps_data.avg_speed(),
    };

    records.total_distance += record.distance;
    records.total_time += record.time;
    if records.total_time == 0.0 {
        records.avg_speed = 0.0;
    }
    else {
        records.avg_speed = records.total_distance / records.total_time;
    }

    records.records.push(record);

    let records_path = format!("{}/records.json", ANDROID_DATA_PATH);

    //write test content
    info!("Opening file {}", records_path);
    let Ok(mut file) = File::create(&records_path) else {
        warn!("File open failed! Creating empty records object...");
        return;
    };
    info!("writing to file...");
    if let Err(e) = file.write_all(serde_json::to_string(&*records).unwrap().as_bytes()) {
        warn!("Writing to file failed! {:?}", e);
    }
}

lazy_static!(
    pub static ref RECORDS_LIST: Mutex<Records> = Mutex::new(Records {
        records: vec![],
        total_distance: 0.0,
        total_time: 0.0,
        avg_speed: 0.0,
    });
);

pub struct RecordsScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,

    screen_rendering: ScreenRendering,

    logo: Image,

    record_info: TextBox,
    record_square: Squad,

    bottom_home_text: TextBox,
    bottom_records_text: TextBox,
    bottom_stats_text: TextBox,

    home_icon: Image,
    records_icon: Image,
    stats_icon: Image,

    exit_request: Arc<AtomicBool>,
    start: Instant,

    scroll_offset: f64,
}

impl RecordsScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl.clone(), (0.6, 0.8, 0.2));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        let font = get_font("queensides").unwrap();

        let logo = Image::new(gl.clone(), get_image("panther_logo").unwrap(),
                                                                         FixedPosition::new().bottom(1.75).width(0.25).left(0.65), Some((0.05, 0.06, 0.1)));

        let bottom_home_text = TextBox::new(gl.clone(), font.clone(), "Home".to_string(), (0.2, 0.068), 0.45, 1);
        let bottom_records_text = TextBox::new(gl.clone(), font.clone(), "Records".to_string(), (0.44, 0.068), 0.45, 1);
        let bottom_stats_text = TextBox::new(gl.clone(), font.clone(), "Stats".to_string(), (0.72, 0.068), 0.45, 1);

        let home_icon = Image::new(gl.clone(), get_image("home").unwrap(),
                                   FixedPosition::new().bottom(0.12).height(0.08).left(0.2), Some((0.4, 0.2, 0.6)));
        let records_icon = Image::new(gl.clone(), get_image("records").unwrap(),
                                      FixedPosition::new().bottom(0.12).height(0.08).left(0.45), Some((1.0, 0.9, 1.0)));
        let stats_icon = Image::new(gl.clone(), get_image("stats").unwrap(),
                                    FixedPosition::new().bottom(0.12).height(0.08).left(0.715), Some((0.4, 0.5, 0.9)));

        let record_info = TextBox::new(gl.clone(), font.clone(), "Record 0".to_string(), (0.12, 1.5), 0.68, 1);
        let record_square = Squad::new(gl.clone(), (0.5, 0.3, 0.5, 1.0),
            FreePosition::new().bottom(1.38).left(0.1).width(0.8).height(0.2));

        RecordsScreen {
            gl,
            bg_squad: squad,

            exit_request,
            start: Instant::now(),
            screen_rendering,

            logo,

            record_info,
            record_square,

            bottom_home_text,
            bottom_records_text,
            bottom_stats_text,

            home_icon,
            records_icon,
            stats_icon,

            scroll_offset: 0.0,
        }
    }
}

impl ScreenTrait for RecordsScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        if pos.1 < 0.25 {
            match pos.0 {
                x if x < 0.33 => {
                    ScreenManagementCmd::PushScreen(Box::new(MainScreen::new(self.gl.clone(), self.exit_request.clone())))
                }
                x if x < 0.66 => {
                    // ScreenManagementCmd::PushScreen(Box::new(RecordsScreen::new(self.gl.clone(), self.exit_request.clone())))
                    ScreenManagementCmd::None
                }
                _ => {
                    ScreenManagementCmd::PushScreen(Box::new(StatsScreen::new(self.gl.clone(), self.exit_request.clone())))
                }

            }
        }
        else {
            ScreenManagementCmd::None
        }
    }
    fn back(&mut self) -> ScreenManagementCmd {
        // self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::PushScreen(Box::new(MainScreen::new(self.gl.clone(), self.exit_request.clone())))
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);

        self.logo.draw(texture_id);

        let records = RECORDS_LIST.lock();
        for (i, record) in records.records.iter().enumerate() {
            let text = format!("Record {}\n{:.2}m in {:.2}s at {:.2}m/s", i, record.distance, record.time, record.speed);
            self.record_square.set_pos_y_offset(- 0.3 * i as f64 + self.scroll_offset);

            self.record_info.set_text(text);
            self.record_info.set_pos((0.12, 1.5 - 0.3 * i as f32 + self.scroll_offset as f32));

            self.record_square.draw(texture_id);
            self.record_info.draw(texture_id);
        }

        self.bottom_home_text.draw(texture_id);
        self.bottom_records_text.draw(texture_id);
        self.bottom_stats_text.draw(texture_id);

        self.home_icon.draw(texture_id);
        self.records_icon.draw(texture_id);
        self.stats_icon.draw(texture_id);

        self.screen_rendering.present();
    }
    fn scroll(&mut self, pos: (f64, f64)) {
        self.scroll_offset += pos.1;
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 1.0
    }
}