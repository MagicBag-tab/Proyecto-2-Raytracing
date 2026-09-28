use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::transform::Transform;
use nalgebra_glm::{inverse, transpose, vec4, vec4_to_vec3, Vec3};

pub struct Plane {
    pub transform: Transform,
    pub material: Material,
}

impl RayIntersect for Plane {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let model_matrix = self.transform.matrix();
        let inverse_matrix = inverse(&model_matrix);

        let local_origin = vec4_to_vec3(&(inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)));
        let local_direction = vec4_to_vec3(&(inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)));
        let local_dir_norm = local_direction.normalize();

        if local_dir_norm.y.abs() < 1e-4 {
            return None;
        }

        let t = -local_origin.y / local_dir_norm.y;

        if t <= 1e-4 {
            return None;
        }

        let x = local_origin.x + t * local_dir_norm.x;
        let z = local_origin.z + t * local_dir_norm.z;

        if x < -0.5 || x > 0.5 || z < -0.5 || z > 0.5 {
            return None;
        }

        let local_point = Vec3::new(x, 0.0, z);
        let local_normal = if local_dir_norm.y > 0.0 {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        let world_point = vec4_to_vec3(&(model_matrix * vec4(local_point.x, local_point.y, local_point.z, 1.0)));
        let normal_matrix = transpose(&inverse_matrix);
        let world_normal = vec4_to_vec3(&(normal_matrix * vec4(local_normal.x, local_normal.y, local_normal.z, 0.0))).normalize();
        let distance = nalgebra_glm::magnitude(&(world_point - ray_origin));

        let u = x + 0.5;
        let v = z + 0.5;

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
