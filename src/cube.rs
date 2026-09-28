use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::{Vec3};

pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let half_size = self.size / 2.0;
        let min_bound = self.center - Vec3::new(half_size, half_size, half_size);
        let max_bound = self.center + Vec3::new(half_size, half_size, half_size);

        let mut t_min = (min_bound.x - ray_origin.x) / ray_direction.x;
        let mut t_max = (max_bound.x - ray_origin.x) / ray_direction.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut ty_min = (min_bound.y - ray_origin.y) / ray_direction.y;
        let mut ty_max = (max_bound.y - ray_origin.y) / ray_direction.y;

        if ty_min > ty_max {
            std::mem::swap(&mut ty_min, &mut ty_max);
        }

        if (t_min > ty_max) || (ty_min > t_max) {
            return None;
        }

        if ty_min > t_min {
            t_min = ty_min;
        }
        if ty_max < t_max {
            t_max = ty_max;
        }

        let mut tz_min = (min_bound.z - ray_origin.z) / ray_direction.z;
        let mut tz_max = (max_bound.z - ray_origin.z) / ray_direction.z;

        if tz_min > tz_max {
            std::mem::swap(&mut tz_min, &mut tz_max);
        }

        if (t_min > tz_max) || (tz_min > t_max) {
            return None;
        }

        if tz_min > t_min {
            t_min = tz_min;
        }
        if t_min <= 0.0 {
            return None;
        }

        let point = ray_origin + ray_direction * t_min;

        let p = point - self.center;
        let p_abs = Vec3::new(p.x.abs(), p.y.abs(), p.z.abs());
        
        let mut normal = Vec3::new(0.0, 0.0, 0.0);
        if p_abs.x > p_abs.y && p_abs.x > p_abs.z {
            normal.x = p.x.signum();
        } else if p_abs.y > p_abs.x && p_abs.y > p_abs.z {
            normal.y = p.y.signum();
        } else {
            normal.z = p.z.signum();
        }

        let u;
        let v;
        if normal.x.abs() > 0.5 {
            u = (p.z + half_size) / self.size;
            v = (p.y + half_size) / self.size;
        } else if normal.y.abs() > 0.5 {
            u = (p.x + half_size) / self.size;
            v = (p.z + half_size) / self.size;
        } else {
            u = (p.x + half_size) / self.size;
            v = (p.y + half_size) / self.size;
        }

        Some(Intersect {
            point, 
            normal, 
            distance: t_min,
            material: self.material.clone(),
            u,
            v,
        })
    }

}
