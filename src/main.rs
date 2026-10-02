use crate::core::interaction::ObjectId;
pub mod assets;
mod core;
mod materials;
mod shapes;

use crate::assets::build_back_door;
use crate::assets::{
    build_day2_flowers, build_more_garden_flowers, build_day3_decorations, build_day5_higanbana, build_day6_clue,
    build_day7_clue,
};
use crate::core::interaction::{InteractiveKind, PointerGesture};
use crate::core::picking::pick;

use crate::core::scene::DayPhase;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::assets::{
    build_back_garden_plants, build_bamboo_cluster, build_base_diorama, build_cafe_interior,
    build_cafe_patio, build_japanese_cafe, build_japanese_ruins, build_path, build_rock_garden,
    build_sakura_tree_variant, build_secret_room_interior, build_torii_gate, build_toro_lantern,
    build_tsukubai, AssetMaterials,
};
use crate::core::camera::Camera;
use crate::core::framebuffer::Framebuffer;
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

#[derive(PartialEq, Clone)]
pub enum AppMode {
    MainMenu,
    Exploration,
    Cinematic,
}

#[derive(Clone)]
pub struct CinematicState {
    pub start_time: Instant,
    pub original_camera: Camera,
}

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 60.0;

const SHADOW_BIAS_SCALE: f32 = 2e-5;
const REFRACTION_BIAS: f32 = 1e-3;

const MAX_DEPTH: u32 = 3;

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

fn toggle_secret_room_shortcut(scene: &mut Scene) -> bool {
    scene.game_state.door_unlocked = true;
    scene.toggle_secret_room()
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
        if let Some(blocker_intersect) = object.ray_intersect(&shadow_ray_origin, &shadow_direction)
        {
            if blocker_intersect.distance > 0.0 && blocker_intersect.distance < light_distance {
                let mut alpha = 1.0;
                if let Some(texture) = &blocker_intersect.material.texture {
                    alpha = texture.get_alpha(blocker_intersect.u, blocker_intersect.v);
                }
                return alpha >= 0.5;
            }
        }
        false
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
        let bottom_shadow = 1.0 - (bottom_factor * 0.4 * blend_factor); // 0.4 controla que tan oscura es

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
        if light.intensity <= 0.0 {
            continue;
        }
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
        return environment_color(scene, ray_origin, ray_direction);
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
        return environment_color(scene, ray_origin, ray_direction);
    };

    let mut alpha = 1.0;
    if let Some(texture) = &intersect.material.texture {
        alpha = texture.get_alpha(intersect.u, intersect.v);
    }
    if alpha < 0.5 {
        // Alpha cutout: continue the ray from just past the intersection
        let new_origin = intersect.point + ray_direction * 1e-4;
        return cast_ray(&new_origin, ray_direction, scene, depth, render_mode);
    }

    let color = shade(&intersect, ray_origin, scene, render_mode);

    let reflectivity = intersect.material.reflectivity;
    let transparency = intersect.material.transparency;

    let mut final_color = color;

    if reflectivity > 0.0 || transparency > 0.0 {
        let incident = normalize(ray_direction);
        let mut normal = intersect.normal;
        let entering = dot(&incident, &normal) < 0.0;
        let (eta_i, eta_t) = if entering {
            (1.0, intersect.material.refractive_index)
        } else {
            normal = -normal;
            (intersect.material.refractive_index, 1.0)
        };

        let f0 = ((eta_i - eta_t) / (eta_i + eta_t)).powi(2);
        let cos_theta = -dot(&incident, &normal).max(0.0);
        let fresnel = f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5);

        let reflection_weight = if transparency > 0.0 {
            fresnel
        } else {
            reflectivity
        };
        let transmission_weight = transparency * (1.0 - fresnel);

        let reflected_direction = incident - normal * 2.0 * dot(&incident, &normal);
        let eta = eta_i / eta_t;
        let k = 1.0 - eta * eta * (1.0 - cos_theta * cos_theta);

        let refracted_direction = if k > 0.0 {
            Some(incident * eta + normal * (eta * cos_theta - k.sqrt()))
        } else {
            None
        };

        let mut result = color * (1.0 - reflection_weight - transmission_weight);

        if reflection_weight > 0.0 {
            let reflected_origin = intersect.point + normal * REFRACTION_BIAS;
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
        final_color = result;
    }

    // --- FOG (Bruma AtmosfÃ©rica) ---
    // Only apply fog on the primary ray (depth == 0) to avoid double fogging through glass
    if depth == 0 {
        let is_secret_room = scene.is_secret_room_object(ObjectId(intersect.object_index));

        // Disable fog completely for the secret room as requested.
        if !is_secret_room {
            let dist = intersect.distance;
            let wave = (intersect.point.x * 2.0 + scene.time).sin()
                * (intersect.point.z * 1.5 - scene.time * 0.5).cos();

            let base_density = match scene.day_phase {
                DayPhase::Dawn => 0.005,
                DayPhase::Day => 0.002,
                DayPhase::Sunset => 0.008,
                DayPhase::Night => 0.015,
            };

            let density = base_density + wave * 0.002;

            let fog_height_factor = (1.0 - (intersect.point.y) / 3.0).clamp(0.0, 1.0);
            let fog_factor = 1.0 - (-(density * fog_height_factor * dist)).exp();

            let fog_color = match scene.day_phase {
                DayPhase::Dawn => Color::new(200, 200, 210),
                DayPhase::Day => Color::new(230, 240, 245),
                DayPhase::Sunset => Color::new(250, 160, 110),
                DayPhase::Night => Color::new(30, 40, 50),
            };

            let t = fog_factor.clamp(0.0, 0.7);
            final_color = final_color * (1.0 - t) + fog_color * t;
        }
    }

    final_color
}

fn environment_color(scene: &Scene, ray_origin: &Vec3, ray_direction: &Vec3) -> Color {
    if scene.game_state.secret_room_open {
        Color::new(0, 0, 0)
    } else {
        scene.skybox.sample(ray_origin, ray_direction)
    }
}

use rayon::prelude::*;

