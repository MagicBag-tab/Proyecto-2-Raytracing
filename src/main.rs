use crate::materials::texture::Texture;

mod core;
mod materials;
mod shapes;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::core::camera::Camera;
use crate::core::framebuffer::Framebuffer;
use crate::core::light::Light;
use crate::core::object::Object;
use crate::core::ray_intersect::{Intersect, Material, RayIntersect};
use crate::core::scene::{Scene, Skybox};
use crate::core::transform::Transform;
use crate::materials::color::Color;
use crate::shapes::cone::Cone;
use crate::shapes::cylinder::Cylinder;
use crate::shapes::plane::Plane;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SKY_COLOR: u32 = 0x040C24;

const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 60.0;

const SHADOW_BIAS: f32 = 1e-3;
const REFLECTION_BIAS: f32 = 1e-3;
const REFRACTION_BIAS: f32 = 1e-3;

const MAX_DEPTH: u32 = 3;

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

fn refract_direction(incident: &Vec3, normal: &Vec3, eta_ratio: f32) -> Option<Vec3> {
    let cos_theta = dot(&-*incident, normal).clamp(0.0, 1.0);
    let perpendicular = eta_ratio * (incident + cos_theta * normal);
    let discriminant = 1.0 - perpendicular.magnitude_squared();
    (discriminant >= 0.0).then(|| (perpendicular - discriminant.sqrt() * normal).normalize())
}

pub fn cast_shadow(
    intersect: &Intersect,
    light_direction: &Vec3,
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
) -> bool {
    let shadow_ray_origin = intersect.point + intersect.normal * SHADOW_BIAS;
    let light_distance = (light.position - intersect.point).magnitude();

    objects.iter().any(|object| {
        object
            .ray_intersect(&shadow_ray_origin, light_direction)
            .is_some_and(|blocker| blocker.distance < light_distance)
    })
}

