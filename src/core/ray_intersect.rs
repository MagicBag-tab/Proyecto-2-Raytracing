use crate::materials::color::Color;
use nalgebra_glm::Vec3;

use crate::materials::texture::Texture;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub emission: Color,
    pub emission_strength: f32,
    pub albedo: f32,
    pub specular: f32,
    pub specular_strength: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub refractive_index: f32,
    pub texture: Option<Arc<Texture>>,
    pub normal_map: Option<Arc<Texture>>,
    pub specular_map: Option<Arc<Texture>>,
    pub overlay_texture: Option<Arc<Texture>>,
    pub overlay_normal_map: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(diffuse: Color) -> Self {
        Material {
            diffuse,
            emission: Color::new(0, 0, 0),
            emission_strength: 0.0,
            albedo: 1.0,
            specular: 32.0,
            specular_strength: 0.04,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            texture: None,
            normal_map: None,
            specular_map: None,
            overlay_texture: None,
            overlay_normal_map: None,
        }
    }

    pub fn with_emission(mut self, color: Color, strength: f32) -> Self {
        self.emission = color;
        self.emission_strength = strength.max(0.0);
        self
    }

    pub fn with_albedo(mut self, albedo: f32) -> Self {
        self.albedo = albedo.clamp(0.0, 1.0);
        self
    }

    pub fn with_specular(mut self, exponent: f32, strength: f32) -> Self {
        self.specular = exponent.max(1.0);
        self.specular_strength = strength.clamp(0.0, 1.0);
        self
    }

    pub fn with_transparency(mut self, transparency: f32, refractive_index: f32) -> Self {
        self.transparency = transparency.clamp(0.0, 1.0);
        self.refractive_index = refractive_index.max(1.0);
        self
    }

    pub fn with_reflectivity(mut self, reflectivity: f32) -> Self {
        self.reflectivity = reflectivity.clamp(0.0, 1.0);
        self
    }

    pub fn with_texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_normal_map(mut self, normal_map: Arc<Texture>) -> Self {
        self.normal_map = Some(normal_map);
        self
    }

    pub fn with_specular_map(mut self, specular_map: Arc<Texture>) -> Self {
        self.specular_map = Some(specular_map);
        self
    }

    pub fn with_overlay(mut self, texture: Arc<Texture>, normal_map: Arc<Texture>) -> Self {
        self.overlay_texture = Some(texture);
        self.overlay_normal_map = Some(normal_map);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub material: Material,
    pub u: f32,
    pub v: f32,
}

pub trait RayIntersect: Send + Sync {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;

    fn translate_by(&mut self, _delta: &Vec3) -> bool {
        false
    }

    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        self.ray_intersect(ray_origin, ray_direction)
            .map(|intersect| intersect.distance)
    }
}
