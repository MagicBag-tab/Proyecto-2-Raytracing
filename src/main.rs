use crate::materials::texture::Texture;

mod core;
mod materials;
mod shapes;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::core::camera::Camera;
use crate::core::framebuffer::Framebuffer;
use crate::core::light::Light;
use crate::core::object::{Object, Shape};
use crate::core::ray_intersect::{Intersect, Material, RayIntersect};
use crate::core::scene::{Scene, Skybox};
use crate::core::transform::Transform;
use crate::materials::color::Color;
use crate::shapes::cone::Cone;
use crate::shapes::cube::Cube;
use crate::shapes::cylinder::Cylinder;
use crate::shapes::plane::Plane;
use crate::shapes::pyramid::Pyramid;
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
) {
    objects.push(Box::new(Object {
        shape,
        transform: Transform::new(position, rotation, scale),
        material,
    }));
}

fn create_scene(camera: Camera) -> Scene {
    use crate::materials::presets::{create, Preset};
    use crate::materials::texture::ProceduralTexture;

    let wood = create(Preset::Wood);
    let stone = create(Preset::Stone);
    let metal = create(Preset::Metal);
    let paper = create(Preset::Paper);
    let glass = create(Preset::Glass);
    let grass = Material::new(Color::new(63, 122, 58))
        .with_albedo(0.92)
        .with_specular(10.0, 0.04)
        .with_texture(std::sync::Arc::new(Texture::procedural(
            ProceduralTexture::Grass,
        )));
    let cream = paper.clone();
    let terracotta = Material::new(Color::new(157, 61, 47))
        .with_albedo(0.82)
        .with_specular(24.0, 0.16);
    let foliage = Material::new(Color::new(44, 112, 54))
        .with_albedo(0.92)
        .with_specular(12.0, 0.04);
    let flower_pink = Material::new(Color::new(229, 91, 143))
        .with_albedo(0.88)
        .with_specular(28.0, 0.16);
    let flower_yellow = Material::new(Color::new(248, 190, 65))
        .with_albedo(0.9)
        .with_specular(24.0, 0.18);
    let warm_glow = Material::new(Color::new(255, 202, 112))
        .with_albedo(0.35)
        .with_specular(64.0, 0.5)
        .with_transparency(0.18, 1.15);

    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    // Raised garden platform and grass surface.
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -3.05, 0.0),
        Vec3::zeros(),
        Vec3::new(13.0, 0.55, 10.5),
        stone.clone(),
    );
    add_primitive(
        &mut objects,
        Box::new(Plane),
        Vec3::new(0.0, -2.77, 0.0),
        Vec3::zeros(),
        Vec3::new(12.5, 1.0, 10.0),
        grass.clone(),
    );

    // Cafe body, pitched roof, entrance, windows, awning, sign, and chimney.
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -1.37, -2.25),
        Vec3::zeros(),
        Vec3::new(3.8, 2.8, 2.45),
        cream.clone(),
    );
    add_primitive(
        &mut objects,
        Box::new(Pyramid),
        Vec3::new(0.0, 0.43, -2.25),
        Vec3::zeros(),
        Vec3::new(4.35, 1.3, 2.85),
        terracotta.clone(),
    );
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -2.0, -0.995),
        Vec3::zeros(),
        Vec3::new(0.78, 1.55, 0.08),
        wood.clone(),
    );
    add_primitive(
        &mut objects,
        Box::new(Sphere),
        Vec3::new(0.27, -2.0, -0.91),
        Vec3::zeros(),
        Vec3::new(0.1, 0.1, 0.1),
        metal.clone(),
    );
    for x in [-1.2, 1.2] {
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -1.45, -0.98),
            Vec3::zeros(),
            Vec3::new(0.86, 0.78, 0.08),
            glass.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -1.45, -0.91),
            Vec3::zeros(),
            Vec3::new(0.94, 0.08, 0.08),
            wood.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -1.45, -0.91),
            Vec3::zeros(),
            Vec3::new(0.08, 0.86, 0.08),
            wood.clone(),
        );
    }
    for (index, x) in [-1.5, -0.75, 0.0, 0.75, 1.5].into_iter().enumerate() {
        let awning_color = if index % 2 == 0 {
            terracotta.clone()
        } else {
            cream.clone()
        };
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -0.72, -0.72),
            Vec3::zeros(),
            Vec3::new(0.74, 0.24, 0.62),
            awning_color,
        );
    }
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -0.18, -0.94),
        Vec3::zeros(),
        Vec3::new(1.5, 0.35, 0.12),
        wood.clone(),
    );
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(1.2, 1.05, -2.6),
        Vec3::zeros(),
        Vec3::new(0.48, 1.4, 0.48),
        stone.clone(),
    );

    // Conifers and broad-canopy trees.
    for (x, z, height) in [(-4.7, -1.3, 3.2), (4.7, -1.0, 3.6)] {
        add_primitive(
            &mut objects,
            Box::new(Cylinder),
            Vec3::new(x, -1.52, z),
            Vec3::zeros(),
            Vec3::new(0.34, 2.5, 0.34),
            wood.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Cone),
            Vec3::new(x, -1.15 + height * 0.18, z),
            Vec3::zeros(),
            Vec3::new(2.25, height, 2.25),
            foliage.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Cone),
            Vec3::new(x, -0.45 + height * 0.18, z),
            Vec3::zeros(),
            Vec3::new(1.65, height * 0.78, 1.65),
            foliage.clone(),
        );
    }
    for (x, z) in [(-5.1, 2.5), (4.9, 2.1)] {
        add_primitive(
            &mut objects,
            Box::new(Cylinder),
            Vec3::new(x, -1.62, z),
            Vec3::zeros(),
            Vec3::new(0.3, 2.3, 0.3),
            wood.clone(),
        );
        for (dx, dy, dz, radius) in [
            (0.0, -0.3, 0.0, 1.25),
            (-0.62, 0.18, 0.0, 0.9),
            (0.55, 0.25, 0.12, 0.88),
            (0.0, 0.58, -0.35, 0.82),
        ] {
            add_primitive(
                &mut objects,
                Box::new(Sphere),
                Vec3::new(x + dx, dy, z + dz),
                Vec3::zeros(),
                Vec3::new(radius, radius, radius),
                foliage.clone(),
            );
        }
    }

    // Irregular stones around the planting beds.
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
            stone.clone(),
        );
    }

    // Flowers and low shrubs on both sides of the stepping-stone path.
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
            foliage.clone(),
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
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(x + 0.22, -2.38, z + 0.12),
            Vec3::zeros(),
            Vec3::new(0.34, 0.26, 0.32),
            foliage.clone(),
        );
    }

    // Entry gate, stone pillars, metal bars, and a stepping-stone path.
    for x in [-1.55, 1.55] {
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -1.88, 4.45),
            Vec3::zeros(),
            Vec3::new(0.48, 1.65, 0.48),
            stone.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(x, -0.94, 4.45),
            Vec3::zeros(),
            Vec3::new(0.32, 0.32, 0.32),
            terracotta.clone(),
        );
    }
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -1.1, 4.45),
        Vec3::zeros(),
        Vec3::new(2.65, 0.18, 0.22),
        metal.clone(),
    );
    for x in [-1.05, -0.53, 0.0, 0.53, 1.05] {
        add_primitive(
            &mut objects,
            Box::new(Cylinder),
            Vec3::new(x, -1.88, 4.45),
            Vec3::zeros(),
            Vec3::new(0.055, 1.35, 0.055),
            metal.clone(),
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
                cream.clone()
            } else {
                stone.clone()
            },
        );
    }

    // Lantern posts and warm point lights.
    let mut lights = vec![Light::new(
        Vec3::new(-4.0, 7.0, 7.0),
        Color::new(255, 242, 220),
        2.4,
    )];
    for x in [-5.45, 5.45] {
        add_primitive(
            &mut objects,
            Box::new(Cylinder),
            Vec3::new(x, -1.55, 3.25),
            Vec3::zeros(),
            Vec3::new(0.12, 2.45, 0.12),
            metal.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Sphere),
            Vec3::new(x, -0.15, 3.25),
            Vec3::zeros(),
            Vec3::new(0.32, 0.32, 0.32),
            warm_glow.clone(),
        );
        add_primitive(
            &mut objects,
            Box::new(Cone),
            Vec3::new(x, 0.2, 3.25),
            Vec3::zeros(),
            Vec3::new(0.7, 0.7, 0.7),
            metal.clone(),
        );
        lights.push(Light::new(
            Vec3::new(x, -0.15, 3.25),
            Color::new(255, 190, 104),
            0.85,
        ));
    }

    // Low garden edging defines the diorama footprint.
    for x in [-6.25, 6.25] {
        add_primitive(
            &mut objects,
            Box::new(Cube),
            Vec3::new(x, -2.52, 0.0),
            Vec3::zeros(),
            Vec3::new(0.22, 0.45, 9.9),
            wood.clone(),
        );
    }
    add_primitive(
        &mut objects,
        Box::new(Cube),
        Vec3::new(0.0, -2.52, -5.0),
        Vec3::zeros(),
        Vec3::new(12.5, 0.45, 0.22),
        wood.clone(),
    );

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
        let mouse_position = window.get_mouse_pos(MouseMode::Clamp);
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
mod tests {
    use super::*;
    use crate::core::object::Shape;
    use crate::materials::presets::{self, Preset};
    use crate::shapes::cone::Cone;
    use crate::shapes::cube::Cube;
    use crate::shapes::cylinder::Cylinder;
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
    fn cafe_garden_scene_contains_primitive_built_landmarks() {
        let scene = create_scene(create_camera());
        assert!(scene.objects.len() >= 70);
        assert!(scene.lights.len() >= 3);

        let ground_hit = scene.objects[0]
            .ray_intersect(&Vec3::new(0.0, 5.0, 0.0), &Vec3::new(0.0, -1.0, 0.0))
            .expect("raised garden platform should be intersectable");
        assert!(ground_hit.material.texture.is_some());

        let cafe_door_hit = scene
            .objects
            .iter()
            .filter_map(|object| {
                object.ray_intersect(&Vec3::new(0.0, -2.0, 1.0), &Vec3::new(0.0, 0.0, -1.0))
            })
            .min_by(|left, right| left.distance.total_cmp(&right.distance))
            .expect("cafe entrance should be visible from the garden");
        assert_eq!(
            cafe_door_hit.material.diffuse.to_hex(),
            Color::new(150, 86, 42).to_hex()
        );

        let path_hit = scene
            .objects
            .iter()
            .filter_map(|object| {
                object.ray_intersect(&Vec3::new(0.0, 3.0, 3.0), &Vec3::new(0.0, -1.0, 0.0))
            })
            .min_by(|left, right| left.distance.total_cmp(&right.distance))
            .expect("stepping path should be visible in front of the cafe");
        assert_eq!(
            path_hit.material.diffuse.to_hex(),
            Color::new(125, 132, 139).to_hex()
        );

        let gate_hit = scene
            .objects
            .iter()
            .filter_map(|object| {
                object.ray_intersect(&Vec3::new(0.0, -1.8, 6.0), &Vec3::new(0.0, 0.0, -1.0))
            })
            .min_by(|left, right| left.distance.total_cmp(&right.distance))
            .expect("gate bars should cross the garden entrance");
        assert_eq!(
            gate_hit.material.diffuse.to_hex(),
            Color::new(170, 180, 190).to_hex()
        );

        let tree_hit = scene
            .objects
            .iter()
            .filter_map(|object| {
                object.ray_intersect(&Vec3::new(-4.7, 4.0, -1.3), &Vec3::new(0.0, -1.0, 0.0))
            })
            .min_by(|left, right| left.distance.total_cmp(&right.distance))
            .expect("a tree canopy should occupy the garden");
        assert_eq!(
            tree_hit.material.diffuse.to_hex(),
            Color::new(44, 112, 54).to_hex()
        );
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
    fn glass_sphere_refracts_checkerboard_background() {
        let mut glass = presets::create(Preset::Glass);
        glass.albedo = 0.0;
        glass.reflectivity = 0.0;
        glass.specular_strength = 0.0;
        let mut air = glass.clone();
        air.refractive_index = 1.0;

        let checker_material = Material::new(Color::new(220, 224, 222))
            .with_albedo(1.0)
            .with_specular(1.0, 0.0)
            .with_texture(Arc::new(Texture::procedural(
                crate::materials::texture::ProceduralTexture::Checkerboard,
            )));
        let make_scene = |sphere_material| {
            Scene::new(
                vec![
                    Box::new(Object {
                        shape: Box::new(Sphere),
                        transform: Transform::default(),
                        material: sphere_material,
                    }) as Box<dyn RayIntersect>,
                    Box::new(Object {
                        shape: Box::new(Plane),
                        transform: Transform::new(
                            Vec3::new(0.0, 0.0, -1.55),
                            Vec3::new(PI / 2.0, 0.0, 0.0),
                            Vec3::new(14.0, 1.0, 9.0),
                        ),
                        material: checker_material.clone(),
                    }),
                ],
                Vec::new(),
                create_camera(),
                Skybox::new(Color::from_hex(SKY_COLOR)),
            )
        };

        let glass_scene = make_scene(glass);
        let air_scene = make_scene(air);
        let ray_origin = Vec3::new(-0.91, 0.0, 3.0);
        let ray_direction = normalize(&Vec3::new(0.2, 0.0, -1.0));
        let glass_color = cast_ray(&ray_origin, &ray_direction, &glass_scene, 0, 0);
        let air_color = cast_ray(&ray_origin, &ray_direction, &air_scene, 0, 0);

        assert_ne!(glass_color.to_hex(), air_color.to_hex());
    }

    #[test]
    fn perfectly_reflective_plane_returns_reflected_scene_color() {
        let material = Material::new(Color::new(0, 0, 0))
            .with_albedo(0.0)
            .with_reflectivity(1.0);
        let plane = Object {
            shape: Box::new(Plane),
            transform: Transform::default(),
            material,
        };
        let reflected_scene = Scene::new(
            vec![Box::new(plane)],
            Vec::new(),
            create_camera(),
            Skybox::new(Color::from_hex(SKY_COLOR)),
        );
        let ray_origin = Vec3::new(0.0, 1.0, 0.0);
        let ray_direction = Vec3::new(0.0, -1.0, 0.0);
        let expected = reflected_scene
            .skybox
            .sample(&Vec3::new(0.0, 1.0, 0.0))
            .to_hex();

        let reflected_color = cast_ray(&ray_origin, &ray_direction, &reflected_scene, 0, 0);

        assert_eq!(reflected_color.to_hex(), expected);
    }

    #[test]
    fn current_scene_renders_geometry() {
        let scene = create_scene(create_camera());
        let mut framebuffer = Framebuffer::new(80, 60);

        render(&mut framebuffer, &scene, 0);

        assert!(framebuffer.buffer.iter().any(|pixel| *pixel != SKY_COLOR));
    }
}
