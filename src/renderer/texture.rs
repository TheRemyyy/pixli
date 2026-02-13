//! Texture loading and management.

use crate::math::Color;

/// Texture data.
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Texture {
    /// Create empty texture.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![255; (width * height * 4) as usize],
        }
    }

    /// Create texture filled with color.
    pub fn from_color(width: u32, height: u32, color: Color) -> Self {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;

        let mut data = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }

        Self {
            width,
            height,
            data,
        }
    }

    /// Load texture from file.
    pub fn load(path: &str) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("Failed to load texture: {}", e))?;

        let img = img.to_rgba8();
        let (width, height) = img.dimensions();
        let data = img.into_raw();

        Ok(Self {
            width,
            height,
            data,
        })
    }

    /// Load texture from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("Failed to load texture from bytes: {}", e))?;

        let img = img.to_rgba8();
        let (width, height) = img.dimensions();
        let data = img.into_raw();

        Ok(Self {
            width,
            height,
            data,
        })
    }

    /// Create checkerboard pattern.
    pub fn checkerboard(size: u32, tile_size: u32, color1: Color, color2: Color) -> Self {
        let mut data = Vec::with_capacity((size * size * 4) as usize);

        for y in 0..size {
            for x in 0..size {
                let is_color1 = ((x / tile_size) + (y / tile_size)).is_multiple_of(2);
                let color = if is_color1 { color1 } else { color2 };

                data.push((color.r * 255.0) as u8);
                data.push((color.g * 255.0) as u8);
                data.push((color.b * 255.0) as u8);
                data.push((color.a * 255.0) as u8);
            }
        }

        Self {
            width: size,
            height: size,
            data,
        }
    }

    /// Create gradient texture.
    pub fn gradient_vertical(width: u32, height: u32, top: Color, bottom: Color) -> Self {
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            let t = y as f32 / (height - 1) as f32;
            let color = top.lerp(bottom, t);

            for _ in 0..width {
                data.push((color.r * 255.0) as u8);
                data.push((color.g * 255.0) as u8);
                data.push((color.b * 255.0) as u8);
                data.push((color.a * 255.0) as u8);
            }
        }

        Self {
            width,
            height,
            data,
        }
    }

    /// Create noise texture.
    pub fn noise(width: u32, height: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for _ in 0..(width * height) {
            let v = rng.gen_range(0..=255);
            data.push(v);
            data.push(v);
            data.push(v);
            data.push(255);
        }

        Self {
            width,
            height,
            data,
        }
    }

    /// Get pixel at coordinates.
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.data.len() {
            Color::rgba(
                self.data[idx],
                self.data[idx + 1],
                self.data[idx + 2],
                self.data[idx + 3],
            )
        } else {
            Color::BLACK
        }
    }

    /// Set pixel at coordinates.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.data.len() {
            self.data[idx] = (color.r * 255.0) as u8;
            self.data[idx + 1] = (color.g * 255.0) as u8;
            self.data[idx + 2] = (color.b * 255.0) as u8;
            self.data[idx + 3] = (color.a * 255.0) as u8;
        }
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::new(1, 1)
    }
}