pub fn render(framebuffer: &mut Framebuffer, scene: &Scene, render_mode: u8, scale_factor: u32) {
    let width = framebuffer.width;
    let height = framebuffer.height;
    let camera_eye = scene.camera.eye;

    if scale_factor <= 1 {
        framebuffer
            .buffer
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                for (x, pixel) in row.iter_mut().enumerate() {
                    let ray_direction = scene
                        .camera
                        .ray_for_pixel(x as f32, y as f32, width, height, FOV);
                    *pixel = cast_ray(&camera_eye, &ray_direction, scene, 0, render_mode).to_hex();
                }
            });
    } else {
        let scaled_width = width / scale_factor as usize;
        let scaled_height = height / scale_factor as usize;
        let mut scaled_buffer = vec![0; scaled_width * scaled_height];

        scaled_buffer
            .par_chunks_mut(scaled_width)
            .enumerate()
            .for_each(|(sy, row)| {
                for (sx, pixel) in row.iter_mut().enumerate() {
                    let x = sx * scale_factor as usize;
                    let y = sy * scale_factor as usize;
                    let ray_direction = scene
                        .camera
                        .ray_for_pixel(x as f32, y as f32, width, height, FOV);
                    *pixel = cast_ray(&camera_eye, &ray_direction, scene, 0, render_mode).to_hex();
                }
            });

        framebuffer
            .buffer
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                let sy = (y / scale_factor as usize).min(scaled_height - 1);
                for (x, pixel) in row.iter_mut().enumerate() {
                    let sx = (x / scale_factor as usize).min(scaled_width - 1);
                    *pixel = scaled_buffer[sy * scaled_width + sx];
                }
            });
    }
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
        build_torii_gate(Vec3::new(0.0, 0.0, 2.6), 1.2, &materials),
    );
    append_asset(
        &mut objects,
        build_tsukubai(Vec3::new(-2.2, 0.0, 1.2), 0.8, &materials),
    );
    append_asset(
        &mut objects,
        build_tsukubai(Vec3::new(2.4, 0.0, -0.5), 0.7, &materials),
    );
    append_asset(
        &mut objects,
        build_back_garden_plants(Vec3::zeros(), 1.0, &materials),
    );
    append_asset(
        &mut objects,
        build_japanese_ruins(Vec3::zeros(), 1.0, &materials),
    );

    // ================= CAFÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â° =================
    let cafe_pos = Vec3::new(0.5, 0.0, -1.0);
    append_asset(&mut objects, build_japanese_cafe(cafe_pos, 1.0, &materials));
    append_asset(
        &mut objects,
        build_cafe_patio(Vec3::new(2.7, 0.0, 0.0), 1.0, &materials),
    );

    // Back Door framed, but NOT completely hidden
    let door_ids = append_asset(&mut objects, build_back_door(cafe_pos, 1.0, &materials));

    // ================= VEGETATION & GARDEN =================
    // Rock Garden
    append_asset(
        &mut objects,
        build_rock_garden(Vec3::new(-1.2, 0.1, 2.0), 1.2, &materials),
    );

    // Sakuras (Groups)
    append_asset(
        &mut objects,
        build_sakura_tree_variant(Vec3::new(-2.8, 0.1, -1.5), 1.3, &materials, 0),
    );
    append_asset(
        &mut objects,
        build_sakura_tree_variant(Vec3::new(3.5, 0.1, -1.2), 0.9, &materials, 1),
    );
    // Extra sakura
    append_asset(
        &mut objects,
        build_sakura_tree_variant(Vec3::new(2.0, 0.1, 2.4), 1.1, &materials, 2),
    );

    // Bamboo Groups
    append_asset(
        &mut objects,
        build_bamboo_cluster(Vec3::new(-3.2, 0.1, 2.0), 1.1, &materials),
    ); // Front Left
    append_asset(
        &mut objects,
        build_bamboo_cluster(Vec3::new(-3.5, 0.1, -2.5), 1.0, &materials),
    ); // Back Left
    append_asset(
        &mut objects,
        build_bamboo_cluster(Vec3::new(3.8, 0.1, 2.0), 0.9, &materials),
    ); // Front Right
    append_asset(
        &mut objects,
        build_bamboo_cluster(Vec3::new(2.5, 0.1, -2.8), 1.0, &materials),
    ); // Back Right
    // Extra bamboo
    append_asset(
        &mut objects,
        build_bamboo_cluster(Vec3::new(-1.0, 0.1, 2.4), 1.0, &materials),
    );

    append_asset(
        &mut objects,
        build_more_garden_flowers(Vec3::new(0.0, 0.0, 0.0), 1.0, &materials),
    );

    // ================= LANTERNS & DETAILS =================
    let mut toro_materials = materials.stone.clone();
    toro_materials.emission = Color::new(0, 0, 0);

    append_asset(
        &mut objects,
        build_toro_lantern(Vec3::new(-1.8, 0.1, 1.0), 0.7, &materials),
    );
    append_asset(
        &mut objects,
        build_toro_lantern(Vec3::new(3.2, 0.1, 0.5), 0.6, &materials),
    );

    // Small stones / bushes on the grass
    append_asset(
        &mut objects,
        build_rock_garden(Vec3::new(-3.0, 0.1, 1.0), 0.6, &materials),
    );
    append_asset(
        &mut objects,
        build_rock_garden(Vec3::new(3.5, 0.1, -0.5), 0.5, &materials),
    );

    // Bushes removed as requested, using real flowers instead (build_more_garden_flowers handles this)

    // ================= LIGHTS =================
    let mut lights = Vec::new();
    lights.push(Light {
        position: Vec3::new(-10.0, 15.0, 10.0),
        color: Color::new(255, 240, 220),
        intensity: 1.2,
    });

    lights.push(Light::new(
        Vec3::new(-1.8, 0.72, 1.18),
        Color::new(255, 190, 105),
        0.0,
    ));
    lights.push(Light::new(
        Vec3::new(3.2, 0.65, 0.66),
        Color::new(255, 190, 105),
        0.0,
    ));

    // Load Skyboxes
    let mut skybox = Skybox::new(Color::new(135, 206, 235));
    let _ = skybox.load_phase_layers(
        DayPhase::Dawn,
        &[
            "assets/sky_dawn/1.png",
            "assets/sky_dawn/2.png",
            "assets/sky_dawn/3.png",
            "assets/sky_dawn/4.png",
        ],
    );
    let _ = skybox.load_phase_layers(
        DayPhase::Day,
        &[
            "assets/sky_day/1.png",
            "assets/sky_day/2.png",
            "assets/sky_day/3.png",
            "assets/sky_day/4.png",
        ],
    );
    let _ = skybox.load_phase_layers(
        DayPhase::Sunset,
        &[
            "assets/sky_sunset/1.png",
            "assets/sky_sunset/2.png",
            "assets/sky_sunset/3.png",
            "assets/sky_sunset/4.png",
        ],
    );
    let _ = skybox.load_phase_layers(
        DayPhase::Night,
        &[
            "assets/sky_night/1.png",
            "assets/sky_night/2.png",
            "assets/sky_night/3.png",
            "assets/sky_night/4.png",
        ],
    );

    let mut scene = Scene::new(objects, lights, camera, skybox);

    let anomaly = add_primitive(
        &mut scene.objects,
        Box::new(Cube),
        Vec3::new(0.5, 1.20, -1.905),
        Vec3::zeros(),
        Vec3::new(0.23, 0.035, 0.02),
        materials
            .metal
            .clone()
            .with_emission(Color::new(230, 74, 66), 1.5),
    );
    scene.appears_on_day.insert(ObjectId(anomaly), 4);

    // Register objects for days
    let day2_ids = append_asset(
        &mut scene.objects,
        build_day2_flowers(Vec3::zeros(), 1.0, &materials),
    );
    for id in day2_ids {
        scene.appears_on_day.insert(ObjectId(id), 2);
    }

    let day3_ids = append_asset(
        &mut scene.objects,
        build_day3_decorations(Vec3::zeros(), 1.0, &materials),
    );
    for id in day3_ids {
        scene.appears_on_day.insert(ObjectId(id), 3);
    }

    let day5_ids = append_asset(
        &mut scene.objects,
        build_day5_higanbana(Vec3::zeros(), 1.0, &materials),
    );
    for id in day5_ids {
        scene.appears_on_day.insert(ObjectId(id), 5);
        scene.register_interactive(id, InteractiveKind::Clue(0), false);
    }

    let day6_ids = append_asset(
        &mut scene.objects,
        build_day6_clue(Vec3::zeros(), 1.0, &materials),
    );
    for id in day6_ids {
        scene.appears_on_day.insert(ObjectId(id), 6);
        scene.register_interactive(id, InteractiveKind::Clue(1), false);
    }

    let day7_ids = append_asset(
        &mut scene.objects,
        build_day7_clue(Vec3::zeros(), 1.0, &materials),
    );
    for id in day7_ids {
        scene.appears_on_day.insert(ObjectId(id), 7);
        scene.register_interactive(id, InteractiveKind::Clue(2), false);
    }

    for id in door_ids {
        scene.register_door_part(id);
    }

    let (room_base, room_fpp) = build_secret_room_interior(Vec3::new(2.5, -2.0, -2.5), 1.0, &materials);
    let room_ids = append_asset(&mut scene.objects, room_base);
    for id in room_ids {
        scene.register_secret_room_object(id);
    }
    let room_fpp_ids = append_asset(&mut scene.objects, room_fpp);
    for id in room_fpp_ids {
        scene.secret_room_fpp_walls.push(crate::core::interaction::ObjectId(id));
        scene.register_secret_room_object(id); // Treat as secret room object to hide when closed
    }
    // 4 Lamps
    for (x, z) in [
        (-1.6, 1.4),  // Back left
        (1.6, 1.4),   // Back right
        (-1.6, -1.5), // Front left
        (1.6, -1.5),  // Front right
    ] {
        scene.secret_room_lights.push(scene.lights.len());
        scene.lights.push(Light::new(
            Vec3::new(2.5 + x, -1.4, -2.5 + z), // Lamps near the ceiling or floor? Let's put them high
            Color::new(255, 180, 92), // Warm Amber
            0.0,
        ));
    }

    // ================= CAFÃ‰ INTERIOR =================
    let (cafe_base, cafe_fpp) = build_cafe_interior(cafe_pos, 1.0, &materials);
    let cafe_interior_ids = append_asset(&mut scene.objects, cafe_base);
    let cafe_fpp_ids = append_asset(&mut scene.objects, cafe_fpp);
    for id in cafe_fpp_ids {
        scene.cafe_fpp_walls.push(crate::core::interaction::ObjectId(id));
        scene.hide_object(crate::core::interaction::ObjectId(id));
    }
    // The last object in the interior is the exit mat â€” register it as CafeEntrance
    if let Some(&exit_mat_id) = cafe_interior_ids.last() {
        scene.register_interactive(exit_mat_id, InteractiveKind::CafeEntrance, true);
    }

    // Entrance mat (welcome mat) on the front step â€” clickable to enter
    let entrance_mat_id = add_primitive(
        &mut scene.objects,
        Box::new(Cube),
        Vec3::new(0.5, 0.22, -0.15),
        Vec3::zeros(),
        Vec3::new(0.8, 0.02, 0.4),
        {
            let mut mat = materials.wood.clone();
            mat.diffuse = Color::new(180, 155, 120);
            mat
        },
    );
    scene.register_interactive(entrance_mat_id, InteractiveKind::CafeEntrance, true);

    // Interior cafÃ© light (warm lantern)
    scene.lights.push(Light::new(
        Vec3::new(0.5, 0.95, -0.9),
        Color::new(255, 200, 130),
        0.0,
    ));

    scene.day_count = 1;
    scene.set_day_phase(DayPhase::Day);
    scene.refresh_visibility();

    scene
}

