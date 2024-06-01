use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::{OnceLock};
use image::{AnimationDecoder, DynamicImage, GenericImageView};
use image::codecs::gif::GifDecoder;
use log::{debug, info};
use crate::render::gl;
use crate::render::gl::Gles2;
use crate::render::gl::types::GLuint;

pub static PANTHER_HD: &[u8] = include_bytes!("../../resources/panther_hd.png");
pub static PANTHER_LOGO: &[u8] = include_bytes!("../../resources/panther_logo.png");
pub static HOME: &[u8] = include_bytes!("../../resources/home.png");
pub static RECORDS: &[u8] = include_bytes!("../../resources/records.png");
pub static STATS: &[u8] = include_bytes!("../../resources/stats.png");
pub static PLAY: &[u8] = include_bytes!("../../resources/play.png");

pub static GIF_RUNNING: &[u8] = include_bytes!("../../resources/running.gif");
pub static GIF_WALKING: &[u8] = include_bytes!("../../resources/walking.gif");
pub static GIF_EYES: &[u8] = include_bytes!("../../resources/eyes.gif");

#[derive(Copy, Clone)]
pub struct ImageData {
    pub texture_id: GLuint,
    pub width: u32,
    pub height: u32,
}

pub struct ImageLoader {
    images: BTreeMap<String, ImageData>,
    gifs: BTreeMap<String, Vec<ImageData>>,
}

fn load_image(gl: &Gles2, image: DynamicImage) -> ImageData {
    let (width, height) = image.dimensions();
    let image_data = image.to_rgba8().into_raw();

    debug!("Image decoded! Width: {}, height: {}", width, height);

    let mut texture_id = 0;
    unsafe {
        gl.GenTextures(1, &mut texture_id);
        gl.BindTexture(gl::TEXTURE_2D, texture_id);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            image_data.as_ptr() as *const _,
        );
    }

    ImageData {
        texture_id,
        width,
        height
    }
}

//just bunch of calls to load_image
fn load_gif(gl: &Gles2, bytes: &'static[u8]) -> Vec<ImageData> {
    let gif_images = GifDecoder::new(Cursor::new(bytes)).unwrap()
        .into_frames().collect_frames().unwrap()
        .into_iter().map(|i| DynamicImage::ImageRgba8(i.into_buffer()));

    gif_images.map(|i| load_image(gl, i)).collect()
}

impl ImageLoader {
    pub fn new(gl: &Gles2) -> Self {
        let mut images = BTreeMap::new();

        images.insert("panther_hd".to_string(), load_image(gl, image::load_from_memory(PANTHER_HD).unwrap()));
        images.insert("panther_logo".to_string(), load_image(gl, image::load_from_memory(PANTHER_LOGO).unwrap()));
        images.insert("home".to_string(), load_image(gl, image::load_from_memory(HOME).unwrap()));
        images.insert("records".to_string(), load_image(gl, image::load_from_memory(RECORDS).unwrap()));
        images.insert("stats".to_string(), load_image(gl, image::load_from_memory(STATS).unwrap()));
        images.insert("play".to_string(), load_image(gl, image::load_from_memory(PLAY).unwrap()));

        let mut gifs = BTreeMap::new();

        gifs.insert("running".to_string(), load_gif(gl, GIF_RUNNING));
        gifs.insert("walking".to_string(), load_gif(gl, GIF_WALKING));
        gifs.insert("eyes".to_string(), load_gif(gl, GIF_EYES));



        ImageLoader {
            images,
            gifs
        }
    }

    pub fn get_image(&self, name: &str) -> Option<ImageData> {
        self.images.get(name).cloned()
    }

    pub fn get_gif(&self, name: &str) -> Option<Vec<ImageData>> {
        self.gifs.get(name).cloned()
    }
}

static IMAGES: OnceLock<ImageLoader> = OnceLock::new();

pub fn load_images(gl: &Gles2) {
    info!("Loading images & gifs started...");
    IMAGES.get_or_init(|| ImageLoader::new(gl));
    info!("Loading images & gifs finished!");
}
pub fn get_image(name: &str) -> Option<ImageData> {
    IMAGES.get().unwrap().get_image(name)
}
pub fn get_gif(name: &str) -> Option<Vec<ImageData>> {
    IMAGES.get().unwrap().get_gif(name)
}