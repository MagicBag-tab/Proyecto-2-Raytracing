use image::{DynamicImage, GenericImageView};
use crate::color::Color;

pub struct Texture {
    image: DynamicImage,
    width: u32,
    height: u32,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Texture {
    pub fn new(file_path: &str) -> Self {
        let img = image::open(file_path).expect(&format!("Failed to load texture: {}", file_path));
        let (width, height) = img.dimensions();
        Texture {
            image: img,
            width,
            height,
        }
    }

    pub fn get_color(&self, u: f32, v: f32) -> Color {
        let rgba = self.get_pixel(u, v);
        Color::new(rgba[0], rgba[1], rgba[2])
    }

    pub fn get_normal(&self, u: f32, v: f32) -> nalgebra_glm::Vec3 {
        let rgba = self.get_pixel(u, v);
        // Convert from [0, 255] to [-1.0, 1.0]
        let nx = (rgba[0] as f32 / 255.0) * 2.0 - 1.0;
        let ny = (rgba[1] as f32 / 255.0) * 2.0 - 1.0;
        let nz = (rgba[2] as f32 / 255.0) * 2.0 - 1.0;
        nalgebra_glm::normalize(&nalgebra_glm::Vec3::new(nx, ny, nz))
    }

    pub fn get_intensity(&self, u: f32, v: f32) -> f32 {
        let rgba = self.get_pixel(u, v);
        rgba[0] as f32 / 255.0 // Just use the red channel
    }

    pub fn get_alpha(&self, u: f32, v: f32) -> f32 {
        let rgba = self.get_pixel(u, v);
        rgba[3] as f32 / 255.0 // Use the alpha channel
    }

    fn get_pixel(&self, u: f32, v: f32) -> image::Rgba<u8> {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let x = (u * (self.width - 1) as f32).round() as u32;
        let y = ((1.0 - v) * (self.height - 1) as f32).round() as u32;
        self.image.get_pixel(x, y)
    }
}
