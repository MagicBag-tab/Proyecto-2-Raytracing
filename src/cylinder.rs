use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::transform::Transform;
use nalgebra_glm::{inverse, transpose, vec4, vec4_to_vec3, Vec3};

const EPSILON: f32 = 1e-4;

pub struct Cylinder {
    pub transform: Transform,
    pub material: Material,
}

fn closest(current: Option<(f32, Vec3)>, t: f32, normal: Vec3) -> Option<(f32, Vec3)> {
    if t > EPSILON && current.is_none_or(|(best, _)| t < best) {
        Some((t, normal))
    } else {
        current
    }
}

impl RayIntersect for Cylinder {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let model_matrix = self.transform.matrix();
        let inverse_matrix = inverse(&model_matrix);

        let local_origin = vec4_to_vec3(&(inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)));
        let local_direction = vec4_to_vec3(&(inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)));
        let local_dir_norm = local_direction.normalize();

        let oc = local_origin;
        let mut hit: Option<(f32, Vec3)> = None;

        let a = local_dir_norm.x * local_dir_norm.x + local_dir_norm.z * local_dir_norm.z;

        if a > EPSILON {
            let b = 2.0 * (oc.x * local_dir_norm.x + oc.z * local_dir_norm.z);
            let c = (oc.x * oc.x + oc.z * oc.z) - 1.0;

            let discriminant = b * b - 4.0 * a * c;

            if discriminant > 0.0 {
                let root = discriminant.sqrt();

                for t in [(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)] {
                    let y = oc.y + t * local_dir_norm.y;

                    if (-0.5..=0.5).contains(&y) {
                        let normal = Vec3::new(oc.x + t * local_dir_norm.x, 0.0, oc.z + t * local_dir_norm.z).normalize();
                        hit = closest(hit, t, normal);
                    }
                }
            }
        }

        if local_dir_norm.y.abs() > EPSILON {
            for (y, normal) in [(-0.5, Vec3::new(0.0, -1.0, 0.0)), (0.5, Vec3::new(0.0, 1.0, 0.0))] {
                let t = (y - oc.y) / local_dir_norm.y;
                let x = oc.x + t * local_dir_norm.x;
                let z = oc.z + t * local_dir_norm.z;

                if x * x + z * z <= 1.0 {
                    hit = closest(hit, t, normal);
                }
            }
        }

        let (t, local_normal) = hit?;

        let local_point = local_origin + local_dir_norm * t;

        let world_point = vec4_to_vec3(&(model_matrix * vec4(local_point.x, local_point.y, local_point.z, 1.0)));
        let normal_matrix = transpose(&inverse_matrix);
        let world_normal = vec4_to_vec3(&(normal_matrix * vec4(local_normal.x, local_normal.y, local_normal.z, 0.0))).normalize();
        let distance = nalgebra_glm::magnitude(&(world_point - ray_origin));

        let u = 0.5 + (local_normal.z.atan2(local_normal.x) / (2.0 * std::f32::consts::PI));
        let v = local_point.y + 0.5;

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
