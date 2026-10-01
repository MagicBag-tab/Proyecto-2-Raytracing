pub mod assets;
mod core;
mod materials;
mod shapes;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::assets::{
    create_back_door, create_chair, create_fence, create_house, create_lantern, create_sakura,
    create_table, create_tree, AssetMaterials,
};
use crate::core::camera::Camera;
use crate::core::framebuffer::Framebuffer;
use crate::core::interaction::{InteractiveKind, ObjectId};
use crate::core::light::Light;
use crate::core::object::{Object, Shape};
use crate::core::picking::pick;
use crate::core::ray_intersect::{Intersect, Material, RayIntersect};
use crate::core::scene::{Scene, Skybox};
use crate::core::transform::Transform;
use crate::materials::color::Color;
use crate::shapes::cube::Cube;
use crate::shapes::cylinder::Cylinder;
use crate::shapes::plane::Plane;
use crate::shapes::sphere::Sphere;

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
    scene: &Scene,
) -> bool {
    let shadow_ray_origin = intersect.point + intersect.normal * SHADOW_BIAS;
    let light_distance = (light.position - intersect.point).magnitude();

    scene.objects.iter().enumerate().any(|(index, object)| {
        if !scene.is_object_visible(ObjectId(index)) {
            return false;
        }
        object
            .ray_intersect_distance(&shadow_ray_origin, light_direction)
            .is_some_and(|blocker_distance| blocker_distance < light_distance)
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

fn create_camera() -> Camera {
    Camera::new(
        Vec3::new(0.0, 5.2, 18.0),
        Vec3::new(0.0, -0.45, 0.0),
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

fn create_scene(camera: Camera) -> Scene {
    let materials = AssetMaterials::new();
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    // Raised garden platform and grass.
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -3.05, 0.0),
        Vec3::zeros(),
        Vec3::new(13.0, 0.55, 10.5),
        materials.stone.clone(),
    );
    add_primitive(
        &mut objects,
        Box::new(Plane),
        Vec3::new(0.0, -2.77, 0.0),
        Vec3::zeros(),
        Vec3::new(12.5, 1.0, 10.0),
        materials.grass.clone(),
    );

    append_asset(
        &mut objects,
        create_house(Vec3::new(0.0, -2.27, -2.25), 1.0, &materials),
    );
    append_asset(
        &mut objects,
        create_tree(Vec3::new(-4.7, -2.77, -1.3), 1.0, &materials),
    );
    append_asset(
        &mut objects,
        create_tree(Vec3::new(4.7, -2.77, -1.0), 1.1, &materials),
    );
    append_asset(
        &mut objects,
        create_sakura(Vec3::new(-5.1, -2.77, 2.5), 1.0, &materials),
    );
    append_asset(
        &mut objects,
        create_sakura(Vec3::new(4.9, -2.77, 2.1), 0.95, &materials),
    );

    // Seating beside the cafe, leaving the central path open.
    append_asset(
        &mut objects,
        create_table(Vec3::new(-3.0, -2.77, -0.7), 0.75, &materials),
    );
    append_asset(
        &mut objects,
        create_chair(Vec3::new(-4.15, -2.77, -0.7), 0.65, &materials),
    );
    append_asset(
        &mut objects,
        create_chair(Vec3::new(-1.85, -2.77, -0.7), 0.65, &materials),
    );
    append_asset(
        &mut objects,
        create_table(Vec3::new(3.0, -2.77, -0.7), 0.75, &materials),
    );
    append_asset(
        &mut objects,
        create_chair(Vec3::new(1.85, -2.77, -0.7), 0.65, &materials),
    );
    append_asset(
        &mut objects,
        create_chair(Vec3::new(4.15, -2.77, -0.7), 0.65, &materials),
    );

    // Fence sections preserve a clear opening at the front gate.
    append_asset(
        &mut objects,
        create_fence(Vec3::new(0.0, -2.77, -4.85), 12.0, &materials),
    );
    append_asset(
        &mut objects,
        create_fence(Vec3::new(-4.8, -2.77, 4.85), 3.0, &materials),
    );
    append_asset(
        &mut objects,
        create_fence(Vec3::new(4.8, -2.77, 4.85), 3.0, &materials),
    );
    for x in [-1.55, 1.55] {
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -1.88, 4.45),
            Vec3::zeros(),
            Vec3::new(0.48, 1.65, 0.48),
            materials.stone.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(x, -0.94, 4.45),
            Vec3::zeros(),
            Vec3::new(0.32, 0.32, 0.32),
            materials.terracotta.clone(),
        );
    }
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -1.1, 4.45),
        Vec3::zeros(),
        Vec3::new(2.65, 0.18, 0.22),
        materials.metal.clone(),
    );
    for x in [-1.05, -0.53, 0.0, 0.53, 1.05] {
        add_primitive(
            &mut objects,
            Box::new(Cylinder),
            Vec3::new(x, -1.88, 4.45),
            Vec3::zeros(),
            Vec3::new(0.055, 1.35, 0.055),
            materials.metal.clone(),
        );
    }

    for x in [-5.45, 5.45] {
        append_asset(
            &mut objects,
            create_lantern(Vec3::new(x, -2.77, 3.25), 0.8, &materials),
        );
    }

    // Decorative stones, flower clumps, and stepping stones.
    for (x, z, scale) in [
        (-3.7, 1.1, Vec3::new(0.9, 0.48, 0.68)),
        (-3.0, 1.6, Vec3::new(0.58, 0.34, 0.5)),
        (3.45, 1.3, Vec3::new(0.82, 0.44, 0.7)),
        (3.9, 2.0, Vec3::new(0.5, 0.3, 0.55)),
        (-5.6, -0.2, Vec3::new(0.7, 0.38, 0.55)),
    ] {
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(x, -2.43, z),
            Vec3::zeros(),
            scale,
            materials.stone.clone(),
        );
    }

    let flower_pink = Material::new(Color::new(229, 91, 143)).with_specular(28.0, 0.16);
    let flower_yellow = Material::new(Color::new(248, 190, 65)).with_specular(24.0, 0.18);
    for (index, (x, z)) in [
        (-3.6, -0.1),
        (-3.1, 0.35),
        (-3.9, 0.65),
        (3.25, -0.2),
        (3.75, 0.25),
        (3.2, 0.75),
        (-2.9, 2.3),
        (2.8, 2.6),
    ]
    .into_iter()
    .enumerate()
    {
        add_primitive(
            &mut objects,
            Box::new(Cylinder),
            Vec3::new(x, -2.38, z),
            Vec3::zeros(),
            Vec3::new(0.055, 0.62, 0.055),
            materials.foliage.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(x, -2.03, z),
            Vec3::zeros(),
            Vec3::new(0.2, 0.2, 0.2),
            if index % 2 == 0 {
                flower_pink.clone()
            } else {
                flower_yellow.clone()
            },
        );
    }

    for (index, z) in [3.8, 3.0, 2.2, 1.4, 0.6, -0.2].into_iter().enumerate() {
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(0.0, -2.64, z),
            Vec3::zeros(),
            Vec3::new(0.92, 0.12, 0.64),
            if index % 2 == 0 {
                materials.paper.clone()
            } else {
                materials.stone.clone()
            },
        );
    }

    // Low garden edging and warm lantern lighting.
    for x in [-6.25, 6.25] {
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -2.52, 0.0),
            Vec3::zeros(),
            Vec3::new(0.22, 0.45, 9.9),
            materials.wood.clone(),
        );
    }
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -2.52, -5.0),
        Vec3::zeros(),
        Vec3::new(12.5, 0.45, 0.22),
        materials.wood.clone(),
    );

    let clue_material = Material::new(Color::new(232, 190, 104))
        .with_specular(48.0, 0.45)
        .with_emission(Color::new(255, 177, 74), 0.35);
    let clue_indices = [
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(-4.4, -2.34, 1.55),
            Vec3::zeros(),
            Vec3::new(0.28, 0.28, 0.28),
            clue_material.clone(),
        ),
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(1.55, -2.47, -0.45),
            Vec3::zeros(),
            Vec3::new(0.34, 0.06, 0.25),
            materials.paper.clone(),
        ),
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(0.0, -2.35, 2.65),
            Vec3::zeros(),
            Vec3::new(0.24, 0.24, 0.24),
            clue_material,
        ),
    ];

    let door_indices = append_asset(
        &mut objects,
        create_back_door(Vec3::new(0.0, -2.05, -3.505), 1.0, &materials),
    );
    let movable_index = add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(2.35, -2.48, 1.55),
        Vec3::zeros(),
        Vec3::new(0.42, 0.42, 0.42),
        materials.wood.clone(),
    );

    let mut secret_room_indices = Vec::new();
    for (position, size, material) in [
        (
            Vec3::new(0.0, -2.68, -5.15),
            Vec3::new(3.8, 0.16, 3.1),
            materials.stone.clone(),
        ),
        (
            Vec3::new(-1.84, -1.25, -5.15),
            Vec3::new(0.16, 2.8, 3.1),
            materials.paper.clone(),
        ),
        (
            Vec3::new(1.84, -1.25, -5.15),
            Vec3::new(0.16, 2.8, 3.1),
            materials.paper.clone(),
        ),
        (
            Vec3::new(0.0, -1.25, -6.62),
            Vec3::new(3.8, 2.8, 0.16),
            materials.stone.clone(),
        ),
    ] {
        secret_room_indices.push(add_primitive(
            &mut objects,
            Box::new(Cube),
            position,
            Vec3::zeros(),
            size,
            material,
        ));
    }
    secret_room_indices.push(add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(-0.78, -1.45, -6.48),
        Vec3::zeros(),
        Vec3::new(0.72, 1.45, 0.08),
        materials.metal.clone().with_reflectivity(0.72),
    ));
    secret_room_indices.push(add_primitive(
        &mut objects,
        Box::new(Sphere),
        Vec3::new(0.1, -2.31, -5.25),
        Vec3::zeros(),
        Vec3::new(0.25, 0.3, 0.25),
        materials.glass.clone(),
    ));
    secret_room_indices.push(add_primitive(
        &mut objects,
        Box::new(Cylinder),
        Vec3::new(0.88, -2.22, -5.95),
        Vec3::zeros(),
        Vec3::new(0.12, 0.48, 0.12),
        materials.warm_glow.clone(),
    ));

    let lights = vec![
        Light::new(Vec3::new(-4.0, 7.0, 7.0), Color::new(255, 242, 220), 2.4),
        Light::new(
            Vec3::new(-5.45, -0.15, 3.25),
            Color::new(255, 190, 104),
            0.85,
        ),
        Light::new(
            Vec3::new(5.45, -0.15, 3.25),
            Color::new(255, 190, 104),
            0.85,
        ),
        Light::new(Vec3::new(0.88, -1.72, -5.8), Color::new(255, 173, 92), 0.65),
    ];

    let mut skybox = Skybox::new(Color::from_hex(SKY_COLOR));
    for (phase, paths) in [
        (
            crate::core::scene::DayPhase::Day,
            vec![
                "assets/sky_day/1.png",
                "assets/sky_day/2.png",
                "assets/sky_day/3.png",
                "assets/sky_day/4.png",
            ],
        ),
        (
            crate::core::scene::DayPhase::Sunset,
            vec![
                "assets/sky_sunset/1.png",
                "assets/sky_sunset/2.png",
                "assets/sky_sunset/3.png",
                "assets/sky_sunset/4.png",
            ],
        ),
        (
            crate::core::scene::DayPhase::Night,
            vec![
                "assets/sky_night/1.png",
                "assets/sky_night/2.png",
                "assets/sky_night/3.png",
                "assets/sky_night/4.png",
            ],
        ),
        (
            crate::core::scene::DayPhase::Dawn,
            vec![
                "assets/sky_dawn/1.png",
                "assets/sky_dawn/2.png",
                "assets/sky_dawn/3.png",
                "assets/sky_dawn/4.png",
            ],
        ),
    ] {
        if let Err(error) = skybox.load_phase_layers(phase, &paths) {
            eprintln!("Could not load sky layers for {:?}: {}", phase, error);
        }
    }

    let mut scene = Scene::new(objects, lights, camera, skybox);
    for (clue_index, object_index) in clue_indices.into_iter().enumerate() {
        scene.register_interactive(object_index, InteractiveKind::Clue(clue_index), true);
    }
    for object_index in door_indices {
        scene.register_door_part(object_index);
    }
    scene.register_interactive(movable_index, InteractiveKind::Movable, true);
    for object_index in secret_room_indices {
        scene.register_secret_room_object(object_index);
    }
    scene
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