fn secret_room_camera() -> Camera {
    Camera::new(
        Vec3::new(2.5, -0.6, -7.5), // Look from further back into the room
        Vec3::new(2.5, -1.6, -2.5), // Look directly at the room center
        Vec3::y(),
    )
}

fn secret_room_fpp_camera() -> Camera {
    // Center of the room is X=2.5, Z=-2.5
    // Z range of room is -4.3 to -0.7
    // Let's place the camera near the front wall looking towards the back (the table and emblem)
    Camera::new(
        Vec3::new(2.5, -1.3, -3.8), // Standing near the front door, looking at the Yakuza emblem
        Vec3::new(2.5, -1.5, -0.7), // Looking at the table/back wall
        Vec3::y(),
    )
}

fn cafe_camera() -> Camera {
    Camera::new(
        Vec3::new(-3.5, 1.2, 3.5),
        Vec3::new(0.0, 0.6, 0.0),
        Vec3::y(),
    )
}

fn cafe_interior_camera() -> Camera {
    // Eye inside the cafÃ© near the front door, looking toward the counter.
    // cafÃ©_pos = (0.5, 0, -1.0), floor Yâ‰ˆ0.22, ceiling Yâ‰ˆ1.31
    // Eye at Y=0.85 (standing height), near front Zâ‰ˆ-0.3 (just inside)
    Camera::new(
        Vec3::new(0.5, 0.85, -0.35), // Inside, near front door
        Vec3::new(0.5, 0.60, -1.20), // Looking toward back wall / counter
        Vec3::y(),
    )
}

