use crate::core::object::Shape;
use nalgebra_glm::Vec3;

pub struct Cylinder;

impl Shape for Cylinder {
    fn local_intersect(
        &self,
        local_origin: &Vec3,
        local_dir_norm: &Vec3,
    ) -> Option<(f32, Vec3, f32, f32)> {
        let mut hit: Option<(f32, Vec3)> = None;

        let a = local_dir_norm.x * local_dir_norm.x + local_dir_norm.z * local_dir_norm.z;
        if a > 1e-4 {
            let b = 2.0 * (local_origin.x * local_dir_norm.x + local_origin.z * local_dir_norm.z);
            let c = (local_origin.x * local_origin.x + local_origin.z * local_origin.z) - 1.0;
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                let root = discriminant.sqrt();
                for t in [(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)] {
                    if t <= 1e-4 {
                        continue;
                    }
                    let y = local_origin.y + t * local_dir_norm.y;
                    if (-0.5..=0.5).contains(&y) {
                        let normal = Vec3::new(
                            local_origin.x + t * local_dir_norm.x,
                            0.0,
                            local_origin.z + t * local_dir_norm.z,
                        )
                        .normalize();
                        if hit.is_none() || t < hit.unwrap().0 {
                            hit = Some((t, normal));
                        }
                    }
                }
            }
        }

        if local_dir_norm.y.abs() > 1e-4 {
            for (y, normal) in [
                (-0.5, Vec3::new(0.0, -1.0, 0.0)),
                (0.5, Vec3::new(0.0, 1.0, 0.0)),
            ] {
                let t = (y - local_origin.y) / local_dir_norm.y;
                if t <= 1e-4 {
                    continue;
                }
                let x = local_origin.x + t * local_dir_norm.x;
                let z = local_origin.z + t * local_dir_norm.z;
                if x * x + z * z <= 1.0 {
                    if hit.is_none() || t < hit.unwrap().0 {
                        hit = Some((t, normal));
                    }
                }
            }
        }

        hit.map(|(t, local_normal)| {
            let local_point = local_origin + local_dir_norm * t;
            let u = 0.5 + (local_normal.z.atan2(local_normal.x) / (2.0 * std::f32::consts::PI));
            let v = local_point.y + 0.5;
            (t, local_normal, u, v)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Cylinder;
    use crate::core::object::Shape;
    use nalgebra_glm::Vec3;

    #[test]
    fn ray_pointing_away_does_not_hit_behind_origin() {
        let hit = Cylinder.local_intersect(&Vec3::new(2.0, 0.0, 0.0), &Vec3::new(1.0, 0.0, 0.0));
        assert!(hit.is_none());
    }

    #[test]
    fn ray_starting_inside_uses_positive_exit_root() {
        let hit = Cylinder
            .local_intersect(&Vec3::zeros(), &Vec3::new(1.0, 0.0, 0.0))
            .unwrap();
        assert!((hit.0 - 1.0).abs() < 1e-6);
        let point = Vec3::zeros() + Vec3::new(1.0, 0.0, 0.0) * hit.0;
        assert!(point.x > 0.0);
        assert!((point.magnitude() - 1.0).abs() < 1e-6);
    }
}
