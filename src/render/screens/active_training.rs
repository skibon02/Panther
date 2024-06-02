use std::sync::Arc;
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


use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jdouble;
use lazy_static::lazy_static;
use log::{info, warn};
use parking_lot::Mutex;
use crate::render::screens::paused_screen::PausedScreen;

#[derive(Clone)]
pub struct LocationMetric {
    latitude: f64,
    longitude: f64,
    accuracy: f64,
    timestamp: f64,
}


pub struct GpsData {
    available_since: Option<Instant>,
    gps_acc_good: bool,
    last_known_acc: Option<f64>,
    initial_metric: Option<LocationMetric>,
    all_metrics: Vec<LocationMetric>,
    total_time: f64,
    total_distance: f64,
    paused: bool,
}

impl GpsData {
    fn new() -> Self {
        GpsData {
            available_since: None,
            initial_metric: None,
            gps_acc_good: false,
            last_known_acc: None,
            all_metrics: Vec::new(),
            total_time: 0.0,
            total_distance: 0.0,
            paused: false
        }
    }

    fn update_location(&mut self, latitude: f64, longitude: f64, accuracy: f64, timestamp: f64) {
        if self.paused {
            return;
        }

        if self.available_since.is_none() {
            self.available_since = Some(Instant::now());
        }

        let metric = LocationMetric {
            latitude,
            longitude,
            accuracy,
            timestamp,
        };


        self.last_known_acc = Some(accuracy);
        if let Some(available_since) = &self.available_since {
            let elapsed = Instant::now().duration_since(*available_since).as_secs_f64();
            if elapsed < 10.0 {
                return;
            }
        }

        if accuracy <= 5.5 {
            self.gps_acc_good = true;
            if self.initial_metric.is_none(){
                self.initial_metric = Some(metric.clone());
                info!("Initial metric recorded! Training is started");
            }

            if let Some(initial_metric) = &self.initial_metric {
                let lat_offset = (metric.latitude - initial_metric.latitude) * 111_319.5;
                let lon_offset = (metric.longitude - initial_metric.longitude) * (111_319.5 * initial_metric.latitude.to_radians().cos());


                if let Some(prev_metric) = self.all_metrics.last() {
                    let distance = ((lat_offset - prev_metric.latitude).powi(2) + (lon_offset - prev_metric.longitude).powi(2)).sqrt();
                    self.total_distance += distance;

                    let time_diff = metric.timestamp - prev_metric.timestamp;
                    self.total_time += time_diff;
                }
                info!("Offset: Lat: {}, Lon: {}", lat_offset, lon_offset);

                self.all_metrics.push(LocationMetric {
                    latitude: lat_offset,
                    longitude: lon_offset,
                    accuracy: metric.accuracy,
                    timestamp: metric.timestamp,
                });

                info!("\nTotal time: {}, total distance: {}", self.total_time, self.total_distance);
                info!("\nAvg speed: {}", self.avg_speed());
            }
        }
        else {
            self.gps_acc_good = false;
        }
    }

    fn has_initial_metric(&self) -> bool {
        self.initial_metric.is_some()
    }

    fn gps_online(&self) -> bool {
        self.available_since.is_some()
    }

    fn is_good_accuracy(&self) -> bool {
        self.gps_acc_good
    }

    pub fn avg_speed(&self) -> f64 {
        if self.total_time == 0.0 {
            return 0.0;
        }
        self.total_distance / self.total_time
    }

    pub fn total_time(&self) -> f64 {
        self.total_time
    }

    pub fn total_distance(&self) -> f64 {
        self.total_distance
    }

    fn get_last_known_acc(&self) -> Option<f64> {
        self.last_known_acc
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.initial_metric = None;
        self.all_metrics.clear();
        self.last_known_acc = None;
        self.available_since = None;
    }

    pub(crate) fn resume(&mut self) {
        self.paused = false;
    }
}

