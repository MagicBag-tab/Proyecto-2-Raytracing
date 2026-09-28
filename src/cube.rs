use nalgebra_glm::Vec3;
use crate::object::Shape;

pub struct Cube;

impl Shape for Cube {
    fn local_intersect(&self, local_origin: &Vec3, local_dir_norm: &Vec3) -> Option<(f32, Vec3, f32, f32)> {
        let min_bound = Vec3::new(-0.5, -0.5, -0.5);
        let max_bound = Vec3::new(0.5, 0.5, 0.5);

        let mut t_min = (min_bound.x - local_origin.x) / local_dir_norm.x;
        let mut t_max = (max_bound.x - local_origin.x) / local_dir_norm.x;
        if t_min > t_max { std::mem::swap(&mut t_min, &mut t_max); }

        let mut ty_min = (min_bound.y - local_origin.y) / local_dir_norm.y;
        let mut ty_max = (max_bound.y - local_origin.y) / local_dir_norm.y;
        if ty_min > ty_max { std::mem::swap(&mut ty_min, &mut ty_max); }

        if (t_min > ty_max) || (ty_min > t_max) { return None; }
        if ty_min > t_min { t_min = ty_min; }
        if ty_max < t_max { t_max = ty_max; }

        let mut tz_min = (min_bound.z - local_origin.z) / local_dir_norm.z;
        let mut tz_max = (max_bound.z - local_origin.z) / local_dir_norm.z;
        if tz_min > tz_max { std::mem::swap(&mut tz_min, &mut tz_max); }

        if (t_min > tz_max) || (tz_min > t_max) { return None; }
        if tz_min > t_min { t_min = tz_min; }
        if t_min <= 0.0 { return None; }

        let local_point = local_origin + local_dir_norm * t_min;
        let p_abs = Vec3::new(local_point.x.abs(), local_point.y.abs(), local_point.z.abs());
        let mut local_normal = Vec3::new(0.0, 0.0, 0.0);
        if p_abs.x > p_abs.y && p_abs.x > p_abs.z {
            local_normal.x = local_point.x.signum();
        } else if p_abs.y > p_abs.x && p_abs.y > p_abs.z {
            local_normal.y = local_point.y.signum();
        } else {
            local_normal.z = local_point.z.signum();
        }

        let (u, v) = if local_normal.x.abs() > 0.0 {
            (local_point.z + 0.5, local_point.y + 0.5)
        } else if local_normal.y.abs() > 0.0 {
            (local_point.x + 0.5, local_point.z + 0.5)
        } else {
            (local_point.x + 0.5, local_point.y + 0.5)
        };

        Some((t_min, local_normal, u, v))
    }
}