fn phase_name(phase: crate::core::scene::DayPhase) -> &'static str {
    match phase {
        crate::core::scene::DayPhase::Day => "DAY",
        crate::core::scene::DayPhase::Sunset => "SUNSET",
        crate::core::scene::DayPhase::Night => "NIGHT",
        crate::core::scene::DayPhase::Dawn => "DAWN",
    }
}

fn update_window_title(window: &mut Window, scene: &Scene) {
    let door_status = if scene.game_state.door_unlocked {
        if scene.game_state.secret_room_open {
            "Open"
        } else {
            "Unlocked"
        }
    } else {
        "Locked"
    };
    window.set_title(&format!(
        "La Puerta Trasera | {} | Clues: {}/3 | Door: {}",
        phase_name(scene.day_phase),
        scene.game_state.clues_count(),
        door_status,
    ));
}

fn main() {
    let frame_delay = Duration::from_millis(16);
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new("Lakitu", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut scene = create_scene(create_camera());

    let mut camera_moved = true;
    let mut render_mode = 0;
    let mut last_mouse_position: Option<(f32, f32)> = None;
    let mut mouse_press_position: Option<(f32, f32)> = None;
    let mut was_mouse_down = false;
    update_window_title(&mut window, &scene);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::Tab, minifb::KeyRepeat::No) {
            scene.advance_day_phase();
            camera_moved = true;
            update_window_title(&mut window, &scene);
        }

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

        if let Some(selected) = scene.game_state.selected_object {
            let movement = [
                (Key::W, Vec3::new(0.0, 0.0, -0.06)),
                (Key::S, Vec3::new(0.0, 0.0, 0.06)),
                (Key::A, Vec3::new(-0.06, 0.0, 0.0)),
                (Key::D, Vec3::new(0.06, 0.0, 0.0)),
            ];
            for (key, delta) in movement {
                if window.is_key_down(key) && scene.move_interactive(selected, delta) {
                    camera_moved = true;
                }
            }
        }

        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let mouse_position = window.get_mouse_pos(MouseMode::Clamp);
        if mouse_down {
            if !was_mouse_down {
                mouse_press_position = mouse_position;
            }
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
            if was_mouse_down {
                let click_position = mouse_position.or(mouse_press_position);
                if let (Some((press_x, press_y)), Some((x, y))) =
                    (mouse_press_position, click_position)
                {
                    if (x - press_x).powi(2) + (y - press_y).powi(2) <= 25.0 {
                        if let Some(hit) = pick(&scene, &scene.camera, x, y, WIDTH, HEIGHT, FOV) {
                            let id = ObjectId(hit.object_index);
                            scene.game_state.selected_object = Some(id);
                            match scene.interaction_for(id) {
                                Some(InteractiveKind::Clue(_)) => {
                                    if scene.discover_clue(id) {
                                        println!(
                                            "Clue found: {}/3",
                                            scene.game_state.clues_count()
                                        );
                                        if scene.game_state.door_unlocked {
                                            println!("The back door is unlocked.");
                                        }
                                        camera_moved = true;
                                    }
                                }
                                Some(InteractiveKind::BackDoor) => {
                                    if scene.toggle_secret_room() {
                                        println!(
                                            "Back door {}.",
                                            if scene.game_state.secret_room_open {
                                                "opened"
                                            } else {
                                                "closed"
                                            }
                                        );
                                        camera_moved = true;
                                    } else {
                                        println!("The back door is locked.");
                                    }
                                }
                                Some(InteractiveKind::Movable) => {
                                    println!("Movable object selected. Use WASD to move it.");
                                }
                                None => println!("Selected object {}.", id.0),
                            }
                        } else {
                            scene.game_state.selected_object = None;
                        }
                        update_window_title(&mut window, &scene);
                    }
                }
            }
            mouse_press_position = None;
            last_mouse_position = None;
        }
        was_mouse_down = mouse_down;

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
