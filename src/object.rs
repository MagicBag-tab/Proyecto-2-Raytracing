use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::transform::Transform;
use nalgebra_glm::{inverse, transpose, vec4, vec4_to_vec3, Vec3};

pub trait Shape: Send + Sync {
    fn local_intersect(&self, local_origin: &Vec3, local_dir_norm: &Vec3) -> Option<(f32, Vec3, f32, f32)>;
}

pub struct Object {
    pub shape: Box<dyn Shape>,
    pub transform: Transform,
    pub material: Material,
}

impl RayIntersect for Object {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let model_matrix = self.transform.matrix();
        let inverse_matrix = inverse(&model_matrix);

        let local_origin = vec4_to_vec3(&(inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)));
        let local_direction = vec4_to_vec3(&(inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)));
        let local_dir_norm = local_direction.normalize();

        if let Some((t, local_normal, u, v)) = self.shape.local_intersect(&local_origin, &local_dir_norm) {
            let local_point = local_origin + local_dir_norm * t;
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
        } else {
            None
        }
    }
}
