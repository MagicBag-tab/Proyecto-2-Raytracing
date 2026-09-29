use crate::core::ray_intersect::Material;
use crate::materials::color::Color;
use crate::materials::texture::{ProceduralTexture, Texture};
use std::sync::Arc;

pub enum Preset {
    Wood,
    Stone,
    Metal,
    Glass,
    Paper,
}

pub fn create(preset: Preset) -> Material {
    match preset {
        Preset::Wood => wood(),
        Preset::Stone => stone(),
        Preset::Metal => metal(),
        Preset::Glass => glass(),
        Preset::Paper => paper(),
    }
}

fn wood() -> Material {
    Material::new(Color::new(150, 86, 42))
        .with_albedo(0.82)
        .with_specular(24.0, 0.12)
        .with_texture(Arc::new(Texture::procedural(ProceduralTexture::Wood)))
}

fn stone() -> Material {
    Material::new(Color::new(125, 132, 139))
        .with_albedo(0.9)
        .with_specular(18.0, 0.08)
        .with_texture(Arc::new(Texture::procedural(ProceduralTexture::Stone)))
}

fn metal() -> Material {
    Material::new(Color::new(170, 180, 190))
        .with_albedo(0.55)
        .with_specular(96.0, 0.8)
        .with_reflectivity(0.35)
        .with_texture(Arc::new(Texture::procedural(ProceduralTexture::Metal)))
}

fn glass() -> Material {
    Material::new(Color::new(185, 222, 232))
        .with_albedo(0.18)
        .with_specular(128.0, 0.9)
        .with_transparency(0.94, 1.5)
        .with_reflectivity(0.04)
        .with_texture(Arc::new(Texture::procedural(ProceduralTexture::Glass)))
}

fn paper() -> Material {
    Material::new(Color::new(231, 221, 194))
        .with_albedo(0.96)
        .with_specular(8.0, 0.025)
        .with_texture(Arc::new(Texture::procedural(ProceduralTexture::Paper)))
}