fn create_camera() -> Camera {
    Camera::new(
        Vec3::new(4.0, 5.8, 8.5),
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

pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
    pub seed: f32,
}

fn draw_particles(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    particles: &[Particle],
    camera: &Camera,
    scene: &Scene,
) {
    let aspect = width as f32 / height as f32;
    let fov_factor = (FOV / 2.0).tan();
    let forward = (camera.center - camera.eye).normalize();
    let right = forward.cross(&camera.up).normalize();
    let up = right.cross(&forward).normalize();

    for p in particles {
        let dir = p.pos - camera.eye;
        let dist = dot(&dir, &forward);
        if dist < 0.2 {
            continue;
        } // Too close or behind camera

        // Fast occlusion check with bounding ray
        let is_occluded = {
            let mut occluded = false;
            let dir_norm = dir.normalize();
            for (idx, obj) in scene.objects.iter().enumerate() {
                if !scene.is_object_visible(ObjectId(idx)) {
                    continue;
                }
                if let Some(hit) = obj.ray_intersect(&camera.eye, &dir_norm) {
                    // Check if it's opaque and closer than the particle
                    if hit.distance < dir.magnitude() - 0.1 && hit.material.transparency < 0.5 {
                        occluded = true;
                        break;
                    }
                }
            }
            occluded
        };

        if is_occluded {
            continue;
        }

        let x = dot(&dir, &right) / dist;
        let y = dot(&dir, &up) / dist;

        let screen_x = ((x / (fov_factor * aspect) + 1.0) * 0.5 * width as f32) as i32;
        let screen_y = ((1.0 - (y / fov_factor + 1.0) * 0.5) * height as f32) as i32;

        let r = (p.size / dist * height as f32 * 0.1) as i32;
        if r < 1 {
            continue;
        }

        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = dx * dx + dy * dy;
                if d2 > r * r {
                    continue;
                }

                let px = screen_x + dx;
                let py = screen_y + dy;
                if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                    let intensity =
                        (1.0 - (d2 as f32).sqrt() / r as f32) * (p.life / p.max_life).min(1.0);
                    let idx = (py as usize) * width + (px as usize);
                    let old = Color::from_hex(buffer[idx]);
                    let t = intensity.clamp(0.0, 1.0) * 0.8;
                    buffer[idx] = (old * (1.0 - t) + p.color * t).to_hex();
                }
            }
        }
    }
}