pub fn shade(
    intersect: &Intersect,
    ray_origin: &Vec3,
    lights: &[Light],
    objects: &[Box<dyn RayIntersect>],
    render_mode: u8,
) -> Color {
    let view_direction = (ray_origin - intersect.point).normalize();

    let u = intersect.u;
    let v = intersect.v;
    let mut blend_factor = 0.0;
    let mut water_normal_mapped = Vec3::new(0.0, 0.0, 1.0);

    if render_mode != 1 {
        if let Some(overlay) = &intersect.material.overlay_texture {
            blend_factor = overlay.get_intensity(u, v).sqrt().clamp(0.0, 1.0);

            if blend_factor > 0.0 {
                if let Some(overlay_normal) = &intersect.material.overlay_normal_map {
                    water_normal_mapped = overlay_normal.get_normal(u, v);
                }
            }
        }
    }

    let mut diffuse_color = intersect.material.diffuse;
    if render_mode != 2 {
        if let Some(texture) = &intersect.material.texture {
            let refracted_u = u + water_normal_mapped.x * 0.03 * blend_factor;
            let refracted_v = v + water_normal_mapped.y * 0.03 * blend_factor;

            diffuse_color = texture.get_color(refracted_u, refracted_v);
        }
    }

    if blend_factor > 0.0 {
        let drop_tint = Color::new(255, 255, 255) * (0.03 * blend_factor);

        let bottom_factor = (-water_normal_mapped.y).max(0.0);
        let bottom_shadow = 1.0 - (bottom_factor * 0.4 * blend_factor); // 0.4 controla qué tan oscura es

        diffuse_color = (diffuse_color * bottom_shadow) + drop_tint;
    }

    let base_normal = intersect.normal;
    let mut tangent = Vec3::new(1.0, 0.0, 0.0);
    if base_normal.x.abs() > 0.9 {
        tangent = Vec3::new(0.0, 1.0, 0.0);
    }
    let bitangent = base_normal.cross(&tangent).normalize();
    let tangent = bitangent.cross(&base_normal).normalize();

    let mut tile_normal = Vec3::new(0.0, 0.0, 1.0);
    if render_mode != 2 {
        if let Some(normal_map) = &intersect.material.normal_map {
            tile_normal = normal_map.get_normal(u, v);
        }
    }

    let tile_world_normal =
        (tangent * tile_normal.x + bitangent * tile_normal.y + base_normal * tile_normal.z)
            .normalize();

    let face_ambient = if base_normal.y < -0.5 { 0.45 } else { 1.0 };
    let ambient_intensity = 0.35 * face_ambient;
    let ambient = diffuse_color * ambient_intensity;

    let mut specular_normal = tile_world_normal;
    let mut specular_factor = intersect.material.specular_strength;
    let mut specular_exponent = intersect.material.specular;

    if render_mode != 2 {
        if let Some(specular_map) = &intersect.material.specular_map {
            specular_factor *= specular_map.get_intensity(u, v);
        }
    }

    let mut ambient_reflection = Color::new(0, 0, 0);

    if blend_factor > 0.0 {
        let water_world_normal = (tangent * water_normal_mapped.x
            + bitangent * water_normal_mapped.y
            + base_normal * water_normal_mapped.z)
            .normalize();

        specular_normal = (tile_world_normal * (1.0 - blend_factor)
            + water_world_normal * blend_factor)
            .normalize();

        let fresnel = (1.0
            - dot(&view_direction, &water_world_normal)
                .abs()
                .clamp(0.0, 1.0))
        .powf(5.0);

        let sky_color = Color::new(100, 130, 160);
        ambient_reflection = sky_color * (fresnel * 0.6 * blend_factor);

        specular_factor =
            specular_factor * (1.0 - blend_factor) + (2.0 + 1.0 * fresnel) * blend_factor;
        specular_exponent = specular_exponent * (1.0 - blend_factor) + 150.0 * blend_factor;
    }

    let mut diffuse = Color::new(0, 0, 0);
    let mut specular = Color::new(0, 0, 0);

    for light in lights {
        let light_direction = (light.position - intersect.point).normalize();
        let light_intensity = if cast_shadow(intersect, &light_direction, light, objects) {
            0.0
        } else {
            light.intensity
        };
        let diffuse_intensity = dot(&tile_world_normal, &light_direction).max(0.0);
        diffuse = diffuse
            + diffuse_color * (diffuse_intensity * intersect.material.albedo * light_intensity);

        let reflect_direction = reflect(&-light_direction, &specular_normal);
        let specular_intensity = dot(&view_direction, &reflect_direction)
            .max(0.0)
            .powf(specular_exponent);
        specular =
            specular + light.color * (specular_intensity * specular_factor * light_intensity);
    }

    ambient + diffuse + specular + ambient_reflection
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    scene: &Scene,
    depth: u32,
    render_mode: u8,
) -> Color {
    if depth > MAX_DEPTH {
        return scene.skybox.sample(ray_direction);
    }

    let mut closest: Option<Intersect> = None;

    for object in &scene.objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest
                .as_ref()
                .is_none_or(|current| intersect.distance < current.distance)
            {
                closest = Some(intersect);
            }
        }
    }

    let Some(intersect) = closest else {
        return scene.skybox.sample(ray_direction);
    };

    let color = shade(
        &intersect,
        ray_origin,
        &scene.lights,
        &scene.objects,
        render_mode,
    );

    let reflectivity = intersect.material.reflectivity;
    let transparency = intersect.material.transparency;

    if reflectivity <= 0.0 && transparency <= 0.0 {
        return color;
    }

    let incident = normalize(ray_direction);
    let mut normal = intersect.normal;
    let entering = dot(&incident, &normal) < 0.0;
    let (eta_i, eta_t) = if entering {
        (1.0, intersect.material.refractive_index)
    } else {
        normal = -normal;
        (intersect.material.refractive_index, 1.0)
    };
    let cos_theta = dot(&-incident, &normal).clamp(0.0, 1.0);
    let eta = eta_i / eta_t;
    let refracted_direction = refract_direction(&incident, &normal, eta);

    let r0 = ((eta_i - eta_t) / (eta_i + eta_t)).powi(2);
    let fresnel = if refracted_direction.is_some() {
        r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
    } else {
        1.0
    };
    let surface_weight = (1.0 - transparency) * (1.0 - reflectivity);
    let reflection_weight = reflectivity + transparency * (1.0 - reflectivity) * fresnel;
    let transmission_weight = transparency * (1.0 - reflectivity) * (1.0 - fresnel);

    let mut result = color * surface_weight;
    if reflection_weight > 0.0 {
        let reflected_direction = reflect(&incident, &normal).normalize();
        let reflected_origin = intersect.point + normal * REFLECTION_BIAS;
        let reflected = cast_ray(
            &reflected_origin,
            &reflected_direction,
            scene,
            depth + 1,
            render_mode,
        );
        result = result + reflected * reflection_weight;
    }
    if transmission_weight > 0.0 {
        if let Some(refracted_direction) = refracted_direction {
            let refracted_origin = intersect.point - normal * REFRACTION_BIAS;
            let refracted = cast_ray(
                &refracted_origin,
                &refracted_direction,
                scene,
                depth + 1,
                render_mode,
            );
            result = result + refracted * transmission_weight;
        }
    }
    result
}

