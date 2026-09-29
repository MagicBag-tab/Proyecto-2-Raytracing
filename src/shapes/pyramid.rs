use crate::core::object::Shape;
use nalgebra_glm::{cross, dot, Vec3};

pub struct Pyramid;

impl Shape for Pyramid {
    fn local_intersect(
        &self,
        local_origin: &Vec3,
        local_dir_norm: &Vec3,
    ) -> Option<(f32, Vec3, f32, f32)> {
        let v0 = Vec3::new(-0.5, -0.5, -0.5);
        let v1 = Vec3::new(0.5, -0.5, -0.5);
        let v2 = Vec3::new(0.5, -0.5, 0.5);
        let v3 = Vec3::new(-0.5, -0.5, 0.5);
        let apex = Vec3::new(0.0, 0.5, 0.0);

        let triangles = [
            (v0, v1, v2),
            (v0, v2, v3),
            (v3, v2, apex),
            (v2, v1, apex),
            (v1, v0, apex),
            (v0, v3, apex),
        ];

        let mut best_t = f32::INFINITY;
        let mut best_normal = Vec3::zeros();
        let mut hit_u = 0.0;
        let mut hit_v = 0.0;

        for (p0, p1, p2) in triangles {
            let edge1 = p1 - p0;
            let edge2 = p2 - p0;
            let h = cross(local_dir_norm, &edge2);
            let a = dot(&edge1, &h);
            if a > -1e-4 && a < 1e-4 {
                continue;
            }

            let f = 1.0 / a;
            let s = local_origin - p0;
            let u = f * dot(&s, &h);
            if !(0.0..=1.0).contains(&u) {
                continue;
            }

            let q = cross(&s, &edge1);
            let v = f * dot(local_dir_norm, &q);
            if v < 0.0 || u + v > 1.0 {
                continue;
            }

            let t = f * dot(&edge2, &q);
            if t > 1e-4 && t < best_t {
                best_t = t;
                best_normal = cross(&edge1, &edge2).normalize();
                hit_u = u;
                hit_v = v;
            }
        }

        if best_t == f32::INFINITY {
            None
        } else {
            Some((best_t, best_normal, hit_u, hit_v))
        }
    }
}
