use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use jni::{JavaVM, JNIEnv};
use jni::objects::{JClass, JObject, JObjectArray, JValue};
use jni::sys::{jdouble, jobject};
use log::info;
use raw_window_handle::HasRawDisplayHandle;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::android::activity::AndroidApp;
use crate::app::App;

pub mod app;
pub mod render;

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

    // get activity field locationManager
    let location_helper_instance = env.get_field(activity, "locationHelper", "Lcom/skygrel/panther/LocationHelper;").unwrap().l().unwrap();

    // Now call the startLocationUpdates method
    env.call_method(location_helper_instance, "startLocationUpdates", "()V", &[])
        .expect("Failed to call startLocationUpdates");
}


#[no_mangle]
pub extern "system" fn Java_com_skygrel_panther_LocationHelper_onLocationUpdate(
    env: JNIEnv,
    class: JClass,
    latitude: jdouble,
    longitude: jdouble,
) {
    // Handle the location update
    println!("Received location update: Lat {}, Lon {}", latitude, longitude);
}

fn run(event_loop: EventLoop<()>) {
    info!("Running mainloop...");

    let raw_display = event_loop.raw_display_handle();

    let exit_request = Arc::new(AtomicBool::new(false));
    let mut app = Some(App::new(raw_display, exit_request.clone()));




    event_loop.run(move |event, event_loop, control_flow| {
        // log::debug!("Received Winit event: {event:?}");

        *control_flow = ControlFlow::Wait;
        if exit_request.load(Ordering::Relaxed) {
            info!("Exit requested! Dropping app...");
            *control_flow = ControlFlow::Exit;
            app = None;
        }
        if let Some(app) = app.as_mut() {
            match event {
                Event::Resumed => {
                    app.resume(event_loop);
                }
                Event::Suspended => {
                    log::trace!("Suspended, dropping surface state...");
                    app.handle_suspend();
                }

                Event::WindowEvent {
                    event: winit::event::WindowEvent::KeyboardInput{
                        input: winit::event::KeyboardInput {
                            scancode: 0,
                            state: winit::event::ElementState::Pressed,
                            ..
                        },
                        ..
                    },
                    ..
                } => {
                    app.handle_back_button();
                }

                Event::WindowEvent {
                    event: WindowEvent::Touch(winit::event::Touch {
                                                  phase,
                                                  id,
                                                  location,
                                                  ..
                                              }),
                    ..
                } => {
                    app.handle_touch(id, location, phase);
                }

                Event::RedrawRequested(_) => {
                    app.handle_redraw_request();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    app.handle_close_request();
                }
                _ => {}
            }
        }
        else {
            log::warn!("App exiting... Event ignored: {:?}", event);
        }
    });
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );

    set_max_framerate(&app);

    let event_loop = EventLoopBuilder::new().with_android_app(app).build();
    run(event_loop);
}