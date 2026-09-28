use nalgebra_glm::{dot, Vec3};
use crate::core::object::Shape;

pub struct Sphere;

impl Shape for Sphere {
    fn local_intersect(&self, local_origin: &Vec3, local_dir_norm: &Vec3) -> Option<(f32, Vec3, f32, f32)> {
        let a = dot(local_dir_norm, local_dir_norm);
        let b = 2.0 * dot(local_origin, local_dir_norm);
        let c = dot(local_origin, local_origin) - 1.0;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant <= 0.0 { return None; }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t <= 0.0 { return None; }

        let local_point = local_origin + local_dir_norm * t;
        let local_normal = local_point.normalize();
        let u = 0.5 + (local_normal.z.atan2(local_normal.x) / (2.0 * std::f32::consts::PI));
        let v = 0.5 - (local_normal.y.asin() / std::f32::consts::PI);

        Some((t, local_normal, u, v))
    }
}
