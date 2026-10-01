use crate::core::object::Shape;
use nalgebra_glm::Vec3;

pub struct Cube;

impl Shape for Cube {
    fn local_intersect(
        &self,
        local_origin: &Vec3,
        local_dir_norm: &Vec3,
    ) -> Option<(f32, Vec3, f32, f32)> {
        let bounds_min = [-0.5_f32; 3];
        let bounds_max = [0.5_f32; 3];
        let origins = [local_origin.x, local_origin.y, local_origin.z];
        let directions = [local_dir_norm.x, local_dir_norm.y, local_dir_norm.z];
        let mut enter_t = f32::NEG_INFINITY;
        let mut exit_t = f32::INFINITY;
        let mut enter_normal = Vec3::zeros();
        let mut exit_normal = Vec3::zeros();

        for axis in 0..3 {
            let origin = origins[axis];
            let direction = directions[axis];
            if direction.abs() <= 1e-8 {
                if origin < bounds_min[axis] || origin > bounds_max[axis] {
                    return None;
                }
                continue;
            }

            let axis_normal = match axis {
                0 => Vec3::new(1.0, 0.0, 0.0),
                1 => Vec3::new(0.0, 1.0, 0.0),
                _ => Vec3::new(0.0, 0.0, 1.0),
            };
            let mut near_t = (bounds_min[axis] - origin) / direction;
            let mut far_t = (bounds_max[axis] - origin) / direction;
            let mut near_normal = -axis_normal;
            let mut far_normal = axis_normal;
            if near_t > far_t {
                std::mem::swap(&mut near_t, &mut far_t);
                std::mem::swap(&mut near_normal, &mut far_normal);
            }

            if near_t > enter_t {
                enter_t = near_t;
                enter_normal = near_normal;
            }
            if far_t < exit_t {
                exit_t = far_t;
                exit_normal = far_normal;
            }
            if enter_t > exit_t {
                return None;
            }
        }

        let (hit_t, local_normal) = if enter_t > 1e-4 {
            (enter_t, enter_normal)
        } else if exit_t > 1e-4 {
            (exit_t, exit_normal)
        } else {
            return None;
        };

        let local_point = local_origin + local_dir_norm * hit_t;
        let (u, v) = if local_normal.x.abs() > 0.5 {
            (local_point.z + 0.5, local_point.y + 0.5)
        } else if local_normal.y.abs() > 0.5 {
            (local_point.x + 0.5, local_point.z + 0.5)
        } else {
            (local_point.x + 0.5, local_point.y + 0.5)
        };

        Some((hit_t, local_normal, u, v))
    }
}

#[cfg(test)]
mod tests {
    use super::Cube;
    use crate::core::object::Shape;
    use nalgebra_glm::Vec3;

    #[test]
    fn parallel_axis_ray_hits_without_nan_slabs() {
        let hit = Cube
            .local_intersect(&Vec3::new(0.0, 0.0, 2.0), &Vec3::new(0.0, 0.0, -1.0))
            .unwrap();
        assert!((hit.0 - 1.5).abs() < 1e-6);
        let point = Vec3::new(0.0, 0.0, 2.0) + Vec3::new(0.0, 0.0, -1.0) * hit.0;
        assert!((point.z - 0.5).abs() < 1e-6);
    }

    #[test]
    fn ray_pointing_away_does_not_hit_behind_origin() {
        assert!(Cube
            .local_intersect(&Vec3::new(2.0, 0.0, 0.0), &Vec3::new(1.0, 0.0, 0.0))
            .is_none());
    }

    #[test]
    fn ray_starting_inside_uses_positive_exit_root() {
        let hit = Cube
            .local_intersect(&Vec3::zeros(), &Vec3::new(0.0, 1.0, 0.0))
            .unwrap();
        assert!((hit.0 - 0.5).abs() < 1e-6);
        assert!(hit.1.y > 0.0);
    }
}
