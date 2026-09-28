use nalgebra_glm::Vec3;
use crate::object::Shape;

pub struct Cone;

impl Shape for Cone {
    fn local_intersect(&self, local_origin: &Vec3, local_dir_norm: &Vec3) -> Option<(f32, Vec3, f32, f32)> {
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

        hit.map(|(t, local_normal)| {
            let local_point = local_origin + local_dir_norm * t;
            let u = 0.5 + (local_normal.z.atan2(local_normal.x) / (2.0 * std::f32::consts::PI));
            let v = local_point.y + 0.5;
            (t, local_normal, u, v)
        })
    }
}
