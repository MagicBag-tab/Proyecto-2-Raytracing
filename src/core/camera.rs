use nalgebra_glm::Vec3;
use std::f32::consts::PI;

const PITCH_LIMIT: f32 = PI / 2.0 - 0.1;
const MIN_ORBIT_RADIUS: f32 = 4.5;
const MAX_ORBIT_RADIUS: f32 = 32.0;

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
    use super::*;

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-4,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn orbit_keeps_the_diorama_center_and_camera_distance() {
        let center = Vec3::new(1.0, -0.65, 2.0);
        let mut camera = Camera::new(
            center + Vec3::new(0.0, 0.9, 13.0),
            center,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let initial_radius = (camera.eye - camera.center).magnitude();

        camera.orbit(0.7, -0.2);

        assert_eq!(camera.center, center);
        assert_near((camera.eye - camera.center).magnitude(), initial_radius);
    }

    #[test]
    fn scroll_zoom_changes_distance_but_keeps_center() {
        let center = Vec3::new(0.0, -0.65, 0.0);
        let mut camera = Camera::new(Vec3::new(0.0, 0.25, 13.0), center, Vec3::new(0.0, 1.0, 0.0));
        let initial_radius = (camera.eye - camera.center).magnitude();

        camera.zoom(1.0);

        assert_eq!(camera.center, center);
        assert!((camera.eye - camera.center).magnitude() < initial_radius);
    }

    #[test]
    fn scroll_zoom_clamps_to_safe_orbit_distances() {
        let mut camera = Camera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::zeros(),
            Vec3::new(0.0, 1.0, 0.0),
        );

        camera.zoom(100.0);
        assert_near((camera.eye - camera.center).magnitude(), MIN_ORBIT_RADIUS);
        camera.zoom(-100.0);
        assert_near((camera.eye - camera.center).magnitude(), MAX_ORBIT_RADIUS);
    }
}