use rayon::prelude::*;

pub fn render(framebuffer: &mut Framebuffer, scene: &Scene, render_mode: u8) {
    let width_f = framebuffer.width as f32;
    let height_f = framebuffer.height as f32;
    let aspect_ratio = width_f / height_f;
    let perspective_scale = (FOV / 2.0).tan();
    let width = framebuffer.width;

    framebuffer
        .buffer
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, pixel)| {
            let x = i % width;
            let y = i / width;

            let screen_x = (2.0 * x as f32) / width_f - 1.0;
            let screen_y = -(2.0 * y as f32) / height_f + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let ray_direction = scene.camera.basis_change(&ray_direction);

            let sample_color = cast_ray(&scene.camera.eye, &ray_direction, scene, 0, render_mode);
            let hex = sample_color.to_hex();

            let r = (hex >> 16) & 0xFF;
            let g = (hex >> 8) & 0xFF;
            let b = hex & 0xFF;
            *pixel = r << 16 | g << 8 | b;
        });
}

fn create_camera() -> Camera {
    Camera::new(
        Vec3::new(0.0, 0.4, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    )
}

fn create_scene(camera: Camera) -> Scene {
    let tile_texture =
        std::sync::Arc::new(Texture::new("assets/TilesSquarePoolMixed001_COL_2K.jpg"));
    let normal_map = std::sync::Arc::new(Texture::new("assets/TilesSquarePoolMixed001_NRM_2K.jpg"));
    let specular_map =
        std::sync::Arc::new(Texture::new("assets/TilesSquarePoolMixed001_REFL_2K.jpg"));

    let water_mask = std::sync::Arc::new(Texture::new(
        "assets/WaterDropletsMixedBubbled001_ALPHAMASKED_2K.png",
    ));
    let water_normal = std::sync::Arc::new(Texture::new(
        "assets/WaterDropletsMixedBubbled001_NRM_2K.jpg",
    ));

    let base_material = Material::new(Color::new(199, 159, 224))
        .with_albedo(0.6)
        .with_specular(250.0, 1.0)
        .with_reflectivity(0.1)
        .with_texture(tile_texture.clone())
        .with_normal_map(normal_map.clone())
        .with_specular_map(specular_map.clone())
        .with_overlay(water_mask.clone(), water_normal.clone());

    let brown = crate::materials::presets::create(crate::materials::presets::Preset::Wood);
    let green = Material::new(Color::new(34, 139, 34))
        .with_albedo(0.9)
        .with_specular(10.0, 0.1);

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(Object {
            shape: Box::new(Plane),
            transform: Transform::new(
                Vec3::new(0.0, -1.5, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(10.0, 1.0, 10.0),
            ),
            material: base_material.clone(),
        }),
        // Tronco (Cilindro)
        Box::new(Object {
            shape: Box::new(Cylinder),
            transform: Transform::new(
                Vec3::new(0.0, -0.5, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.5, 2.0, 0.5),
            ),
            material: brown.clone(),
        }),
        // Hojas (Cono)
        Box::new(Object {
            shape: Box::new(Cone),
            transform: Transform::new(
                Vec3::new(0.0, 1.5, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(2.0, 2.0, 2.0),
            ),
            material: green.clone(),
        }),
    ];

    let lights = vec![Light::new(
        Vec3::new(-4.5, 4.0, 6.0),
        Color::new(255, 250, 244),
        2.0,
    )];

    Scene::new(
        objects,
        lights,
        camera,
        Skybox::new(Color::from_hex(SKY_COLOR)),
    )
}

fn main() {
    let frame_delay = Duration::from_millis(16);
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new("Lakitu", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut scene = create_scene(create_camera());

    let mut camera_moved = true;
    let mut render_mode = 0; // 0 = Azulejo con gotas, 1 = Sólo Azulejo, 2 = Color Plano con gotas

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            render_mode = 0;
            camera_moved = true;
        }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
            render_mode = 1;
            camera_moved = true;
        }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
            render_mode = 2;
            camera_moved = true;
        }

        let orbit = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];

        for (key, delta_yaw, delta_pitch) in orbit {
            if window.is_key_down(key) {
                scene.camera.orbit(delta_yaw, delta_pitch);
                camera_moved = true;
            }
        }

        if camera_moved {
            render(&mut framebuffer, &scene, render_mode);
            camera_moved = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::object::Shape;
    use crate::materials::presets::{self, Preset};
    use crate::shapes::cube::Cube;
    use crate::shapes::pyramid::Pyramid;
    use crate::shapes::sphere::Sphere;
    use crate::shapes::triangle::Triangle;
    use std::sync::Arc;

    fn test_material() -> Material {
        Material::new(Color::new(180, 180, 180)).with_specular(16.0, 0.0)
    }

    fn test_object(shape: Box<dyn Shape>, transform: Transform) -> Object {
        Object {
            shape,
            transform,
            material: test_material(),
        }
    }

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-3,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn every_shape_is_intersectable() {
        let cases: [(&str, Box<dyn Shape>, Vec3, Vec3); 7] = [
            (
                "cube",
                Box::new(Cube),
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            (
                "sphere",
                Box::new(Sphere),
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            (
                "cylinder",
                Box::new(Cylinder),
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            (
                "cone",
                Box::new(Cone),
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            (
                "plane",
                Box::new(Plane),
                Vec3::new(0.0, 2.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
            ),
            (
                "pyramid",
                Box::new(Pyramid),
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            (
                "triangle",
                Box::new(Triangle),
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
        ];

        for (name, shape, origin, direction) in cases {
            let object = test_object(shape, Transform::default());
            let hit = object.ray_intersect(&origin, &direction);
            assert!(hit.is_some(), "{name} did not intersect the test ray");
        }
    }

    #[test]
    fn object_translation_moves_its_intersection() {
        let object = test_object(
            Box::new(Sphere),
            Transform::new(
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::zeros(),
                Vec3::new(1.0, 1.0, 1.0),
            ),
        );
        let origin = Vec3::new(2.0, 0.0, 3.0);
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let hit = object.ray_intersect(&origin, &direction).unwrap();

        assert_near(hit.point.x, 2.0);
        assert_near(hit.distance, 2.0);
        assert!(object
            .ray_intersect(&Vec3::new(0.0, 0.0, 3.0), &direction)
            .is_none());
    }

    #[test]
    fn object_rotation_changes_its_intersection_normal() {
        let object = test_object(
            Box::new(Triangle),
            Transform::new(
                Vec3::zeros(),
                Vec3::new(0.0, PI / 2.0, 0.0),
                Vec3::new(1.0, 1.0, 1.0),
            ),
        );
        let hit = object
            .ray_intersect(&Vec3::new(3.0, 0.0, 0.0), &Vec3::new(-1.0, 0.0, 0.0))
            .unwrap();

        assert!(hit.normal.x > 0.99, "rotated normal was {:?}", hit.normal);
        assert_near(hit.point.x, 0.0);
    }

    #[test]
    fn object_scale_changes_its_intersection_distance() {
        let object = test_object(
            Box::new(Sphere),
            Transform::new(Vec3::zeros(), Vec3::zeros(), Vec3::new(2.0, 1.0, 1.0)),
        );
        let hit = object
            .ray_intersect(&Vec3::new(3.0, 0.0, 0.0), &Vec3::new(-1.0, 0.0, 0.0))
            .unwrap();

        assert_near(hit.point.x, 2.0);
        assert_near(hit.distance, 1.0);
    }

    #[test]
    fn objects_can_keep_distinct_color_textures() {
        let tile_texture = Arc::new(Texture::new("assets/TilesSquarePoolMixed001_COL_2K.jpg"));
        let water_texture = Arc::new(Texture::new(
            "assets/WaterDropletsMixedBubbled001_COL_2K.jpg",
        ));
        let tile = test_material().with_texture(tile_texture.clone());
        let water = test_material().with_texture(water_texture.clone());

        let tile_object = Object {
            shape: Box::new(Sphere),
            transform: Transform::default(),
            material: tile,
        };
        let water_object = Object {
            shape: Box::new(Sphere),
            transform: Transform::default(),
            material: water,
        };
        let origin = Vec3::new(0.0, 0.0, 3.0);
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let tile_hit = tile_object.ray_intersect(&origin, &direction).unwrap();
        let water_hit = water_object.ray_intersect(&origin, &direction).unwrap();
        let tile_bound = tile_hit.material.texture.as_ref().unwrap();
        let water_bound = water_hit.material.texture.as_ref().unwrap();

        assert!(Arc::ptr_eq(tile_bound, &tile_texture));
        assert!(Arc::ptr_eq(water_bound, &water_texture));
        let samples_differ = [0.1, 0.3, 0.5, 0.7, 0.9].into_iter().any(|u| {
            [0.1, 0.3, 0.5, 0.7, 0.9].into_iter().any(|v| {
                tile_bound.get_color(u, v).to_hex() != water_bound.get_color(u, v).to_hex()
            })
        });
        assert!(
            samples_differ,
            "the two bound color textures sampled identically"
        );
    }

    #[test]
    fn five_material_presets_have_distinct_textures_and_properties() {
        let materials = [
            presets::create(Preset::Wood),
            presets::create(Preset::Stone),
            presets::create(Preset::Metal),
            presets::create(Preset::Glass),
            presets::create(Preset::Paper),
        ];

        for material in &materials {
            assert!(material.texture.is_some());
            assert!((0.0..=1.0).contains(&material.albedo));
            assert!((0.0..=1.0).contains(&material.specular_strength));
            assert!((0.0..=1.0).contains(&material.transparency));
            assert!((0.0..=1.0).contains(&material.reflectivity));
            assert!(material.refractive_index >= 1.0);
        }

        let textures: Vec<_> = materials
            .iter()
            .map(|material| material.texture.as_ref().unwrap())
            .collect();
        for (index, texture) in textures.iter().enumerate() {
            assert!(textures[index + 1..]
                .iter()
                .all(|other| !Arc::ptr_eq(texture, other)));
        }

        let glass = &materials[3];
        assert!(glass.transparency > 0.9);
        assert!(glass.reflectivity > 0.0);
        assert!(glass.refractive_index > 1.0);
        assert_eq!(materials[0].transparency, 0.0);
        assert_eq!(materials[1].transparency, 0.0);
        assert_eq!(materials[2].reflectivity, 0.35);
        assert_eq!(materials[4].transparency, 0.0);
    }

    #[test]
    fn refraction_obeys_snell_and_detects_total_internal_reflection() {
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let incident = Vec3::new(0.5, 0.0, -(3.0_f32).sqrt() / 2.0);
        let refracted = refract_direction(&incident, &normal, 1.0 / 1.5).unwrap();
        assert_near(refracted.x, 1.0 / 3.0);
        assert_near(refracted.z, -(8.0_f32 / 9.0).sqrt());

        let inside_incident = Vec3::new(0.9, 0.0, -((1.0_f32 - 0.9_f32.powi(2)).sqrt()));
        assert!(refract_direction(&inside_incident, &normal, 1.5).is_none());
    }

    #[test]
    fn current_scene_renders_geometry() {
        let scene = create_scene(create_camera());
        let mut framebuffer = Framebuffer::new(80, 60);

        render(&mut framebuffer, &scene, 0);

        assert!(framebuffer.buffer.iter().any(|pixel| *pixel != SKY_COLOR));
    }
}
