use crate::core::object::Shape;
use nalgebra_glm::{cross, dot, Vec3};

pub struct Triangle;

impl Shape for Triangle {
    fn local_intersect(
        &self,
        local_origin: &Vec3,
        local_dir_norm: &Vec3,
    ) -> Option<(f32, Vec3, f32, f32)> {
        let v0 = Vec3::new(0.0, 0.5, 0.0);
        let v1 = Vec3::new(-0.5, -0.5, 0.0);
        let v2 = Vec3::new(0.5, -0.5, 0.0);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = cross(local_dir_norm, &edge2);
        let a = dot(&edge1, &h);

        if a > -1e-4 && a < 1e-4 {
            return None;
        }

        let f = 1.0 / a;
        let s = local_origin - v0;
        let u = f * dot(&s, &h);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = cross(&s, &edge1);
        let v = f * dot(local_dir_norm, &q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * dot(&edge2, &q);
        if t <= 1e-4 {
            return None;
        }

        let local_normal = cross(&edge1, &edge2).normalize();
        Some((t, local_normal, u, v))
    }
}
