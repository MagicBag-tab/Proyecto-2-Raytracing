pub mod assets;
mod core;
mod materials;
mod shapes;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Duration;

use crate::assets::{
    build_bamboo_cluster, build_base_diorama, build_cafe_patio, build_japanese_cafe, build_path,
    build_rock_garden, build_sakura_tree_variant, build_toro_lantern, AssetMaterials,
};
use crate::core::camera::Camera;
use crate::core::framebuffer::Framebuffer;
use crate::core::interaction::ObjectId;
use crate::core::light::Light;
use crate::core::object::{Object, Shape};
use crate::core::ray_intersect::{Intersect, Material, RayIntersect};
use crate::core::scene::{Scene, Skybox};
use crate::core::transform::Transform;
use crate::materials::color::Color;
use crate::materials::texture::Texture;
use crate::shapes::cube::Cube;
use crate::shapes::plane::Plane;
use crate::shapes::sphere::Sphere;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 60.0;

const SHADOW_BIAS_SCALE: f32 = 2e-5;
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
    scene: &Scene,
) -> bool {
    let normal_side = if dot(&intersect.normal, light_direction) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let epsilon = SHADOW_BIAS_SCALE * intersect.point.magnitude().max(1.0);
    let shadow_ray_origin = intersect.point + intersect.normal * (epsilon * normal_side);
    let to_light = light.position - shadow_ray_origin;
    let light_distance = to_light.magnitude();
    if light_distance <= epsilon {
        return false;
    }
    let shadow_direction = to_light / light_distance;

    scene.objects.iter().enumerate().any(|(index, object)| {
        if index == intersect.object_index || !scene.is_object_visible(ObjectId(index)) {
            return false;
        }
        object
            .ray_intersect_distance(&shadow_ray_origin, &shadow_direction)
            .is_some_and(|blocker_distance| {
                blocker_distance > 0.0 && blocker_distance < light_distance
            })
    })
}

pub fn shade(intersect: &Intersect, ray_origin: &Vec3, scene: &Scene, render_mode: u8) -> Color {
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
    let ambient_intensity = scene.ambient_intensity * face_ambient;
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

    for light in &scene.lights {
        let light_direction = (light.position - intersect.point).normalize();
        let light_intensity = if cast_shadow(intersect, &light_direction, light, scene) {
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

    let emission = intersect.material.emission
        * (intersect.material.emission_strength * scene.lantern_emission);
    ambient + diffuse + specular + ambient_reflection + emission
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    scene: &Scene,
    depth: u32,
    render_mode: u8,
) -> Color {
    if depth > MAX_DEPTH {
        return scene.skybox.sample(ray_origin, ray_direction);
    }

    let mut closest: Option<Intersect> = None;

    for (index, object) in scene.objects.iter().enumerate() {
        if !scene.is_object_visible(ObjectId(index)) {
            continue;
        }
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest
                .as_ref()
                .is_none_or(|current| intersect.distance < current.distance)
            {
                let mut intersect = intersect;
                intersect.object_index = index;
                closest = Some(intersect);
            }
        }
    }

    let Some(intersect) = closest else {
        return scene.skybox.sample(ray_origin, ray_direction);
    };

    let color = shade(&intersect, ray_origin, scene, render_mode);

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
    let width = framebuffer.width;
    let camera_eye = scene.camera.eye;

    framebuffer
        .buffer
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, pixel)| {
            let x = i % width;
            let y = i / width;

            let ray_direction = scene.camera.ray_for_pixel(
                x as f32,
                y as f32,
                framebuffer.width,
                framebuffer.height,
                FOV,
            );

            let sample_color = cast_ray(&camera_eye, &ray_direction, scene, 0, render_mode);
            let hex = sample_color.to_hex();

            let r = (hex >> 16) & 0xFF;
            let g = (hex >> 8) & 0xFF;
            let b = hex & 0xFF;
            *pixel = r << 16 | g << 8 | b;
        });
}