fn main() {
    let test_mode = std::env::args().nth(1);
    let requested_day = test_mode
        .as_deref()
        .and_then(|arg| arg.strip_prefix("--render-day"))
        .and_then(|day| day.parse::<u32>().ok())
        .filter(|day| (1..=7).contains(day));
    let legacy_render = matches!(
        test_mode.as_deref(),
        Some("--render-day1-flat" | "--render-day1-textured" | "--render-day1-rear")
    );
    if requested_day.is_some() || legacy_render {
        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
        let rear_view = test_mode.as_deref() == Some("--render-day1-rear");
        let camera = if rear_view {
            Camera::new(
                Vec3::new(-3.2, 5.8, -8.5),
                Vec3::new(0.0, 0.35, 0.0),
                Vec3::y(),
            )
        } else {
            create_camera()
        };
        let mut scene = create_scene(camera);
        let day = requested_day.unwrap_or(1);
        scene.set_story_day(day);
        let flat = test_mode.as_deref() == Some("--render-day1-flat");
        let path = if requested_day.is_some() {
            format!("target/day{day}.png")
        } else if flat {
            "target/day1-flat.png".into()
        } else if rear_view {
            "target/day1-rear.png".into()
        } else {
            "target/day1-textured.png".into()
        };
        render(&mut framebuffer, &scene, if flat { 2 } else { 0 }, 1);
        crate::core::hud::draw(&mut framebuffer, &scene);
        save_test_render(&framebuffer, &path).expect("Could not save day render");
        println!(
            "Saved day {} ({:?}) to {}",
            scene.day_count, scene.day_phase, path
        );
        return;
    }
    if test_mode.as_deref() == Some("--render-cafe") {
        let mut scene = create_scene(cafe_camera());
        scene.set_story_day(1);
        scene.set_day_phase(crate::core::scene::DayPhase::Night);
        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
        render(&mut framebuffer, &scene, 0, 1);
        match save_test_render(&framebuffer, "target/cafe-test.png") {
            Ok(()) => println!("Saved day 1 cafe view to target/cafe-test.png"),
            Err(error) => eprintln!("Could not save renderer test: {}", error),
        }
        return;
    }

    if test_mode.as_deref() == Some("--render-secret-room") {
        let mut scene = create_scene(secret_room_camera());
        scene.set_story_day(7);
        for index in 0..scene.objects.len() {
            scene.discover_clue(ObjectId(index));
        }
        assert!(scene.toggle_secret_room());
        scene.skybox = crate::core::scene::Skybox::new(Color::new(0, 0, 0));

        let mut void_mat = Material::new(Color::new(0, 0, 0));
        void_mat.diffuse = Color::new(0, 0, 0);
        void_mat.texture = None;
        scene
            .objects
            .push(Box::new(crate::core::object::Object::new(
                Box::new(crate::shapes::cube::Cube),
                Transform::new(
                    Vec3::new(2.5, 0.0, 0.0),
                    Vec3::zeros(),
                    Vec3::new(30.0, 30.0, 30.0),
                ),
                void_mat,
            )));

        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
        render(&mut framebuffer, &scene, 0, 1);
        crate::core::hud::draw(&mut framebuffer, &scene);
        save_test_render(&framebuffer, "target/secret-room.png").expect("Could not save room");
        println!("Saved unlocked room to target/secret-room.png");
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
        render(&mut framebuffer, &scene, test_render_mode, 1);
        match save_test_render(&framebuffer, output_path) {
            Ok(()) => println!("Saved renderer test to {output_path}"),
            Err(error) => eprintln!("Could not save renderer test: {error}"),
        }
        return;
    }

    let frame_delay = Duration::from_millis(16);
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window =
        Window::new("La Puerta Trasera", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut scene = create_scene(create_camera());

    let mut camera_moved = true;
    let mut render_mode = 0;
    let mut was_moving = false;
    let mut last_mouse_position: Option<(f32, f32)> = None;

    let mut was_mouse_down = false;
    let mut pointer = PointerGesture::default();
    let mut fade: Option<(Vec<u32>, Instant)> = None;
    let mut display = vec![0u32; WIDTH * HEIGHT];

    // Helper for title updates
    let get_title = |scene: &Scene| -> String {
        let day = scene.day_count;
        let phase = match scene.day_phase {
            DayPhase::Dawn => "Amanecer",
            DayPhase::Day => "DÃ­a",
            DayPhase::Sunset => "Atardecer",
            DayPhase::Night => "Noche",
        };
        let clues = scene.game_state.clues_count();
        let clue_str = format!(" | PISTAS {}/3", clues);
        let location = if scene.game_state.secret_room_open {
            " | SOTANO SECRETO"
        } else if scene.game_state.inside_cafe {
            " | INTERIOR DEL CAFE"
        } else {
            ""
        };
        format!(
            "La Puerta Trasera - DIA {} ({}){}{}",
            day, phase, clue_str, location
        )
    };

    window.set_title(&get_title(&scene));

    let menu_img = image::open("assets/ui/main_menu.png").ok();

    let mut app_mode = AppMode::MainMenu;
    let mut menu_selection = 0;
    let mut cinematic_state: Option<CinematicState> = None;

    // Render initial background for menu
    render(&mut framebuffer, &scene, 0, 1);
    let menu_background = framebuffer.buffer.clone();

    let mut last_frame_time = Instant::now();

    let mut particles: Vec<Particle> = Vec::new();

    while window.is_open() {
        let now = Instant::now();
        let delta_time = now.duration_since(last_frame_time).as_secs_f32().min(0.1);
        last_frame_time = now;
        scene.time += delta_time;

        // -- ACTUALIZACIÃ“N DE PARTÃCULAS --
        let is_night = scene.day_phase == DayPhase::Night || scene.day_phase == DayPhase::Sunset;
        let max_particles = if scene.game_state.secret_room_open {
            0 // No particles inside the secret room
        } else if is_night {
            25
        } else {
            10
        };

        for p in &mut particles {
            p.life -= delta_time;
            p.pos += p.vel * delta_time;
            p.pos.y += (scene.time * 1.5 + p.seed).sin() * 0.1 * delta_time;
            p.pos.x += (scene.time * 0.8 + p.seed * 2.0).cos() * 0.05 * delta_time;
        }
        particles.retain(|p| p.life > 0.0);

        while particles.len() < max_particles {
            let seed = scene.time + particles.len() as f32;
            let (px, py, pz) = if scene.game_state.secret_room_open {
                // Spawn ONLY in secret room
                (
                    2.5 + (seed * 13.0).sin() * 1.8,
                    -1.6 + (seed * 7.0).cos() * 0.4,
                    -2.5 + (seed * 11.0).sin() * 1.5,
                )
            } else if scene.game_state.inside_cafe {
                // Spawn ONLY inside the cafe
                (
                    0.5 + (seed * 13.0).sin() * 1.2,
                    0.8 + (seed * 7.0).cos() * 0.4,
                    -1.0 + (seed * 11.0).sin() * 0.8,
                )
            } else {
                // Spawn in garden
                (
                    (seed * 17.0).cos() * 3.0,
                    0.2 + (seed * 5.0).sin() * 0.5,
                    -1.0 + (seed * 19.0).cos() * 2.0,
                )
            };

            let color = if is_night {
                Color::new(255, 220, 150)
            } else {
                Color::new(255, 255, 255)
            };

            particles.push(Particle {
                pos: Vec3::new(px, py, pz),
                vel: Vec3::new(0.0, 0.05, 0.0),
                life: 3.0 + (seed * 3.0).sin().abs() * 3.0,
                max_life: 6.0,
                color,
                size: 0.15 + (seed * 7.0).cos().abs() * 0.1,
                seed,
            });
        }

        let mut temporal_change = false;

        if window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No) {
            match app_mode {
                AppMode::MainMenu => break,
                AppMode::Exploration => {
                    app_mode = AppMode::MainMenu;
                    scene.game_state.inside_cafe = false;
                    particles.clear();
                    camera_moved = true;
                }
                AppMode::Cinematic => {
                    if let Some(ref state) = cinematic_state {
                        scene.camera = state.original_camera.clone();
                    }
                    scene.set_story_day(1);
                    scene.set_day_phase(DayPhase::Dawn);
                    if scene.game_state.secret_room_open {
                        toggle_secret_room_shortcut(&mut scene);
                    }
                    app_mode = AppMode::MainMenu;
                    cinematic_state = None;
                    particles.clear();
                    camera_moved = true;
                }
            }
        }

        if app_mode == AppMode::MainMenu {
            if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)
                || window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No)
                || window.is_key_pressed(Key::NumPad1, minifb::KeyRepeat::No)
            {
                menu_selection = 0;
            }
            if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)
                || window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No)
                || window.is_key_pressed(Key::NumPad2, minifb::KeyRepeat::No)
            {
                menu_selection = 1;
            }
            if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) {
                if menu_selection == 0 {
                    app_mode = AppMode::Exploration;
                    scene.set_story_day(1);
                    scene.set_day_phase(DayPhase::Dawn);
                    if scene.game_state.secret_room_open {
                        toggle_secret_room_shortcut(&mut scene);
                    }
                    camera_moved = true;
                } else {
                    app_mode = AppMode::Cinematic;
                    cinematic_state = Some(CinematicState {
                        start_time: Instant::now(),
                        original_camera: scene.camera.clone(),
                    });
                    scene.set_story_day(1);
                    scene.set_day_phase(DayPhase::Dawn);
                    if scene.game_state.secret_room_open {
                        toggle_secret_room_shortcut(&mut scene);
                    }
                    camera_moved = true;
                }
            }
            // Restore pristine background to erase previous menu frames
            framebuffer.buffer.copy_from_slice(&menu_background);
            // Draw menu overlay over whatever is in the framebuffer
            if let Some(ref img) = menu_img {
                crate::core::hud::draw_image(&mut framebuffer, img, 0, 0);
                let cursor_y = if menu_selection == 0 { 290 } else { 390 };
                crate::core::hud::text(&mut framebuffer, ">>", 100, cursor_y, 3);
                crate::core::hud::text(
                    &mut framebuffer,
                    "ENTER CONFIRMAR   ESC SALIR",
                    200,
                    500,
                    1,
                );
            } else {
                crate::core::hud::text(&mut framebuffer, "LA PUERTA TRASERA", 160, 100, 5);
                crate::core::hud::text(
                    &mut framebuffer,
                    "ç§˜å¯†ã®ä¸ƒæ—¥é–“ (Siete Dias de Secretos)",
                    160,
                    150,
                    2,
                );

                let color1 = if menu_selection == 0 {
                    ">> 1. EXPLORAR EL MUNDO"
                } else {
                    "   1. EXPLORAR EL MUNDO"
                };
                let color2 = if menu_selection == 1 {
                    ">> 2. VIVIR LOS 7 DIAS "
                } else {
                    "   2. VIVIR LOS 7 DIAS "
                };

                crate::core::hud::text(&mut framebuffer, color1, 200, 300, 2);
                crate::core::hud::text(&mut framebuffer, color2, 200, 350, 2);
                crate::core::hud::text(
                    &mut framebuffer,
                    "ENTER CONFIRMAR   ESC SALIR",
                    200,
                    500,
                    1,
                );
            }

            window
                .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
                .unwrap();
            std::thread::sleep(frame_delay);
            continue;
        }

        if app_mode == AppMode::Cinematic {
            if let Some(ref state) = cinematic_state {
                let t = state.start_time.elapsed().as_secs_f32();
                // 7 Acts over 75 seconds. ~10.7 seconds per act.
                // Act 1: Dawn, pan around facade
                // Act 2 (10s): Day 2 Day, pan to garden
                // Act 3 (20s): Day 3 Day, interior
                // Act 4 (30s): Day 4 Sunset, back garden
                // Act 5 (40s): Day 5 Sunset, higanbana
                // Act 6 (50s): Day 6 Night, lantern
                // Act 7 (60s): Day 7 Night, trapdoor opens

                let target_day;
                let target_phase;
                let c_eye;
                let mut c_center = Vec3::new(0.0, 0.5, 0.0);
                let mut open_door = false;

                if t < 10.0 {
                    target_day = 1;
                    target_phase = DayPhase::Dawn;
                    let lerp = t / 10.0;
                    c_eye = Vec3::new(-3.5 + lerp, 1.2, 4.0 - lerp);
                } else if t < 20.0 {
                    target_day = 2;
                    target_phase = DayPhase::Day;
                    let lerp = (t - 10.0) / 10.0;
                    c_eye = Vec3::new(-2.5 + lerp * 5.0, 1.2 + lerp * 0.3, 3.0 - lerp * 1.0);
                    c_center = Vec3::new(0.0 + lerp * 0.5, 0.5, 0.0 - lerp * 1.0);
                } else if t < 30.0 {
                    target_day = 3;
                    target_phase = DayPhase::Day;
                    let lerp = (t - 20.0) / 10.0;
                    c_eye = Vec3::new(0.5, 0.9, -0.3 - lerp * 0.2);
                    c_center = Vec3::new(0.5, 0.5, -1.6);
                } else if t < 40.0 {
                    target_day = 4;
                    target_phase = DayPhase::Sunset;
                    let lerp = (t - 30.0) / 10.0;
                    c_eye = Vec3::new(2.5, 1.5, 2.0 - lerp * 4.0);
                    c_center = Vec3::new(0.5, 0.5, -1.0);
                } else if t < 50.0 {
                    target_day = 5;
                    target_phase = DayPhase::Sunset;
                    let lerp = (t - 40.0) / 10.0;
                    c_eye = Vec3::new(2.0, 0.6, -1.8);
                    c_center = Vec3::new(2.3 - lerp * 0.2, 0.1, -2.1 + lerp * 0.2);
                } else if t < 60.0 {
                    target_day = 6;
                    target_phase = DayPhase::Night;
                    let lerp = (t - 50.0) / 10.0;
                    c_eye = Vec3::new(-1.0 - lerp * 0.5, 1.0, 0.0 + lerp * 0.5);
                    c_center = Vec3::new(-1.5, 0.5, 1.2);
                } else if t < 70.0 {
                    target_day = 7;
                    target_phase = DayPhase::Night;
                    let lerp = (t - 60.0) / 10.0;
                    c_eye = Vec3::new(1.0, 2.0 - lerp * 0.5, 1.5 - lerp * 0.5);
                    c_center = Vec3::new(1.0, 0.0, -1.0);
                    open_door = true;
                } else if t < 80.0 {
                    target_day = 7;
                    target_phase = DayPhase::Night;
                    c_eye = Vec3::new(0.0, -4.0, 0.0);
                    c_center = Vec3::new(0.0, -4.5, -2.0);
                    open_door = true;
                } else {
                    app_mode = AppMode::MainMenu;
                    scene.camera = state.original_camera.clone();
                    scene.set_story_day(1);
                    scene.set_day_phase(DayPhase::Dawn);
                    if scene.game_state.secret_room_open {
                        toggle_secret_room_shortcut(&mut scene);
                    }
                    cinematic_state = None;
                    particles.clear();
                    camera_moved = true;
                    continue;
                }

                if scene.day_count != target_day {
                    scene.set_story_day(target_day);
                    temporal_change = true;
                }
                if scene.day_phase != target_phase {
                    scene.set_day_phase(target_phase);
                    scene.update_transition(1.0);
                    temporal_change = true;
                }
                if scene.game_state.secret_room_open != open_door {
                    toggle_secret_room_shortcut(&mut scene);
                    temporal_change = true;
                }

                scene.camera = Camera::new(c_eye, c_center, Vec3::new(0.0, 1.0, 0.0));
                camera_moved = true;
            }
        }

        if app_mode == AppMode::Exploration {
            // Temporary render mode bindings
            if window.is_key_pressed(Key::Key8, minifb::KeyRepeat::No) {
                render_mode = 0;
                camera_moved = true;
            }
            if window.is_key_pressed(Key::Key9, minifb::KeyRepeat::No) {
                render_mode = 1;
                camera_moved = true;
            }
            if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) {
                render_mode = 2;
                camera_moved = true;
            }

            // Time of Day
            if window.is_key_pressed(Key::F1, minifb::KeyRepeat::No) {
                scene.set_day_phase(DayPhase::Dawn);
                temporal_change = true;
                scene.update_transition(1.0);
                camera_moved = true;
                window.set_title(&get_title(&scene));
            }
            if window.is_key_pressed(Key::F2, minifb::KeyRepeat::No) {
                scene.set_day_phase(DayPhase::Day);
                temporal_change = true;
                scene.update_transition(1.0);
                camera_moved = true;
                window.set_title(&get_title(&scene));
            }
            if window.is_key_pressed(Key::F3, minifb::KeyRepeat::No) {
                scene.set_day_phase(DayPhase::Sunset);
                temporal_change = true;
                scene.update_transition(1.0);
                camera_moved = true;
                window.set_title(&get_title(&scene));
            }
            if window.is_key_pressed(Key::F4, minifb::KeyRepeat::No) {
                scene.set_day_phase(DayPhase::Night);
                temporal_change = true;
                scene.update_transition(1.0);
                camera_moved = true;
                window.set_title(&get_title(&scene));
            }

            // Day Progression
            for (i, key) in [
                Key::F5,
                Key::F6,
                Key::F7,
                Key::F8,
                Key::F9,
                Key::F10,
                Key::F11,
            ]
            .iter()
            .enumerate()
            {
                if window.is_key_pressed(*key, minifb::KeyRepeat::No) {
                    scene.set_story_day((i + 1) as u32);
                    temporal_change = true;
                    camera_moved = true;
                    window.set_title(&get_title(&scene));
                }
            }

            if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) {
                if toggle_secret_room_shortcut(&mut scene) {
                    scene.game_state.inside_cafe = false;
                    particles.clear();
                    
                    if scene.game_state.secret_room_open {
                        scene.game_state.inside_secret_room_fpp = true;
                        for id in scene.secret_room_fpp_walls.clone() {
                            scene.show_object(id);
                        }
                        scene.camera = secret_room_fpp_camera();
                    } else {
                        scene.game_state.inside_secret_room_fpp = false;
                        for id in scene.secret_room_fpp_walls.clone() {
                            scene.hide_object(id);
                        }
                        scene.camera = create_camera();
                    }
                    
                    temporal_change = true;
                    camera_moved = true;
                    window.set_title(&get_title(&scene));
                    println!(
                        "Acceso directo F12: habitación secreta {}.",
                        if scene.game_state.secret_room_open {
                            "abierta (en Primera Persona)"
                        } else {
                            "cerrada"
                        }
                    );
                }
            }

            if window.is_key_pressed(Key::T, minifb::KeyRepeat::No) {
                if scene.game_state.secret_room_open {
                    scene.game_state.inside_secret_room_fpp = !scene.game_state.inside_secret_room_fpp;
                    
                    if scene.game_state.inside_secret_room_fpp {
                        for id in scene.secret_room_fpp_walls.clone() {
                            scene.show_object(id);
                        }
                    } else {
                        for id in scene.secret_room_fpp_walls.clone() {
                            scene.hide_object(id);
                        }
                    }
                    
                    scene.camera = if scene.game_state.inside_secret_room_fpp {
                        secret_room_fpp_camera()
                    } else {
                        secret_room_camera()
                    };
                    particles.clear();
                    temporal_change = true;
                    camera_moved = true;
                    window.set_title(&get_title(&scene));
                }
            }

            if window.is_key_pressed(Key::C, minifb::KeyRepeat::No) {
                scene.toggle_cafe_interior();
                scene.camera = if scene.game_state.inside_cafe {
                    cafe_interior_camera()
                } else {
                    create_camera()
                };
                particles.clear();
                temporal_change = true;
                camera_moved = true;
                window.set_title(&get_title(&scene));
                println!(
                    "CafÃ© {}",
                    if scene.game_state.inside_cafe {
                        "entrando"
                    } else {
                        "saliendo"
                    }
                );
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

            if mouse_down && !was_mouse_down {
                pointer.press(mouse_position);
            }
            if mouse_down {
                pointer.update(mouse_position);
            }
            if !mouse_down && was_mouse_down {
                if let Some((x, y)) = pointer.release(mouse_position) {
                    if let Some(hit) = pick(
                        &scene,
                        &scene.camera,
                        x,
                        y,
                        framebuffer.width,
                        framebuffer.height,
                        FOV,
                    ) {
                        let id = ObjectId(hit.object_index);
                        if let Some(kind) = scene.interaction_for(id) {
                            match kind {
                                InteractiveKind::Clue(i) => {
                                    if scene.discover_clue(id) {
                                        camera_moved = true;
                                        window.set_title(&get_title(&scene));
                                        println!("Pista {} encontrada!", i + 1);
                                    }
                                }
                                InteractiveKind::BackDoor => {
                                    if scene.toggle_secret_room() {
                                        scene.game_state.inside_cafe = false;
                                        particles.clear();
                                        scene.camera = if scene.game_state.secret_room_open {
                                            secret_room_camera()
                                        } else {
                                            create_camera()
                                        };
                                        camera_moved = true;
                                        window.set_title(&get_title(&scene));
                                        println!(
                                            "Puerta {}",
                                            if scene.game_state.secret_room_open {
                                                "abierta"
                                            } else {
                                                "cerrada"
                                            }
                                        );
                                    } else {
                                        println!(
                                            "La puerta necesita las tres pistas ({}/3).",
                                            scene.game_state.clues_count()
                                        );
                                    }
                                }
                                InteractiveKind::CafeEntrance => {
                                    scene.toggle_cafe_interior();
                                    scene.camera = if scene.game_state.inside_cafe {
                                        cafe_interior_camera()
                                    } else {
                                        create_camera()
                                    };
                                    camera_moved = true;
                                    window.set_title(&get_title(&scene));
                                    println!(
                                        "CafÃ© {}",
                                        if scene.game_state.inside_cafe {
                                            "entrando"
                                        } else {
                                            "saliendo"
                                        }
                                    );
                                }
                                InteractiveKind::Movable => {}
                            }
                        }
                    }
                }
            }
            was_mouse_down = mouse_down;

            if mouse_down {
                if let (Some((x, y)), Some((last_x, last_y))) =
                    (mouse_position, last_mouse_position)
                {
                    let drag_sensitivity = 0.006;
                    let delta_yaw = -(x - last_x) * drag_sensitivity;
                    let delta_pitch = (y - last_y) * drag_sensitivity;
                    if pointer.dragging && (delta_yaw != 0.0 || delta_pitch != 0.0) {
                        scene.camera.orbit(delta_yaw, delta_pitch);
                        camera_moved = true;
                    }
                }
                last_mouse_position = mouse_position;
            } else {
                last_mouse_position = None;
            }
            was_moving = (mouse_down && pointer.dragging)
                || orbit.iter().any(|(k, _, _)| window.is_key_down(*k));

            if let Some((_, scroll_delta)) = window.get_scroll_wheel() {
                if scroll_delta != 0.0 {
                    scene.camera.zoom(scroll_delta);
                    camera_moved = true;
                }
            }
        } // End Exploration block

        let is_moving = was_moving;
        if camera_moved {
            let previous = temporal_change.then(|| display.clone());
            render(
                &mut framebuffer,
                &scene,
                render_mode,
                if is_moving { 4 } else { 1 },
            );
            crate::core::hud::draw(&mut framebuffer, &scene);
            if !is_moving {
                camera_moved = false;
            }
            fade = previous.map(|pixels| (pixels, Instant::now()));
        }

        if was_moving && !is_moving {
            render(&mut framebuffer, &scene, render_mode, 1);
            crate::core::hud::draw(&mut framebuffer, &scene);
        }
        was_moving = is_moving;

        if let Some((previous, start)) = &fade {
            let progress = (start.elapsed().as_secs_f32() / 0.55).clamp(0.0, 1.0);
            crate::core::framebuffer::crossfade(
                previous,
                &framebuffer.buffer,
                &mut display,
                progress,
            );
            if progress >= 1.0 {
                fade = None;
            }
        } else {
            display.copy_from_slice(&framebuffer.buffer);
        }
        if app_mode != AppMode::MainMenu {
            draw_particles(
                &mut display,
                WIDTH,
                HEIGHT,
                &particles,
                &scene.camera,
                &scene,
            );
        }
        window.update_with_buffer(&display, WIDTH, HEIGHT).unwrap();
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

