use std::time::Instant;
use crate::render::gl::types::GLfloat;

pub struct CircleAnimation {
    coefs: [(f64, f64, f64); 3],
    duration: f64,
    start: Instant,
}

impl CircleAnimation {
    /// points: value for t = 0, t = 0.5, t = 1
    pub fn new(duration: f64, points: [(f64, f64, f64); 3]) -> Self {
        //solve quadratic equation
        let mut coefs = [(0.0, 0.0, 0.0); 3];

        for i in 0..3 {
            let y0 = points[i].0;
            let y_half = points[i].1;
            let y1 = points[i].2;

            let a = 2.0 * y0 - 4.0 * y_half + 2.0 * y1;
            let b = -3.0 * y0 + 4.0 * y_half - y1;
            let c = y0;

            coefs[i] = (a, b, c);
        }
        CircleAnimation {
            coefs,
            duration,
            start: Instant::now(),
        }
    }

    pub fn start(&mut self) {
        self.start = Instant::now();
    }

    pub fn cur(&self) -> (GLfloat, GLfloat, GLfloat) {
        let elapsed_s = (Instant::now() - self.start).as_secs_f64();
        let t = (elapsed_s / self.duration).min(1.0);

        let c_x = self.coefs[0].0 * t*t + self.coefs[0].1 * t + self.coefs[0].2;
        let c_y = self.coefs[1].0 * t*t + self.coefs[1].1 * t + self.coefs[2].2;
        let c_r = self.coefs[2].0 * t*t + self.coefs[2].1 * t + self.coefs[2].2;

        (c_x as GLfloat, c_y as GLfloat, c_r as GLfloat)
    }
}