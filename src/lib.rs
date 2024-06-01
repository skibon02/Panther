use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use jni::JavaVM;
use jni::objects::{JObject, JObjectArray, JValue};
use jni::sys::jobject;
use log::{info, warn};
use parking_lot::Mutex;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder};
use winit::keyboard;
use winit::keyboard::NamedKey;
use winit::platform::android::activity::AndroidApp;
use winit::window::WindowId;
use crate::app::App;

pub mod app;
pub mod render;

pub static JNI_ENV: Mutex<Option<usize>> = Mutex::new(None);
pub static ACTIVITY_OBJ: Mutex<Option<JObject>> = Mutex::new(None);

fn set_max_framerate(android_app: &AndroidApp) {
    let vm = unsafe { JavaVM::from_raw(android_app.vm_as_ptr() as _) }.unwrap();
    let mut env = vm.get_env().unwrap();

    let activity = unsafe { JObject::from_raw(android_app.activity_as_ptr() as jobject) };

    let windowmanager = env.call_method(&activity, "getWindowManager", "()Landroid/view/WindowManager;", &[]).unwrap().l().unwrap();
    let display = env.call_method(&windowmanager, "getDefaultDisplay", "()Landroid/view/Display;", &[]).unwrap().l().unwrap();
    let supported_modes = env.call_method(&display, "getSupportedModes", "()[Landroid/view/Display$Mode;", &[]).unwrap().l().unwrap();
    let supported_modes = JObjectArray::from(supported_modes);
    let length = env.get_array_length(&supported_modes).unwrap();
    info!("Found {} supported modes", length);
    let mut modes = Vec::new();
    for i in 0..length {
        let mode = env.get_object_array_element(&supported_modes, i).unwrap();
        let height = env.call_method(&mode, "getPhysicalHeight", "()I", &[]).unwrap().i().unwrap();
        let width = env.call_method(&mode, "getPhysicalWidth", "()I", &[]).unwrap().i().unwrap();
        let refresh_rate = env.call_method(&mode, "getRefreshRate", "()F", &[]).unwrap().f().unwrap();
        let index = env.call_method(&mode, "getModeId", "()I", &[]).unwrap().i().unwrap();
        modes.push((index, refresh_rate));
        info!("Mode {}: {}x{}@{}", index, width, height, refresh_rate);
    }

    let max_framerate_mode = modes.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    info!("Max framerate: {}", max_framerate_mode.1);

    let preferred_id = 1;

    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[]).unwrap().l().unwrap();

    let layout_params_class = env.find_class("android/view/WindowManager$LayoutParams").unwrap();
    let layout_params = env.call_method(window, "getAttributes", "()Landroid/view/WindowManager$LayoutParams;", &[]).unwrap().l().unwrap();

    let preferred_display_mode_id_field_id = env.get_field_id(layout_params_class, "preferredDisplayModeId", "I").unwrap();
    env.set_field_unchecked(&layout_params, preferred_display_mode_id_field_id, JValue::from(preferred_id)).unwrap();

    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[]).unwrap().l().unwrap();
    env.call_method(window, "setAttributes", "(Landroid/view/WindowManager$LayoutParams;)V", &[(&layout_params).into()]).unwrap();


    //Register GPS
    info!("Registering GPS...");

    let raw_env = env.get_raw() as usize;

    JNI_ENV.lock().replace(raw_env);
    ACTIVITY_OBJ.lock().replace(activity);
}

struct WinitApp {
    app: Option<App>,
    exit_request: Arc<AtomicBool>,
}

impl WinitApp {
    pub fn new() -> Self {
        let exit_request = Arc::new(AtomicBool::new(false));
        let app = Some(App::new(exit_request.clone()));

        Self {
            app,
            exit_request
        }
    }
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(app) = &mut self.app {
            info!("[apploop] Resumed");
            app.resume(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if self.exit_request.load(Ordering::Relaxed) {
            info!("[apploop] Exit requested! Dropping app...");
            event_loop.exit();
            self.app = None;
        }
        if let Some(app) = &mut self.app {
            info!("[apploop] New event: {:?}", event);
            match event {
                WindowEvent::KeyboardInput{
                    event: winit::event::KeyEvent {
                        logical_key: keyboard::Key::Named(NamedKey::GoBack),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    app.handle_back_button();
                }

                WindowEvent::Touch(winit::event::Touch {
                    phase,
                    id,
                    location,
                    ..
                }) => {
                    app.handle_touch(id, location, phase);
                }
                WindowEvent::CloseRequested => {
                    app.handle_close_request();
                },
                WindowEvent::RedrawRequested => {
                    app.handle_redraw_request();
                }
                WindowEvent::Resized(new_size) => {
                    app.handle_resize(new_size);
                }
                _ => {}
            }
        }
        else {
            warn!("Exiting... Event ignored: {:?}", event);
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        info!("[apploop] Suspended");
        if let Some(app) = &mut self.app {
            app.handle_suspend();
        }
    }
}

fn run(event_loop: EventLoop<()>) {
    let mut winit_app = WinitApp::new();

    info!("Running mainloop...");
    event_loop.run_app(&mut winit_app).unwrap();
    info!("Mainloop exited");
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );

    set_max_framerate(&app);

    let event_loop = EventLoopBuilder::default().with_android_app(app).build().unwrap();
    run(event_loop);
}