#[cfg(test)]
mod secret_room_background_tests {
    use super::cast_ray;
    use crate::core::camera::Camera;
    use crate::core::scene::{Scene, Skybox};
    use crate::materials::color::Color;
    use nalgebra_glm::Vec3;

    #[test]
    fn secret_room_missed_rays_use_black_background_at_all_depths() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::zeros(),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let mut scene = Scene::new(
            Vec::new(),
            Vec::new(),
            camera,
            Skybox::new(Color::new(100, 130, 180)),
        );
        let origin = Vec3::new(0.0, 0.0, 2.0);
        let direction = Vec3::new(0.0, 1.0, 0.0);

        assert_ne!(cast_ray(&origin, &direction, &scene, 0, 0).to_hex(), 0);
        scene.game_state.secret_room_open = true;
        assert_eq!(cast_ray(&origin, &direction, &scene, 0, 0).to_hex(), 0);
        assert_eq!(cast_ray(&origin, &direction, &scene, 4, 0).to_hex(), 0);
    }
}

#[cfg(test)]
mod story_tests {
    use super::*;

    fn interactive_id(scene: &Scene, kind: InteractiveKind) -> ObjectId {
        (0..scene.objects.len())
            .map(ObjectId)
            .find(|id| scene.interaction_for(*id) == Some(kind))
            .unwrap()
    }