lazy_static! {
    pub static ref GPS_DATA: Mutex<GpsData> = Mutex::new(GpsData::new());
}

#[no_mangle]
pub extern "system" fn Java_com_skygrel_panther_LocationHelper_onLocationUpdate(
    _env: JNIEnv,
    _class: JClass,
    latitude: jdouble,
    longitude: jdouble,
    acc: jdouble,
    timestamp: jdouble
) {
    // Handle the location update
    println!("Received location update:\n{}:  Lat {}, Lon {}. Acc: {}", timestamp,  latitude, longitude, acc);
    let mut gps_data = GPS_DATA.lock();
    gps_data.update_location(latitude, longitude, acc, timestamp);
}


#[no_mangle]
pub extern "system" fn Java_com_skygrel_panther_LocationHelper_onProviderEnabled(
    _env: JNIEnv,
    _class: JClass,
) {
    info!("GPS provider enabled!");
    let mut gps_data = GPS_DATA.lock();
    gps_data.available_since = Some(Instant::now());
}


#[no_mangle]
pub extern "system" fn Java_com_skygrel_panther_LocationHelper_onProviderDisabled(
    _env: JNIEnv,
    _class: JClass,
) {
    warn!("GPS provider disabled!");
    let mut gps_data = GPS_DATA.lock();
    gps_data.available_since = None;
}

pub struct ActiveTrainingScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,

    screen_rendering: ScreenRendering,

    play: Image,
    walking_gif: AnimatedImage,

    exit_request: Arc<AtomicBool>,
    start: Instant,

    gps_text: TextBox,
    gps_acc_text: TextBox,

    tab1: Tab,
    tab2: Tab,
    tab3: Tab,

    tab_label_1: TextBox,
    tab_label_2: TextBox,
    tab_label_3: TextBox,

    //total
    total_time_val: TextBox,
    total_time_units: TextBox,

    total_dist_val: TextBox,
    total_dist_units: TextBox,
}

impl ActiveTrainingScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl.clone(), (0.4, 0.3, 0.5));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        let sparky_stones = get_font("sparky-stones").unwrap();
        let queensides = get_font("queensides").unwrap();

        let mut pos = FreePosition::new().bottom(-0.5).left(0.0).width(1.0)
            .height(1.8);
        let tab1 = Tab::new(gl.clone(), (0.05, 0.2, 0.3), pos, 0.2);

        pos = pos.height(1.45);
        let tab2 = Tab::new(gl.clone(), (0.15, 0.1, 0.3), pos, 0.4);

        pos = pos.height(1.1);
        let tab3 = Tab::new(gl.clone(), (0.3, 0.05, 0.3), pos, 0.6);

        let tab_label_1 = TextBox::new(gl.clone(), sparky_stones.clone(), "total".to_string(), (0.25, 1.21), 0.5, 0);
        let tab_label_2 = TextBox::new(gl.clone(), sparky_stones.clone(), "cur".to_string(), (0.47, 0.86), 0.5, 0);
        let tab_label_3 = TextBox::new(gl.clone(), sparky_stones.clone(), "avg".to_string(), (0.67, 0.51), 0.5, 0);

        let play = Image::new(gl.clone(), get_image("play").unwrap(),
                              FixedPosition::new().bottom(1.7).width(0.25).left(0.15), Some((0.1, 0.9, 0.3)));
        let walking_gif = AnimatedImage::new(gl.clone(), get_gif("walking").unwrap(),
                                             FixedPosition::new().bottom(1.7).width(0.55).left(0.45), 0.08);

        let total_time_val = TextBox::new(gl.clone(), queensides.clone(), "-".to_string(), (0.1, 1.05), 1.0, 0);
        let total_time_units = TextBox::new(gl.clone(), queensides.clone(), "min:sec".to_string(), (0.1, 0.95), 1.0, 0);

        let total_dist_val = TextBox::new(gl.clone(), queensides.clone(), "-".to_string(), (0.75, 1.05), 1.0, 0);
        let total_dist_units = TextBox::new(gl.clone(), queensides.clone(), "m".to_string(), (0.76, 0.95), 1.0, 0);

        let gps_text = TextBox::new(gl.clone(), queensides.clone(), "GPS status: waiting...".to_string(), (0.03, 1.55), 0.8, 1);

        let gps_acc_text = TextBox::new(gl.clone(), queensides.clone(), "ACC: unknown".to_string(), (0.03, 1.45), 0.6, 0);


        //reset training
        let mut gps_data = GPS_DATA.lock();
        *gps_data = GpsData::new();



        ActiveTrainingScreen {
            gl,
            bg_squad: squad,

            exit_request,
            start: Instant::now(),
            screen_rendering,

            gps_text,
            gps_acc_text,

            tab1,
            tab2,
            tab3,

            tab_label_1,
            tab_label_2,
            tab_label_3,

            play,
            walking_gif,

            total_time_val,
            total_time_units,
            total_dist_val,
            total_dist_units,
        }
    }
}