fn create_scene(camera: Camera) -> Scene {
    let materials = AssetMaterials::new();
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    append_asset(
        &mut objects,
        build_base_diorama(Vec3::zeros(), 1.0, &materials),
    );
    append_asset(&mut objects, build_path(Vec3::zeros(), 1.0, &materials));
    append_asset(
        &mut objects,
        build_rock_garden(Vec3::new(-2.35, 0.1, 1.35), 1.0, &materials),
    );
    append_asset(
        &mut objects,
        build_sakura_tree_variant(Vec3::new(-2.65, 0.1, -0.5), 1.05, &materials, 0),
    );
    append_asset(
        &mut objects,
        build_sakura_tree_variant(Vec3::new(2.45, 0.1, 0.45), 0.68, &materials, 1),
    );
    append_asset(
        &mut objects,
        build_bamboo_cluster(Vec3::new(2.35, 0.1, -1.55), 1.05, &materials),
    );
    append_asset(
        &mut objects,
        build_toro_lantern(Vec3::new(2.85, 0.1, 1.72), 0.62, &materials),
    );
    append_asset(
        &mut objects,
        build_japanese_cafe(Vec3::new(-0.2, 0.15, -0.6), 1.0, &materials),
    );
    append_asset(
        &mut objects,
        build_cafe_patio(Vec3::new(-1.8, 0.1, 0.35), 0.95, &materials),
    );

    let lights = vec![Light {
        position: Vec3::new(0.0, 10.0, 10.0),
        color: Color::new(255, 247, 226),
        intensity: 2.4,
    }];

    let mut skybox = Skybox::new(Color::new(230, 242, 255));
    for (phase, name) in [
        (crate::core::scene::DayPhase::Day, "sky_day.png"),
        (crate::core::scene::DayPhase::Sunset, "sky_sunset.png"),
        (crate::core::scene::DayPhase::Night, "sky_night.png"),
        (crate::core::scene::DayPhase::Dawn, "sky_dawn.png"),
    ] {
        let paths = [
            format!(
                "assets/sky_{}/0.png",
                name.replace("sky_", "").replace(".png", "")
            ),
            format!(
                "assets/sky_{}/1.png",
                name.replace("sky_", "").replace(".png", "")
            ),
            format!(
                "assets/sky_{}/2.png",
                name.replace("sky_", "").replace(".png", "")
            ),
            format!(
                "assets/sky_{}/3.png",
                name.replace("sky_", "").replace(".png", "")
            ),
        ];
        let p_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let _ = skybox.load_phase_layers(phase, &p_refs);
    }

    Scene::new(objects, lights, camera, skybox)
}

