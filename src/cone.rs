use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::transform::Transform;
use nalgebra_glm::{inverse, transpose, vec4, vec4_to_vec3, Vec3};

pub struct Cone {
    pub transform: Transform,
    pub material: Material,
}

impl RayIntersect for Cone {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let model_matrix = self.transform.matrix();
        let inverse_matrix = inverse(&model_matrix);

        let local_origin = vec4_to_vec3(&(inverse_matrix * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.0)));
        let local_direction = vec4_to_vec3(&(inverse_matrix * vec4(ray_direction.x, ray_direction.y, ray_direction.z, 0.0)));
        let local_dir_norm = local_direction.normalize();

        let o = local_origin;
        let d = local_dir_norm;

        let a = d.x * d.x + d.z * d.z - d.y * d.y;
        let b = 2.0 * (o.x * d.x + o.z * d.z - o.y * d.y) + d.y;
        let c = o.x * o.x + o.z * o.z - o.y * o.y + o.y - 0.25;

        let mut hit: Option<(f32, Vec3)> = None;

        if a.abs() > 1e-4 {
            let discriminant = b * b - 4.0 * a * c;

            if discriminant >= 0.0 {
                let root = discriminant.sqrt();
                for t in [(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)] {
                    if t > 1e-4 {
                        let y = o.y + t * d.y;
                        if (-0.5..=0.5).contains(&y) {
                            let x = o.x + t * d.x;
                            let z = o.z + t * d.z;
                            let normal = Vec3::new(x, 0.5 - y, z).normalize();
                            
                            if hit.is_none() || t < hit.unwrap().0 {
                                hit = Some((t, normal));
                            }
                        }
                    }
                }
            }
        }

        if d.y.abs() > 1e-4 {
            let t_base = (-0.5 - o.y) / d.y;
            if t_base > 1e-4 {
                let x = o.x + t_base * d.x;
                let z = o.z + t_base * d.z;
                if x * x + z * z <= 1.0 {
                    if hit.is_none() || t_base < hit.unwrap().0 {
                        hit = Some((t_base, Vec3::new(0.0, -1.0, 0.0)));
                    }
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
