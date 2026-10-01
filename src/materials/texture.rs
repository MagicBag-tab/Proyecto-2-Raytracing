use crate::materials::color::Color;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

pub struct Texture {
    image: DynamicImage,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy)]
pub enum ProceduralTexture {
    Wood,
    Stone,
    Metal,
    Glass,
    Paper,
    Checkerboard,
    Grass,
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
        Self::try_new(file_path)
            .unwrap_or_else(|error| panic!("Failed to load texture {file_path}: {error}"))
    }

    pub fn try_new(file_path: &str) -> Result<Self, image::ImageError> {
        let img = image::open(file_path)?;
        let (width, height) = img.dimensions();
        Ok(Texture {
            image: img,
            width,
            height,
        })
    }

    pub fn load_or_procedural(file_path: &str, fallback: ProceduralTexture) -> Self {
        Self::try_new(file_path).unwrap_or_else(|_| Self::procedural(fallback))
    }

    pub fn procedural(pattern: ProceduralTexture) -> Self {
        let size = 128;
        let image = ImageBuffer::from_fn(size, size, |x, y| {
            let grain = noise(x, y) / 255.0;
            let base = match pattern {
                ProceduralTexture::Wood => {
                    let bands = ((y as f32 * 0.18 + (x as f32 * 0.055).sin() * 2.5).sin() * 0.5
                        + 0.5)
                        .powf(5.0);
                    let shade = (grain * 0.24 + bands * 0.76) * 46.0;
                    [105.0 + shade, 54.0 + shade * 0.62, 25.0 + shade * 0.34]
                }
                ProceduralTexture::Stone => {
                    let veins = ((x as f32 * 0.09 + (y as f32 * 0.13).sin()).sin().abs() < 0.08)
                        as u8 as f32;
                    let shade = grain * 32.0 + veins * 18.0;
                    [92.0 + shade, 99.0 + shade, 105.0 + shade]
                }
                ProceduralTexture::Metal => {
                    let brushed = ((y as f32 * 0.7).sin() * 0.5 + 0.5) * 17.0;
                    let shade = grain * 18.0 + brushed;
                    [142.0 + shade, 151.0 + shade, 158.0 + shade]
                }
                ProceduralTexture::Glass => {
                    let streak = ((x as f32 * 0.11 + y as f32 * 0.025).sin() * 0.5 + 0.5) * 13.0;
                    [143.0 + streak, 205.0 + streak, 218.0 + streak]
                }
                ProceduralTexture::Paper => {
                    let fibers = grain * 12.0;
                    [224.0 + fibers, 215.0 + fibers, 190.0 + fibers]
                }
                ProceduralTexture::Checkerboard => {
                    if ((x / 8) + (y / 8)) % 2 == 0 {
                        [220.0, 224.0, 222.0]
                    } else {
                        [47.0, 63.0, 72.0]
                    }
                }
                ProceduralTexture::Grass => {
                    let blades = (x as f32 * 0.16 + (y as f32 * 0.11).sin()).sin() * 0.5 + 0.5;
                    let shade = grain * 0.22 + blades * 20.0;
                    [30.0 + shade * 0.28, 82.0 + shade, 39.0 + shade * 0.34]
                }
            };
            Rgba([
                base[0].clamp(0.0, 255.0) as u8,
                base[1].clamp(0.0, 255.0) as u8,
                base[2].clamp(0.0, 255.0) as u8,
                255,
            ])
        });
        let width = image.width();
        let height = image.height();
        Texture {
            image: DynamicImage::ImageRgba8(image),
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

fn noise(x: u32, y: u32) -> f32 {
    let mut value = x.wrapping_mul(0x45d9f3b) ^ y.wrapping_mul(0x119de1f3);
    value = (value ^ (value >> 16)).wrapping_mul(0x45d9f3b);
    ((value ^ (value >> 16)) & 0xff) as f32
}

#[cfg(test)]
mod tests {
    use super::{ProceduralTexture, Texture};
    use image::GenericImageView;

    #[test]
    fn wood_stone_metal_and_paper_patterns_keep_texture_detail() {
        for pattern in [
            ProceduralTexture::Wood,
            ProceduralTexture::Stone,
            ProceduralTexture::Metal,
            ProceduralTexture::Paper,
        ] {
            let texture = Texture::procedural(pattern);
            let mut minimum = 255_u8;
            let mut maximum = 0_u8;
            for y in 0..texture.height {
                for x in 0..texture.width {
                    let pixel = texture.image.get_pixel(x, y);
                    minimum = minimum.min(pixel[0].min(pixel[1]).min(pixel[2]));
                    maximum = maximum.max(pixel[0].max(pixel[1]).max(pixel[2]));
                }
            }
            assert!(maximum < 250, "procedural pattern clipped near white");
            assert!(
                maximum > minimum + 4,
                "procedural pattern lost tonal detail"
            );
        }
    }
}
