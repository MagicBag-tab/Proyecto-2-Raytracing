use nalgebra_glm::{Mat4, Vec3, identity, rotate_x, rotate_y, rotate_z, scale, translate};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Transform {
            position,
            rotation,
            scale,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        let mut m = identity();
        m = translate(&m, &self.position);
        m = rotate_x(&m, self.rotation.x);
        m = rotate_y(&m, self.rotation.y);
        m = rotate_z(&m, self.rotation.z);
        m = scale(&m, &self.scale);
        m
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}
