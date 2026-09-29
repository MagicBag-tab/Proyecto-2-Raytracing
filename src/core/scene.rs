use crate::core::camera::Camera;
use crate::core::light::Light;
use crate::core::ray_intersect::RayIntersect;
use crate::materials::color::Color;
use nalgebra_glm::Vec3;

pub struct Skybox {
    pub color: Color,
}

impl Skybox {
    pub fn new(color: Color) -> Self {
        Self { color }
    }

    pub fn sample(&self, direction: &Vec3) -> Color {
        let horizon = Color::new(34, 48, 64);
        let zenith = Color::new(106, 145, 181);
        let blend = ((direction.y + 0.15) / 0.85).clamp(0.0, 1.0);
        let base = Color::new(
            (horizon_component(horizon, zenith, blend, 0)) as u8,
            (horizon_component(horizon, zenith, blend, 1)) as u8,
            (horizon_component(horizon, zenith, blend, 2)) as u8,
        );
        let tint = self.color;
        base * 0.82 + tint * 0.18
    }
}

fn horizon_component(horizon: Color, zenith: Color, blend: f32, channel: usize) -> f32 {
    let horizon = horizon.to_hex();
    let zenith = zenith.to_hex();
    let shift = 16 - channel * 8;
    let start = ((horizon >> shift) & 0xff) as f32;
    let end = ((zenith >> shift) & 0xff) as f32;
    start + (end - start) * blend
}

pub struct Scene {
    pub objects: Vec<Box<dyn RayIntersect>>,
    pub lights: Vec<Light>,
    pub camera: Camera,
    pub skybox: Skybox,
}

impl Scene {
    pub fn new(
        objects: Vec<Box<dyn RayIntersect>>,
        lights: Vec<Light>,
        camera: Camera,
        skybox: Skybox,
    ) -> Self {
        Self {
            objects,
            lights,
            camera,
            skybox,
        }
    }
}