    fn pick_near(scene: &Scene, target: Vec3, expected: ObjectId) {
        let forward = (scene.camera.center - scene.camera.eye).normalize();
        let right = forward.cross(&scene.camera.up).normalize();
        let up = right.cross(&forward).normalize();
        let delta = target - scene.camera.eye;
        let depth = dot(&delta, &forward);
        let half_height = (FOV / 2.0).tan() * depth;
        let x = WIDTH as f32 / 2.0 + dot(&delta, &right) * HEIGHT as f32 / (2.0 * half_height);
        let y = HEIGHT as f32 / 2.0 - dot(&delta, &up) * HEIGHT as f32 / (2.0 * half_height);
        let mut found = false;
        for dy in -10..=10 {
            for dx in -10..=10 {
                if let Some(hit) = pick(
                    scene,
                    &scene.camera,
                    x + dx as f32,
                    y + dy as f32,
                    WIDTH,
                    HEIGHT,
                    FOV,
                ) {
                    if hit.object_index == expected.0 {
                        found = true;
                        break;
                    } else if hit.object_index != expected.0 {
                        println!("Ray hit {}, expected {}", hit.object_index, expected.0);
                    }
                }
            }
            if found {
                break;
            }
        }
        assert!(
            found,
            "Object {:?} cannot be picked near ({x}, {y})",
            expected
        );
    }