fn create_camera() -> Camera {
    Camera::new(
        Vec3::new(2.9, 5.1, 7.5),
        Vec3::new(0.0, 0.35, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    )
}

fn add_primitive(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    shape: Box<dyn Shape>,
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
    material: Material,
) -> usize {
    let object_index = objects.len();
    objects.push(Box::new(Object::new(
        shape,
        Transform::new(position, rotation, scale),
        material,
    )));
    object_index
}

fn append_asset(objects: &mut Vec<Box<dyn RayIntersect>>, asset: Vec<Object>) -> Vec<usize> {
    let start = objects.len();
    objects.extend(
        asset
            .into_iter()
            .map(|object| Box::new(object) as Box<dyn RayIntersect>),
    );
    (start..objects.len()).collect()
}

fn create_shadow_test_scene() -> Scene {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    add_primitive(
        &mut objects,
        Box::new(Plane),
        Vec3::zeros(),
        Vec3::zeros(),
        Vec3::new(8.0, 1.0, 8.0),
        Material::new(Color::new(175, 168, 148)).with_albedo(0.85),
    );
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(-1.05, 0.65, 0.0),
        Vec3::zeros(),
        Vec3::new(1.3, 1.3, 1.3),
        Material::new(Color::new(174, 112, 75)).with_albedo(0.82),
    );
    add_primitive(
        &mut objects,
        Box::new(Sphere),
        Vec3::new(1.05, 0.52, 0.15),
        Vec3::zeros(),
        Vec3::new(0.52, 0.52, 0.52),
        Material::new(Color::new(94, 133, 165)).with_albedo(0.82),
    );
    let camera = Camera::new(
        Vec3::new(4.0, 4.8, 8.0),
        Vec3::new(0.0, 0.45, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let light = Light::new(Vec3::new(-3.0, 6.0, 4.0), Color::new(255, 247, 230), 1.0);
    let mut scene = Scene::new(
        objects,
        vec![light],
        camera,
        Skybox::new(Color::new(80, 108, 140)),
    );
    scene.ambient_intensity = 0.18;
    scene
}

fn create_material_test_scene() -> Scene {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    let materials = AssetMaterials::new();
    add_primitive(
        &mut objects,
        Box::new(Plane),
        Vec3::new(0.0, -0.04, 0.0),
        Vec3::zeros(),
        Vec3::new(9.0, 1.0, 5.0),
        Material::new(Color::new(120, 120, 120)).with_albedo(0.8),
    );

    let material_samples = [
        (Vec3::new(-3.2, 0.72, 0.0), materials.wood.clone()),
        (Vec3::new(-1.6, 0.72, 0.0), materials.paper.clone()),
        (Vec3::new(0.0, 0.72, 0.0), materials.stone.clone()),
        (Vec3::new(1.6, 0.72, 0.0), materials.metal.clone()),
    ];
    for (position, material) in material_samples {
        add_primitive(
            &mut objects,
            Box::new(Cube),
            position,
            Vec3::zeros(),
            Vec3::new(1.0, 1.0, 1.0),
            material,
        );
    }

    let mut glass = materials.glass.clone();
    glass.texture = None;
    add_primitive(
        &mut objects,
        Box::new(Sphere),
        Vec3::new(3.2, 0.72, 0.0),
        Vec3::zeros(),
        Vec3::new(0.58, 0.72, 0.58),
        glass,
    );

    let checker = Arc::new(Texture::procedural(
        crate::materials::texture::ProceduralTexture::Checkerboard,
    ));
    let mut checker_material = Material::new(Color::new(255, 255, 255)).with_texture(checker);
    checker_material.albedo = 1.0;
    add_primitive(
        &mut objects,
        Box::new(Plane),
        Vec3::new(3.2, 0.95, -0.85),
        Vec3::new(PI / 2.0, 0.0, 0.0),
        Vec3::new(1.8, 1.0, 2.0),
        checker_material,
    );

    let camera = Camera::new(
        Vec3::new(4.0, 4.0, 10.0),
        Vec3::new(0.0, 0.65, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let light = Light::new(Vec3::new(-2.0, 7.0, 5.0), Color::new(255, 248, 232), 1.0);
    let mut scene = Scene::new(
        objects,
        vec![light],
        camera,
        Skybox::new(Color::new(95, 125, 150)),
    );
    scene.ambient_intensity = 0.14;
    scene
}

fn save_test_render(framebuffer: &Framebuffer, path: &str) -> Result<(), image::ImageError> {
    let mut rgb = Vec::with_capacity(framebuffer.buffer.len() * 3);
    for pixel in &framebuffer.buffer {
        rgb.push(((pixel >> 16) & 0xff) as u8);
        rgb.push(((pixel >> 8) & 0xff) as u8);
        rgb.push((pixel & 0xff) as u8);
    }
    image::save_buffer(
        path,
        &rgb,
        framebuffer.width as u32,
        framebuffer.height as u32,
        image::ColorType::Rgb8,
    )
}

fn main() {
    let test_mode = std::env::args().nth(1);
    if matches!(
        test_mode.as_deref(),
        Some("--render-day1-flat" | "--render-day1-textured" | "--render-day1-rear")
    ) {
        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
        let rear_view = test_mode.as_deref() == Some("--render-day1-rear");
        let camera = if rear_view {
            Camera::new(
                Vec3::new(-3.2, 5.8, -8.5),
                Vec3::new(0.0, 0.35, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        } else {
            create_camera()
        };
        let scene = create_scene(camera);
        let flat = test_mode.as_deref() == Some("--render-day1-flat");
        let mode = if flat { 2 } else { 0 };
        let path = if flat {
            "target/day1-flat.png"
        } else if rear_view {
            "target/day1-rear.png"
        } else {
            "target/day1-textured.png"
        };
        render(&mut framebuffer, &scene, mode);
        match save_test_render(&framebuffer, path) {
            Ok(()) => println!("Saved Day 1 render to {path}"),
            Err(error) => eprintln!("Could not save Day 1 render: {error}"),
        }
        return;
    }
    if matches!(
        test_mode.as_deref(),
        Some("--test-shadows" | "--test-materials")
    ) {
        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
        let (scene, output_path) = if test_mode.as_deref() == Some("--test-materials") {
            (create_material_test_scene(), "target/material-test.png")
        } else {
            (create_shadow_test_scene(), "target/shadow-test.png")
        };
        let test_render_mode = if test_mode.as_deref() == Some("--test-materials") {
            0
        } else {
            2
        };
        render(&mut framebuffer, &scene, test_render_mode);
        match save_test_render(&framebuffer, output_path) {
            Ok(()) => println!("Saved renderer test to {output_path}"),
            Err(error) => eprintln!("Could not save renderer test: {error}"),
        }
        return;
    }

    let frame_delay = Duration::from_millis(16);
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "La Puerta Trasera | Día 1",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();
    let mut scene = create_scene(create_camera());

    let mut camera_moved = true;
    let mut render_mode = 0;
    let mut last_mouse_position: Option<(f32, f32)> = None;

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

        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let mouse_position = window.get_mouse_pos(MouseMode::Pass);
        if mouse_down {
            if let (Some((x, y)), Some((last_x, last_y))) = (mouse_position, last_mouse_position) {
                let drag_sensitivity = 0.006;
                let delta_yaw = -(x - last_x) * drag_sensitivity;
                let delta_pitch = (y - last_y) * drag_sensitivity;
                if delta_yaw != 0.0 || delta_pitch != 0.0 {
                    scene.camera.orbit(delta_yaw, delta_pitch);
                    camera_moved = true;
                }
            }
            last_mouse_position = mouse_position;
        } else {
            last_mouse_position = None;
        }

        if let Some((_, scroll_delta)) = window.get_scroll_wheel() {
            if scroll_delta != 0.0 {
                scene.camera.zoom(scroll_delta);
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
mod shadow_tests {
    use super::cast_shadow;
    use crate::core::camera::Camera;
    use crate::core::light::Light;
    use crate::core::object::Object;
    use crate::core::ray_intersect::{Intersect, Material, RayIntersect};
    use crate::core::scene::{Scene, Skybox};
    use crate::core::transform::Transform;
    use crate::materials::color::Color;
    use crate::shapes::cube::Cube;
    use crate::shapes::cylinder::Cylinder;
    use crate::shapes::plane::Plane;
    use nalgebra_glm::Vec3;

    fn shadow_scene(occluder: Option<Object>) -> (Scene, Intersect, Light) {
        let material = Material::new(Color::new(160, 160, 160));
        let receiver = Object::new(
            Box::new(Plane),
            Transform::new(Vec3::zeros(), Vec3::zeros(), Vec3::new(8.0, 1.0, 8.0)),
            material.clone(),
        );
        let mut objects: Vec<Box<dyn RayIntersect>> = vec![Box::new(receiver)];
        if let Some(occluder) = occluder {
            objects.push(Box::new(occluder));
        }
        let light = Light::new(Vec3::new(0.0, 5.0, 0.0), Color::new(255, 255, 255), 1.0);
        let camera = Camera::new(
            Vec3::new(0.0, 4.0, 8.0),
            Vec3::zeros(),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let scene = Scene::new(
            objects,
            vec![Light::new(
                Vec3::new(0.0, 5.0, 0.0),
                Color::new(255, 255, 255),
                1.0,
            )],
            camera,
            Skybox::new(Color::new(40, 50, 70)),
        );
        let hit = Intersect {
            object_index: 0,
            point: Vec3::zeros(),
            normal: Vec3::new(0.0, 1.0, 0.0),
            distance: 0.0,
            material,
            u: 0.5,
            v: 0.5,
        };
        (scene, hit, light)
    }

    fn test_occluder(shape: Box<dyn crate::core::object::Shape>, center_y: f32) -> Object {
        Object::new(
            shape,
            Transform::new(
                Vec3::new(0.0, center_y, 0.0),
                Vec3::zeros(),
                Vec3::new(0.8, 0.8, 0.8),
            ),
            Material::new(Color::new(100, 100, 100)),
        )
    }

    #[test]
    fn objects_behind_the_hit_do_not_cast_shadow() {
        let (scene, hit, light) = shadow_scene(Some(test_occluder(Box::new(Cylinder), -1.0)));
        assert!(!cast_shadow(
            &hit,
            &Vec3::new(0.0, 1.0, 0.0),
            &light,
            &scene
        ));
    }

    #[test]
    fn receiver_does_not_shadow_itself() {
        let (scene, hit, light) = shadow_scene(None);
        assert!(!cast_shadow(
            &hit,
            &Vec3::new(0.0, 1.0, 0.0),
            &light,
            &scene
        ));
    }

    #[test]
    fn object_between_hit_and_light_casts_shadow() {
        let (scene, hit, light) = shadow_scene(Some(test_occluder(Box::new(Cube), 2.0)));
        assert!(cast_shadow(&hit, &Vec3::new(0.0, 1.0, 0.0), &light, &scene));
    }

    #[test]
    fn objects_beyond_light_do_not_cast_shadow() {
        let (scene, hit, light) = shadow_scene(Some(test_occluder(Box::new(Cube), 6.0)));
        assert!(!cast_shadow(
            &hit,
            &Vec3::new(0.0, 1.0, 0.0),
            &light,
            &scene
        ));
    }
}