impl ScreenTrait for ActiveTrainingScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        if pos.1 > 1.7 && pos.1 < 1.95 && pos.0 > 0.15 && pos.0 < 0.4  {
            let mut gps_data = GPS_DATA.lock();
            gps_data.pause();
            return ScreenManagementCmd::PushScreen(Box::new(PausedScreen::new(self.gl.clone(), self.exit_request.clone())));
        }
        ScreenManagementCmd::None
    }
    fn back(&mut self) -> ScreenManagementCmd {
        let mut gps_data = GPS_DATA.lock();
        gps_data.pause();
        ScreenManagementCmd::PushScreen(Box::new(PausedScreen::new(self.gl.clone(), self.exit_request.clone())))
    }
    #[profiling::function]
    fn update(&mut self) -> ScreenManagementCmd {
        let gps_data = GPS_DATA.lock();

        if gps_data.gps_online() {
            if gps_data.has_initial_metric() {
                if gps_data.is_good_accuracy() {
                    self.gps_text.set_text("GPS status: training online".to_string());
                    self.total_dist_val.set_text(format!("{:.2}", gps_data.total_distance()));
                }
                else {
                    self.gps_text.set_text("GPS status: training online (bad acc)".to_string());
                }

                let total_time = gps_data.total_time();
                let secs = total_time as u64;
                let mins = secs / 60;
                let secs = secs % 60;
                self.total_time_val.set_text(format!("{:02}:{:02}", mins, secs));
            }
            else {
                self.gps_text.set_text("GPS status: waiting (bad acc)".to_string());
            }

            if gps_data.is_good_accuracy() {
                self.gps_acc_text.set_text(format!("ACC: +-{:.2}m", gps_data.get_last_known_acc().unwrap()));
            }
            else {
                self.gps_acc_text.set_text(format!("ACC: +-{:.2}m (not enough)", gps_data.get_last_known_acc().unwrap()));
            }
        }
        else {
            self.gps_text.set_text("GPS status: offline".to_string());
        }
        ScreenManagementCmd::None
    }
    #[profiling::function]
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);

        self.play.draw(texture_id);
        self.walking_gif.draw(texture_id);

        self.gps_text.draw(texture_id);
        self.gps_acc_text.draw(texture_id);

        self.tab1.draw(texture_id);
        self.tab2.draw(texture_id);
        self.tab3.draw(texture_id);

        self.tab_label_1.draw(texture_id);
        self.tab_label_2.draw(texture_id);
        self.tab_label_3.draw(texture_id);

        self.total_time_val.draw(texture_id);
        self.total_time_units.draw(texture_id);
        self.total_dist_val.draw(texture_id);
        self.total_dist_units.draw(texture_id);

        self.screen_rendering.present();
    }
    fn scroll(&mut self, _pos: (f64, f64)) {

    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 1.0
    }
}