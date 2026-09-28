use nalgebra_glm::Vec3;
use crate::object::Shape;

pub struct Plane;

impl Shape for Plane {
    fn local_intersect(&self, local_origin: &Vec3, local_dir_norm: &Vec3) -> Option<(f32, Vec3, f32, f32)> {
        if local_dir_norm.y.abs() < 1e-4 { return None; }
        let t = -local_origin.y / local_dir_norm.y;
        if t <= 1e-4 { return None; }

        let x = local_origin.x + t * local_dir_norm.x;
        let z = local_origin.z + t * local_dir_norm.z;
        if x < -0.5 || x > 0.5 || z < -0.5 || z > 0.5 { return None; }

        let local_normal = if local_dir_norm.y > 0.0 {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        Some((t, local_normal, x + 0.5, z + 0.5))
    }
}
