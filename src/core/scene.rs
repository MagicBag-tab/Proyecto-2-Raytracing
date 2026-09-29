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

    pub fn sample(&self, _direction: &Vec3) -> Color {
        self.color
    }
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
