use nalgebra_glm::Vec3;
use std::f32::consts::PI;

const PITCH_LIMIT: f32 = PI / 2.0 - 0.1;
const MIN_ORBIT_RADIUS: f32 = 4.5;
const MAX_ORBIT_RADIUS: f32 = 32.0;

#[derive(Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Camera { eye, center, up }
    }

    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(&self.up).normalize();

        let up = right.cross(&forward).normalize();

        let rotated = vector.x * right + vector.y * up - vector.z * forward;

        rotated.normalize()
    }

    pub fn ray_for_pixel(&self, x: f32, y: f32, width: usize, height: usize, fov: f32) -> Vec3 {
        let width_f = width as f32;
        let height_f = height as f32;
        let aspect_ratio = width_f / height_f;
        let perspective_scale = (fov / 2.0).tan();
        let screen_x = x * (2.0 * aspect_ratio * perspective_scale / width_f)
            - aspect_ratio * perspective_scale;
        let screen_y = y * (-2.0 * perspective_scale / height_f) + perspective_scale;
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(&self.up).normalize();
        let camera_up = right.cross(&forward).normalize();
        let camera_direction = Vec3::new(screen_x, screen_y, -1.0);

        (camera_direction.x * right + camera_direction.y * camera_up - camera_direction.z * forward)
            .normalize()
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let radius_vector = self.eye - self.center;
        let radius = radius_vector.magnitude();

        let current_yaw = radius_vector.z.atan2(radius_vector.x);
        let radius_xz =
            (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
        let current_pitch = (-radius_vector.y).atan2(radius_xz);

        let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        let new_pitch = (current_pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        self.eye = self.center
            + Vec3::new(
                radius * new_yaw.cos() * new_pitch.cos(),
                -radius * new_pitch.sin(),
                radius * new_yaw.sin() * new_pitch.cos(),
            );
    }

    pub fn zoom(&mut self, scroll_delta: f32) {
        let offset = self.eye - self.center;
        let radius = offset.magnitude();
        if radius <= f32::EPSILON || !scroll_delta.is_finite() {
            return;
        }

        let new_radius =
            (radius * (-scroll_delta * 0.12).exp()).clamp(MIN_ORBIT_RADIUS, MAX_ORBIT_RADIUS);
        self.eye = self.center + offset * (new_radius / radius);
    }
}

#[cfg(test)]
mod tests {
    use super::Camera;
    use nalgebra_glm::Vec3;
    use std::f32::consts::PI;

    #[test]
    fn center_pixel_ray_follows_camera_forward() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::zeros(),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let ray = camera.ray_for_pixel(400.0, 300.0, 800, 600, PI / 3.0);
        let forward = (camera.center - camera.eye).normalize();

        assert!(nalgebra_glm::dot(&ray, &forward) > 0.99999);
    }
}
