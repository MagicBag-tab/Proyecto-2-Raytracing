use crate::core::object::Shape;
use nalgebra_glm::{dot, Vec3};

pub struct Sphere;

impl Shape for Sphere {
    fn local_intersect(
        &self,
        local_origin: &Vec3,
        local_dir_norm: &Vec3,
    ) -> Option<(f32, Vec3, f32, f32)> {
        let a = dot(local_dir_norm, local_dir_norm);
        let b = 2.0 * dot(local_origin, local_dir_norm);
        let c = dot(local_origin, local_origin) - 1.0;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 || a <= f32::EPSILON {
            return None;
        }

        let root = discriminant.sqrt();
        let first_root = (-b - root) / (2.0 * a);
        let second_root = (-b + root) / (2.0 * a);
        let t = if first_root > 1e-4 {
            first_root
        } else if second_root > 1e-4 {
            second_root
        } else {
            return None;
        };

        let local_point = local_origin + local_dir_norm * t;
        let local_normal = local_point.normalize();
        let u = 0.5 + (local_normal.z.atan2(local_normal.x) / (2.0 * std::f32::consts::PI));
        let v = 0.5 - (local_normal.y.asin() / std::f32::consts::PI);

        Some((t, local_normal, u, v))
    }
}

#[cfg(test)]
mod tests {
    use super::Sphere;
    use crate::core::object::Shape;
    use nalgebra_glm::Vec3;

    #[test]
    fn ray_pointing_away_does_not_hit_behind_origin() {
        assert!(Sphere
            .local_intersect(&Vec3::new(2.0, 0.0, 0.0), &Vec3::new(1.0, 0.0, 0.0))
            .is_none());
    }

    #[test]
    fn ray_starting_inside_uses_positive_exit_root() {
        let hit = Sphere
            .local_intersect(&Vec3::zeros(), &Vec3::new(0.0, 0.0, 1.0))
            .unwrap();
        assert!((hit.0 - 1.0).abs() < 1e-6);
        let point = Vec3::zeros() + Vec3::new(0.0, 0.0, 1.0) * hit.0;
        assert!(point.z > 0.0);
        assert!((point.magnitude() - 1.0).abs() < 1e-6);
    }
}