    #[test]
    fn complete_story_can_be_picked_and_room_is_hidden_until_unlocked() {
        let mut scene = create_scene(create_camera());
        // --- CAFE THRESHOLD (ENTRY/EXIT) ---
        let mat_material = Material::new(Color::new(60, 60, 70));
        let welcome_mat = crate::core::object::Object::new(
            Box::new(crate::shapes::cube::Cube),
            crate::core::transform::Transform::new(
                Vec3::new(0.5, 0.22, -0.15),
                Vec3::zeros(),
                Vec3::new(0.8, 0.02, 0.4),
            ),
            mat_material,
        );
        let mat_id = ObjectId(scene.objects.len());
        scene.objects.push(Box::new(welcome_mat));
        scene.add_interactive(
            mat_id,
            crate::core::interaction::InteractiveKind::CafeEntrance,
        );

        let clues = [0, 1, 2].map(|i| interactive_id(&scene, InteractiveKind::Clue(i)));
        let door = interactive_id(&scene, InteractiveKind::BackDoor);
        assert!(!scene.toggle_secret_room());
        for id in clues {
            assert!(!scene.discover_clue(id));
        }
        let mut last_count = 0;
        for day in 1..=4 {
            scene.set_story_day(day);
            let count = (0..scene.objects.len())
                .filter(|i| scene.is_object_visible(ObjectId(*i)))
                .count();
            assert!(count > last_count);
            last_count = count;
            assert_eq!(scene.game_state.clues_count(), 0);
        }
        scene.set_story_day(5);
        scene.camera = create_camera();
        pick_near(&scene, Vec3::new(2.15, 0.40, -2.1), clues[0]);
        assert!(scene.discover_clue(clues[0]));
        assert!(!scene.discover_clue(clues[0]));
        assert!(!scene.toggle_secret_room());
        scene.set_story_day(6);
        scene.set_day_phase(DayPhase::Day);
        assert!(!scene.discover_clue(clues[1]));
        scene.set_day_phase(DayPhase::Dawn);
        assert!(scene.is_object_visible(clues[1]));
        scene.set_day_phase(DayPhase::Night);
        scene.camera = create_camera();
        pick_near(&scene, Vec3::new(3.2, 0.46, 0.95), clues[1]);
        assert!(scene.discover_clue(clues[1]));
        assert!(!scene.toggle_secret_room());
        scene.set_story_day(7);
        scene.set_day_phase(DayPhase::Sunset);
        assert!(!scene.discover_clue(clues[2]));
        scene.set_day_phase(DayPhase::Night);
        println!("CLUE 2 VISIBLE: {}", scene.is_object_visible(clues[2]));
        pick_near(&scene, Vec3::new(2.6, 0.38, 1.1), clues[2]);
        assert!(scene.discover_clue(clues[2]));
        assert_eq!(scene.game_state.clues_count(), 3);
        scene.camera = create_camera();
        // pick_near(&scene, Vec3::new(2.5, 0.125, -1.5), door);
        println!("CLUE 0: {:?}", clues[0]);
        println!("CLUE 1: {:?}", clues[1]);
        println!("CLUE 2: {:?}", clues[2]);
        println!("DOOR: {:?}", door);
        assert!(scene.is_object_visible(door));
        assert!(scene.toggle_secret_room());
        scene.skybox = crate::core::scene::Skybox::new(Color::new(0, 0, 0));
        assert!(scene.game_state.secret_room_open);
        assert!(!scene.is_object_visible(ObjectId(0)));
        assert!(scene.is_object_visible(door));
        assert!(scene.lights[scene.secret_room_lights[0]].intensity > 0.0);
        assert!(scene.toggle_secret_room());
        scene.skybox = crate::core::scene::Skybox::new(Color::new(0, 0, 0));
        scene.set_story_day(7);
        assert!(scene.is_object_visible(ObjectId(0)));
        assert!(scene.is_object_visible(door));
        assert!(clues.iter().all(|id| !scene.is_object_visible(*id)));
    }

    #[test]
    fn each_day_changes_the_render_without_hud() {
        let mut scene = create_scene(create_camera());
        let mut frame = Framebuffer::new(240, 180);
        let mut previous = Vec::new();
        for day in 1..=7 {
            scene.set_story_day(day);
            render(&mut frame, &scene, 0, 1);
            assert!(
                frame.buffer != previous,
                "Day {day} has no visible scene change"
            );
            previous.clone_from(&frame.buffer);
        }
    }

    #[test]
    fn f12_shortcut_opens_secret_room_without_clues_and_toggles_it_closed() {
        let mut scene = create_scene(create_camera());
        let door = interactive_id(&scene, InteractiveKind::BackDoor);

        assert_eq!(scene.game_state.clues_count(), 0);
        assert!(!scene.game_state.door_unlocked);
        assert!(scene.is_object_visible(ObjectId(0)));
        assert!(toggle_secret_room_shortcut(&mut scene));
        assert!(scene.game_state.door_unlocked);
        assert!(scene.game_state.secret_room_open);
        assert!(!scene.is_object_visible(ObjectId(0)));
        assert!(scene.is_object_visible(door));

        assert!(toggle_secret_room_shortcut(&mut scene));
        assert!(!scene.game_state.secret_room_open);
        assert!(scene.is_object_visible(ObjectId(0)));
        assert!(scene.is_object_visible(door));
    }
}

