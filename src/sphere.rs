use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::transform::Transform;
use nalgebra_glm::{dot, inverse, transpose, vec4, vec4_to_vec3, Vec3};

pub struct Sphere {
    pub transform: Transform,
    pub material: Material,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let model_matrix = self.transform.matrix();
        let inverse_matrix = inverse(&model_matrix);

        let local_origin = vec4_to_vec3(&(inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)));
        let local_direction = vec4_to_vec3(&(inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)));
        let local_dir_norm = local_direction.normalize();

        let a = dot(&local_dir_norm, &local_dir_norm);
        let b = 2.0 * dot(&local_origin, &local_dir_norm);
        let c = dot(&local_origin, &local_origin) - 1.0;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant <= 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);

        if t <= 0.0 {
            return None;
        }

        let local_point = local_origin + local_dir_norm * t;
        let local_normal = local_point.normalize();

        let world_point = vec4_to_vec3(&(model_matrix * vec4(local_point.x, local_point.y, local_point.z, 1.0)));
        let normal_matrix = transpose(&inverse_matrix);
        let world_normal = vec4_to_vec3(&(normal_matrix * vec4(local_normal.x, local_normal.y, local_normal.z, 0.0))).normalize();
        let distance = nalgebra_glm::magnitude(&(world_point - ray_origin));

        let u = 0.5 + (local_normal.z.atan2(local_normal.x) / (2.0 * std::f32::consts::PI));
        let v = 0.5 - (local_normal.y.asin() / std::f32::consts::PI);

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
