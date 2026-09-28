mod camera;
mod color;
mod cylinder;
mod framebuffer;
mod light;
mod ray_intersect;
mod sphere;
mod cube;
mod texture;
mod transform;
mod pyramid;
mod cone;
mod plane;
mod triangle;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::cube::Cube;
use crate::pyramid::Pyramid;
use crate::cone::Cone;
use crate::plane::Plane;
use crate::triangle::Triangle;
use crate::cylinder::Cylinder;
use crate::transform::Transform;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BACKGROUND_COLOR: u32 = 0x040C24;

const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 60.0;

const SHADOW_BIAS: f32 = 1e-3;
const REFLECTION_BIAS: f32 = 1e-3;

const MAX_DEPTH: u32 = 3;

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

fn environment_color(_ray_origin: &Vec3, _ray_direction: &Vec3) -> Color {
    Color::from_hex(BACKGROUND_COLOR)
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
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
    render_mode: u8,
) -> Color {
    let light_direction = (light.position - intersect.point).normalize();
    let view_direction = (ray_origin - intersect.point).normalize();

    let light_intensity = if cast_shadow(intersect, &light_direction, light, objects) {
        0.0
    } else {
        light.intensity
    };

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

    let tile_world_normal = (tangent * tile_normal.x + bitangent * tile_normal.y + base_normal * tile_normal.z).normalize();

    let face_ambient = if base_normal.y < -0.5 { 0.45 } else { 1.0 };
    let ambient_intensity = 0.35 * face_ambient;
    let ambient = diffuse_color * ambient_intensity;

    let diffuse_intensity = dot(&tile_world_normal, &light_direction).max(0.0);

    let diffuse = diffuse_color
        * (diffuse_intensity * intersect.material.albedo[0] * light_intensity);

    let mut specular_normal = tile_world_normal;
    let mut specular_factor = intersect.material.albedo[1];
    let mut specular_exponent = intersect.material.specular;

    if render_mode != 2 {
        if let Some(specular_map) = &intersect.material.specular_map {
            specular_factor *= specular_map.get_intensity(u, v);
        }
    }

    let mut ambient_reflection = Color::new(0, 0, 0);

    if blend_factor > 0.0 {
        let water_world_normal = (tangent * water_normal_mapped.x + bitangent * water_normal_mapped.y + base_normal * water_normal_mapped.z).normalize();
        
        specular_normal = (tile_world_normal * (1.0 - blend_factor) + water_world_normal * blend_factor).normalize();

        let fresnel = (1.0 - dot(&view_direction, &water_world_normal).abs().clamp(0.0, 1.0)).powf(5.0);
        
        let sky_color = Color::new(100, 130, 160);
        ambient_reflection = sky_color * (fresnel * 0.6 * blend_factor);

        specular_factor = specular_factor * (1.0 - blend_factor)
            + (2.0 + 1.0 * fresnel) * blend_factor;
        specular_exponent = specular_exponent * (1.0 - blend_factor)
            + 150.0 * blend_factor;
    }

    let reflect_direction = reflect(&-light_direction, &specular_normal);
    let specular_intensity = dot(&view_direction, &reflect_direction)
        .max(0.0)
        .powf(specular_exponent);

    let specular = light.color * (specular_intensity * specular_factor * light_intensity);

    ambient + diffuse + specular + ambient_reflection
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    depth: u32,
    render_mode: u8,
) -> Color {
    if depth > MAX_DEPTH {
        return Color::from_hex(BACKGROUND_COLOR);
    }

    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest.as_ref().is_none_or(|current| intersect.distance < current.distance) {
                closest = Some(intersect);
            }
        }
    }

    let Some(intersect) = closest else {
        return environment_color(ray_origin, ray_direction);
    };

    let color = shade(&intersect, ray_origin, light, objects, render_mode);

    let reflectivity = intersect.material.albedo[2];

    if reflectivity <= 0.0 {
        return color;
    }

    let reflect_direction = reflect(ray_direction, &intersect.normal).normalize();
    let reflect_origin = intersect.point + intersect.normal * REFLECTION_BIAS;

    let reflected = cast_ray(
        &reflect_origin,
        &reflect_direction,
        objects,
        light,
        depth + 1,
        render_mode,
    );

    color * (1.0 - reflectivity) + reflected * reflectivity
}

use rayon::prelude::*;

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
    render_mode: u8,
) {
    let width_f = framebuffer.width as f32;
    let height_f = framebuffer.height as f32;
    let aspect_ratio = width_f / height_f;
    let perspective_scale = (FOV / 2.0).tan();
    let width = framebuffer.width;

    framebuffer.buffer.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let x = i % width;
        let y = i / width;

        let screen_x = (2.0 * x as f32) / width_f - 1.0;
        let screen_y = -(2.0 * y as f32) / height_f + 1.0;

        let screen_x = screen_x * aspect_ratio * perspective_scale;
        let screen_y = screen_y * perspective_scale;

        let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
        let ray_direction = camera.basis_change(&ray_direction);

        let sample_color = cast_ray(&camera.eye, &ray_direction, objects, light, 0, render_mode);
        let hex = sample_color.to_hex();
        
        let r = (hex >> 16) & 0xFF;
        let g = (hex >> 8) & 0xFF;
        let b = hex & 0xFF;
        *pixel = r << 16 | g << 8 | b;
    });
}

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new("Lakitu", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let tile_texture = std::sync::Arc::new(texture::Texture::new("assets/TilesSquarePoolMixed001_COL_2K.jpg"));
    let normal_map = std::sync::Arc::new(texture::Texture::new("assets/TilesSquarePoolMixed001_NRM_2K.jpg"));
    let specular_map = std::sync::Arc::new(texture::Texture::new("assets/TilesSquarePoolMixed001_REFL_2K.jpg"));

    let water_mask = std::sync::Arc::new(texture::Texture::new("assets/WaterDropletsMixedBubbled001_ALPHAMASKED_2K.png"));
    let water_normal = std::sync::Arc::new(texture::Texture::new("assets/WaterDropletsMixedBubbled001_NRM_2K.jpg"));

    let base_material = Material::new(Color::new(199, 159, 224), 250.0, [0.6, 1.0, 0.1])
        .with_texture(tile_texture.clone())
        .with_normal_map(normal_map.clone())
        .with_specular_map(specular_map.clone())
        .with_overlay(water_mask.clone(), water_normal.clone());

    let brown = Material::new(Color::new(139, 69, 19), 50.0, [0.8, 0.2, 0.0]);
    let green = Material::new(Color::new(34, 139, 34), 10.0, [0.9, 0.1, 0.0]);

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(Plane {
            transform: Transform::new(
                Vec3::new(0.0, -1.5, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(10.0, 1.0, 10.0),
            ),
            material: base_material.clone(),
        }),
        // Tronco (Cilindro)
        Box::new(Cylinder {
            transform: Transform::new(
                Vec3::new(0.0, -0.5, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.5, 2.0, 0.5),
            ),
            material: brown.clone(),
        }),
        // Hojas (Cono)
        Box::new(Cone {
            transform: Transform::new(
                Vec3::new(0.0, 1.5, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(2.0, 2.0, 2.0),
            ),
            material: green.clone(),
        }),
    ];

    let light = Light::new(Vec3::new(-4.5, 4.0, 6.0), Color::new(255, 250, 244), 2.0);

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.4, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

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
                camera.orbit(delta_yaw, delta_pitch);
                camera_moved = true;
            }
        }

        if camera_moved {
            render(&mut framebuffer, &objects, &camera, &light, render_mode);
            camera_moved = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
