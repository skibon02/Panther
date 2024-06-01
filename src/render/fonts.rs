use std::collections::BTreeMap;
use std::sync::OnceLock;
use ab_glyph::{Font, FontRef, Point, PxScaleFont, Rect, ScaleFont};
use log::{error, debug, info};
use crate::render::gl;
use crate::render::gl::Gles2;
use crate::render::gl::types::GLuint;

static QUEENSIDES_FONT: &[u8] = include_bytes!("../../resources/fonts/queensides.ttf");
static SPARKY_STONES_FONT: &[u8] = include_bytes!("../../resources/fonts/SparkyStones.ttf");

#[derive(Clone)]
pub struct GlyphParams {
    pub texture_rect: Rect,
    pub h_advance: f32,
    pub v_advance: f32,
    pub h_side_bearing: f32,
    pub v_side_bearing: f32,
}

#[derive(Clone)]
pub struct FontData {
    pub texture_id: GLuint,
    pub glyph_params: BTreeMap<char, GlyphParams>,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    font: PxScaleFont<FontRef<'static>>
}

impl FontData {
    pub fn kern_space(&self, char1: char, char2: char) -> f32 {
        self.font.kern(self.font.glyph_id(char1), self.font.glyph_id(char2))
    }

    pub fn single_width(&self) -> f32 {
        1.0 / GRID_SIZE as f32
    }
    pub fn single_height(&self) -> f32 {
        1.0 / GRID_SIZE as f32
    }
}


pub struct FontLoader {
    fonts: BTreeMap<String, FontData>
}

const GLYPH_CELL_SIZE: usize = 300;
const GLYPH_RASTER_SIZE: f32 = 300.0;
const GRID_SIZE: usize = 11;

pub fn load_font(gl: &Gles2, font: &'static [u8]) -> FontData {
    let font = FontRef::try_from_slice(font).unwrap().into_scaled(GLYPH_RASTER_SIZE);

    let mut glyph_params = BTreeMap::new();

    let ascent = font.ascent() / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);
    let descent = font.descent() / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);
    let line_gap = font.line_gap() / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);
    let height = font.height() / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);

    debug!("Font loaded! Ascent: {}, Descent: {}, Line gap: {}, Height: {}",
          ascent, descent, line_gap, height);  // Load all characters into grid GRID_SIZE x GRID_SIZE

    let mut i = 1;
    let mut j = 1;
    let mut buf = vec![0u8; GRID_SIZE * GRID_SIZE * GLYPH_CELL_SIZE * GLYPH_CELL_SIZE];
    for c in ('A'..='Z').chain('a'..='z').chain('0'..='9').chain([',', '.', '!', '*', '\'', '?', ':', '-', '(', ')', '+', '/'].into_iter()) {

        let glyph_id = font.glyph_id(c);
        let glyph = glyph_id
            .with_scale_and_position(GLYPH_RASTER_SIZE,
                 ab_glyph::point(i as f32 * GLYPH_CELL_SIZE as f32,
                                 (j + 1) as f32 * GLYPH_CELL_SIZE as f32));

        let outline_glyph = font.outline_glyph(glyph).unwrap_or_else(|| {

            let glyph_id = font.glyph_id('x');
            let glyph = glyph_id
                .with_scale_and_position(GLYPH_RASTER_SIZE,
                                         ab_glyph::point(i as f32 * GLYPH_CELL_SIZE as f32,
                                                         (j + 1) as f32 * GLYPH_CELL_SIZE as f32));

            font.outline_glyph(glyph).unwrap()
        });
        let px_bounds = outline_glyph.px_bounds();
        debug!("{}: px_bounds: {:?}", c, px_bounds);

        let v_advance = font.v_advance(glyph_id) / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32); // should be 0, silly one
        let h_advance = font.h_advance(glyph_id) / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);

        // let v_side_bearing = font.v_side_bearing(glyph_id) / (FONT_RASTER_SIZE as f32 * GRID_SIZE as f32);
        let v_side_bearing = 1.0 / GRID_SIZE as f32 * (j+1) as f32 + (font.v_side_bearing(glyph_id) - px_bounds.max.y) / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);
        let h_side_bearing = -1.0 / GRID_SIZE as f32 * (i) as f32 + (font.h_side_bearing(glyph_id) + px_bounds.min.x) / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);

        let frac_w = px_bounds.width() / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);
        let frac_h = px_bounds.height() / (GLYPH_CELL_SIZE as f32 * GRID_SIZE as f32);

        // fraction 0..1 in the whole texture
        let texture_rect = Rect {
            min: Point {
                x: 1.0 / GRID_SIZE as f32 * i as f32,
                y: 1.0 / GRID_SIZE as f32 * j as f32
            },
            max: Point {
                x: 1.0 / GRID_SIZE as f32 * i as f32 + frac_w,
                y: 1.0 / GRID_SIZE as f32 * j as f32 + frac_h
            }
        };

        outline_glyph.draw(|x, y, v| {
            let x = x + i * GLYPH_CELL_SIZE as u32;
            let y = y + j * GLYPH_CELL_SIZE as u32;
            let idx = (y * GRID_SIZE as u32 * GLYPH_CELL_SIZE as u32 + x) as usize;
            buf[idx] = (v * 255.0) as u8;
        });


        glyph_params.insert(c, GlyphParams {
            texture_rect,
            v_advance,
            h_advance,
            v_side_bearing,
            h_side_bearing
        });

        i += 1;
        if i as usize == GRID_SIZE - 1 {
            i = 0;
            j += 1;
        }
        if j as usize >= GRID_SIZE - 1 {
            error!("Font grid too small, font loaded partially");
            break;
        }
    }

    let mut texture_id = 0;

    unsafe {
        gl.GenTextures(1, &mut texture_id);
        gl.BindTexture(gl::TEXTURE_2D, texture_id);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::R8 as i32,
            GRID_SIZE as i32 * GLYPH_CELL_SIZE as i32,
            GRID_SIZE as i32 * GLYPH_CELL_SIZE as i32,
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            buf.as_ptr() as *const _,
        );
    }

    FontData {
        texture_id,
        glyph_params,
        font,
        ascent,
        descent,
        line_gap
    }
}

impl FontLoader {
    pub fn new(gl: &Gles2) -> Self {
        let mut fonts = BTreeMap::new();

        fonts.insert("queensides".to_string(),
                     load_font(gl, QUEENSIDES_FONT));
        fonts.insert("sparky-stones".to_string(),
                     load_font(gl, SPARKY_STONES_FONT));


        FontLoader {
            fonts
        }
    }

    pub fn get_font(&self, name: &str) -> Option<FontData> {
        self.fonts.get(name).cloned()
    }
}

static FONTS: OnceLock<FontLoader> = OnceLock::new();

pub fn load_fonts(gl: &Gles2) {
    info!("Loading fonts started...");
    FONTS.get_or_init(|| FontLoader::new(gl));
    info!("Loading fonts finished!");
}
pub fn get_font(name: &str) -> Option<FontData> {
    FONTS.get().unwrap().get_font(name)
}