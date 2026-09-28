use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::transform::Transform;
use nalgebra_glm::{cross, dot, inverse, transpose, vec4, vec4_to_vec3, Vec3};

pub struct Triangle {
    pub transform: Transform,
    pub material: Material,
}

impl RayIntersect for Triangle {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let model_matrix = self.transform.matrix();
        let inverse_matrix = inverse(&model_matrix);

        let local_origin = vec4_to_vec3(&(inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)));
        let local_direction = vec4_to_vec3(&(inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)));
        let local_dir_norm = local_direction.normalize();

        let v0 = Vec3::new(0.0, 0.5, 0.0);
        let v1 = Vec3::new(-0.5, -0.5, 0.0);
        let v2 = Vec3::new(0.5, -0.5, 0.0);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = cross(&local_dir_norm, &edge2);
        let a = dot(&edge1, &h);

        if a > -1e-4 && a < 1e-4 {
            return None;
        }

        let f = 1.0 / a;
        let s = local_origin - v0;
        let u = f * dot(&s, &h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = cross(&s, &edge1);
        let v = f * dot(&local_dir_norm, &q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * dot(&edge2, &q);

        if t <= 1e-4 {
            return None;
        }

        let local_point = local_origin + local_dir_norm * t;
        let local_normal = cross(&edge1, &edge2).normalize();

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
