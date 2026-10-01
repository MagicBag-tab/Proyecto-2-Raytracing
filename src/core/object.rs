use crate::core::ray_intersect::{Intersect, Material, RayIntersect};
use crate::core::transform::Transform;
use nalgebra_glm::{inverse, transpose, vec4, vec4_to_vec3, Mat4, Vec3};

pub trait Shape: Send + Sync {
    fn local_intersect(
        &self,
        local_origin: &Vec3,
        local_dir_norm: &Vec3,
    ) -> Option<(f32, Vec3, f32, f32)>;
}

pub struct Object {
    pub shape: Box<dyn Shape>,
    pub material: Material,
    transform: Transform,
    model_matrix: Mat4,
    inverse_matrix: Mat4,
    normal_matrix: Mat4,
}

impl Object {
    pub fn new(shape: Box<dyn Shape>, transform: Transform, material: Material) -> Self {
        let model_matrix = transform.matrix();
        let inverse_matrix = inverse(&model_matrix);
        let normal_matrix = transpose(&inverse_matrix);
        Self {
            shape,
            material,
            transform,
            model_matrix,
            inverse_matrix,
            normal_matrix,
        }
    }
}

impl RayIntersect for Object {
    fn translate_by(&mut self, delta: &Vec3) -> bool {
        self.transform.position += delta;
        self.model_matrix = self.transform.matrix();
        self.inverse_matrix = inverse(&self.model_matrix);
        self.normal_matrix = transpose(&self.inverse_matrix);
        true
    }

    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let local_origin = vec4_to_vec3(
            &(self.inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)),
        );
        let local_direction = vec4_to_vec3(
            &(self.inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)),
        );
        let local_dir_norm = local_direction.normalize();

        if let Some((t, local_normal, u, v)) =
            self.shape.local_intersect(&local_origin, &local_dir_norm)
        {
            let local_point = local_origin + local_dir_norm * t;
            let world_point = vec4_to_vec3(
                &(self.model_matrix * vec4(local_point.x, local_point.y, local_point.z, 1.0)),
            );
            let world_normal = vec4_to_vec3(
                &(self.normal_matrix * vec4(local_normal.x, local_normal.y, local_normal.z, 0.0)),
            )
            .normalize();
            let direction_length = ray_direction.magnitude();
            if direction_length <= f32::EPSILON || !direction_length.is_finite() {
                return None;
            }
            let distance = nalgebra_glm::dot(
                &(world_point - ray_origin),
                &(ray_direction / direction_length),
            );
            if distance <= 0.0 || !distance.is_finite() {
                return None;
            }

            Some(Intersect {
                object_index: usize::MAX,
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

    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        self.ray_intersect(ray_origin, ray_direction)
            .map(|intersect| intersect.distance)
    }
}
