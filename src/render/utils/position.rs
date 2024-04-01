/// Box positioning in wh units
pub struct FreePosition {
    left: Option<f64>,
    bottom: Option<f64>,
    width: Option<f64>,
    height: Option<f64>
}

impl FreePosition {
    pub fn new() -> FreePosition {
        FreePosition {
            left: None,
            bottom: None,
            width: None,
            height: None
        }
    }
    pub fn left(mut self, v: f64) -> FreePosition {
        self.left = Some(v);
        self
    }
    pub fn bottom(mut self, v: f64) -> FreePosition {
        self.bottom = Some(v);
        self
    }
    pub fn width(mut self, v: f64) -> FreePosition {
        self.width = Some(v);
        self
    }
    pub fn height(mut self, v: f64) -> FreePosition {
        self.height = Some(v);
        self
    }

    pub fn get(&self) -> (f64, f64, f64, f64) {
        (self.left.unwrap_or(0.0), self.bottom.unwrap_or(0.0), self.width.unwrap(), self.height.unwrap())
    }
}

impl From<(f64, f64, f64, f64)> for FreePosition {
    fn from(value: (f64, f64, f64, f64)) -> Self {
        Self {
            left: Some(value.0),
            bottom: Some(value.1),
            width: Some(value.2),
            height: Some(value.3),
        }
    }
}

/// Position for box with fixed width/height ratio
pub struct FixedPosition {
    left: Option<f64>,
    bottom: Option<f64>,
    width: Option<f64>,
    height: Option<f64>
}

impl FixedPosition {
    pub fn new() -> FixedPosition {
        FixedPosition {
            left: None,
            bottom: None,
            width: None,
            height: None
        }
    }

    pub fn left(mut self, v: f64) -> FixedPosition {
        self.left = Some(v);
        self
    }
    pub fn bottom(mut self, v: f64) -> FixedPosition {
        self.bottom = Some(v);
        self
    }
    pub fn width(mut self, v: f64) -> FixedPosition {
        self.width = Some(v);
        self
    }
    pub fn height(mut self, v: f64) -> FixedPosition {
        self.height = Some(v);
        self
    }

    pub fn get(&self, ratio: f64) -> (f64, f64, f64, f64) {
        let left = self.left.unwrap_or(0.0);
        let bottom = self.bottom.unwrap_or(0.0);
        match (self.width, self.height) {
            (Some(w), None) => {
                let h = w / ratio;
                (left, bottom, w, h)
            },
            (None, Some(h)) => {
                let w = h * ratio;
                (left, bottom, w, h)
            },
            (Some(w), Some(h)) => {
                if h / w != ratio {
                    panic!("width and height must have the same ratio")
                }
                (left, bottom, w, h)
            },
            _ => panic!("width or height must be set")
        }
    }
}

