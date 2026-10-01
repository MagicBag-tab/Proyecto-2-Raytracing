use crate::core::camera::Camera;
use crate::core::interaction::ObjectId;
use crate::core::ray_intersect::Intersect;
use crate::core::scene::Scene;

pub struct PickHit {
    pub object_index: usize,
    pub intersect: Intersect,
}

pub fn pick(
    scene: &Scene,
    camera: &Camera,
    mouse_x: f32,
    mouse_y: f32,
    width: usize,
    height: usize,
    fov: f32,
) -> Option<PickHit> {
    if width == 0 || height == 0 || !mouse_x.is_finite() || !mouse_y.is_finite() {
        return None;
    }

    let ray_direction = camera.ray_for_pixel(mouse_x, mouse_y, width, height, fov);
    let mut nearest: Option<PickHit> = None;

    for (object_index, object) in scene.objects.iter().enumerate() {
        if !scene.is_object_visible(ObjectId(object_index)) {
            continue;
        }
        let Some(intersect) = object.ray_intersect(&camera.eye, &ray_direction) else {
            continue;
        };

        if nearest
            .as_ref()
            .is_none_or(|current| intersect.distance < current.intersect.distance)
        {
            nearest = Some(PickHit {
                object_index,
                intersect,
            });
        }
    }

    nearest
}

#[cfg(test)]
mod tests {
    use super::pick;
    use crate::core::camera::Camera;
    use crate::core::interaction::InteractiveKind;
    use crate::core::object::Object;
    use crate::core::ray_intersect::Material;
    use crate::core::scene::{Scene, Skybox};
    use crate::core::transform::Transform;
    use crate::materials::color::Color;
    use crate::shapes::sphere::Sphere;
    use nalgebra_glm::Vec3;
    use std::f32::consts::PI;

    #[test]
    fn picking_returns_nearest_visible_intersection() {
        let material = Material::new(Color::new(220, 180, 120));
        let near = Object::new(
            Box::new(Sphere),
            Transform::new(
                Vec3::new(0.0, 0.0, 2.0),
                Vec3::zeros(),
                Vec3::new(0.5, 0.5, 0.5),
            ),
            material.clone(),
        );
        let far = Object::new(
            Box::new(Sphere),
            Transform::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::zeros(),
                Vec3::new(0.5, 0.5, 0.5),
            ),
            material,
        );
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::zeros(),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let mut scene = Scene::new(
            vec![Box::new(near), Box::new(far)],
            Vec::new(),
            camera,
            Skybox::new(Color::new(0, 0, 0)),
        );

        assert_eq!(
            pick(&scene, &scene.camera, 400.0, 300.0, 800, 600, PI / 3.0)
                .unwrap()
                .object_index,
            0
        );
        scene.register_interactive(0, InteractiveKind::Movable, false);
        assert_eq!(
            pick(&scene, &scene.camera, 400.0, 300.0, 800, 600, PI / 3.0)
                .unwrap()
                .object_index,
            1
        );
    }
}
