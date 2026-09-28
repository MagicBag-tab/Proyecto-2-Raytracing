use crate::color::Color;
use nalgebra_glm::Vec3;

use std::sync::Arc;
use crate::texture::Texture;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 3],
    pub texture: Option<Arc<Texture>>,
    pub normal_map: Option<Arc<Texture>>,
    pub specular_map: Option<Arc<Texture>>,
    pub overlay_texture: Option<Arc<Texture>>,
    pub overlay_normal_map: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 3]) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            texture: None,
            normal_map: None,
            specular_map: None,
            overlay_texture: None,
            overlay_normal_map: None,
        }
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

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
}
