use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::transform::Transform;
use nalgebra_glm::{inverse, transpose, vec4, vec4_to_vec3, Vec3};

pub struct Cube {
    pub transform: Transform,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let model_matrix = self.transform.matrix();
        let inverse_matrix = inverse(&model_matrix);

        // Convert rays to object space
        let local_origin = vec4_to_vec3(&(inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)));
        let local_direction = vec4_to_vec3(&(inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)));
        
        let local_dir_norm = local_direction.normalize();

        // Unit cube bounds [-0.5, 0.5]
        let min_bound = Vec3::new(-0.5, -0.5, -0.5);
        let max_bound = Vec3::new(0.5, 0.5, 0.5);

        let mut t_min = (min_bound.x - local_origin.x) / local_dir_norm.x;
        let mut t_max = (max_bound.x - local_origin.x) / local_dir_norm.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut ty_min = (min_bound.y - local_origin.y) / local_dir_norm.y;
        let mut ty_max = (max_bound.y - local_origin.y) / local_dir_norm.y;

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

        let mut tz_min = (min_bound.z - local_origin.z) / local_dir_norm.z;
        let mut tz_max = (max_bound.z - local_origin.z) / local_dir_norm.z;

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

        let local_point = local_origin + local_dir_norm * t_min;

        // Normal computation in local space
        let p_abs = Vec3::new(local_point.x.abs(), local_point.y.abs(), local_point.z.abs());
        let mut local_normal = Vec3::new(0.0, 0.0, 0.0);
        if p_abs.x > p_abs.y && p_abs.x > p_abs.z {
            local_normal.x = local_point.x.signum();
        } else if p_abs.y > p_abs.x && p_abs.y > p_abs.z {
            local_normal.y = local_point.y.signum();
        } else {
            local_normal.z = local_point.z.signum();
        }

        // Texture coordinates (uv mappings based on face)
        let u;
        let v;
        if local_normal.x.abs() > 0.0 {
            u = local_point.z + 0.5;
            v = local_point.y + 0.5;
        } else if local_normal.y.abs() > 0.0 {
            u = local_point.x + 0.5;
            v = local_point.z + 0.5;
        } else {
            u = local_point.x + 0.5;
            v = local_point.y + 0.5;
        }

        // Convert back to world space
        let world_point = vec4_to_vec3(&(model_matrix * vec4(local_point.x, local_point.y, local_point.z, 1.0)));
        let normal_matrix = transpose(&inverse_matrix);
        let world_normal = vec4_to_vec3(&(normal_matrix * vec4(local_normal.x, local_normal.y, local_normal.z, 0.0))).normalize();
        let distance = nalgebra_glm::magnitude(&(world_point - ray_origin));

        Some(Intersect {
            point: world_point,
            normal: world_normal,
            distance,
            material: self.material.clone(),
            u,
            v,
        })
    }
}
