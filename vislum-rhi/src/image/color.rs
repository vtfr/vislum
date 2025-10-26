#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RGBA {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl From<[f32; 4]> for RGBA {
    fn from(arr: [f32; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }
}

impl From<[f32; 3]> for RGBA {
    fn from(arr: [f32; 3]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: 1.0,
        }
    }
}

impl Into<[f32; 4]> for RGBA {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Into<[f32; 3]> for RGBA {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}
