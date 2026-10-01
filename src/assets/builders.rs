use crate::core::object::{Object, Shape};
use crate::core::ray_intersect::Material;
use crate::core::transform::Transform;
use crate::materials::color::Color;
use crate::materials::presets::{self, Preset};
use crate::materials::texture::{ProceduralTexture, Texture};
use crate::shapes::cone::Cone;
use crate::shapes::cube::Cube;
use crate::shapes::cylinder::Cylinder;
use crate::shapes::pyramid::Pyramid;
use crate::shapes::sphere::Sphere;
use nalgebra_glm::Vec3;
use std::sync::Arc;

fn load_optional_map(path: &str) -> Option<Arc<Texture>> {
    Texture::try_new(path).ok().map(Arc::new)
}

fn with_optional_maps(mut material: Material, name: &str) -> Material {
    let normal_path = format!("assets/textures/{name}_normal.png");
    let specular_path = format!("assets/textures/{name}_specular.png");
    let overlay_path = format!("assets/textures/{name}_overlay.png");
    let overlay_normal_path = format!("assets/textures/{name}_overlay_normal.png");

    if let Some(texture) = load_optional_map(&normal_path) {
        material = material.with_normal_map(texture);
    }
    if let Some(texture) = load_optional_map(&specular_path) {
        material = material.with_specular_map(texture);
    }
    if let (Some(overlay), Some(normal)) = (
        load_optional_map(&overlay_path),
        load_optional_map(&overlay_normal_path),
    ) {
        material = material.with_overlay(overlay, normal);
    }
    material
}

pub struct AssetMaterials {
    pub wood: Material,
    pub stone: Material,
    pub metal: Material,
    pub glass: Material,
    pub paper: Material,
    pub grass: Material,
    pub terracotta: Material,
    pub foliage: Material,
    pub blossom: Material,
    pub warm_glow: Material,
}

impl AssetMaterials {
    pub fn new() -> Self {
        Self {
            wood: with_optional_maps(presets::create(Preset::Wood), "wood"),
            stone: with_optional_maps(presets::create(Preset::Stone), "stone"),
            metal: with_optional_maps(presets::create(Preset::Metal), "metal"),
            glass: with_optional_maps(presets::create(Preset::Glass), "glass"),
            paper: with_optional_maps(presets::create(Preset::Paper), "paper"),
            grass: with_optional_maps(
                Material::new(Color::new(63, 122, 58))
                    .with_albedo(0.92)
                    .with_specular(10.0, 0.04)
                    .with_texture(Arc::new(Texture::load_or_procedural(
                        "assets/textures/grass.png",
                        ProceduralTexture::Grass,
                    ))),
                "grass",
            ),
            terracotta: Material::new(Color::new(157, 61, 47))
                .with_albedo(0.82)
                .with_specular(24.0, 0.16),
            foliage: Material::new(Color::new(44, 112, 54))
                .with_albedo(0.92)
                .with_specular(12.0, 0.04),
            blossom: Material::new(Color::new(229, 91, 143))
                .with_albedo(0.88)
                .with_specular(28.0, 0.16),
            warm_glow: Material::new(Color::new(255, 202, 112))
                .with_albedo(0.35)
                .with_specular(64.0, 0.5)
                .with_transparency(0.18, 1.15)
                .with_emission(Color::new(255, 166, 67), 1.8),
        }
    }
}

fn append_primitive(
    objects: &mut Vec<Object>,
    shape: Box<dyn Shape>,
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
    material: &Material,
) {
    objects.push(Object::new(
        shape,
        Transform::new(position, rotation, scale),
        material.clone(),
    ));
}

fn scaled_position(origin: Vec3, offset: Vec3, scale: f32) -> Vec3 {
    origin + offset * scale
}

fn scaled_size(size: Vec3, scale: f32) -> Vec3 {
    size * scale
}

pub fn create_tree(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::with_capacity(3);
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(0.0, 1.25, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.34, 2.5, 0.34), scale),
        &materials.wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Cone),
        scaled_position(position, Vec3::new(0.0, 1.15, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(2.25, 3.2, 2.25), scale),
        &materials.foliage,
    );
    append_primitive(
        &mut objects,
        Box::new(Cone),
        scaled_position(position, Vec3::new(0.0, 2.0, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.65, 2.4, 1.65), scale),
        &materials.foliage,
    );
    objects
}

pub fn create_sakura(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::with_capacity(6);
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(0.0, 1.1, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.3, 2.2, 0.3), scale),
        &materials.wood,
    );
    for (offset, radius) in [
        (Vec3::new(0.0, 1.7, 0.0), 1.1),
        (Vec3::new(-0.65, 1.35, 0.05), 0.78),
        (Vec3::new(0.62, 1.4, 0.08), 0.82),
        (Vec3::new(-0.1, 2.12, -0.32), 0.76),
        (Vec3::new(0.2, 1.92, 0.48), 0.7),
    ] {
        append_primitive(
            &mut objects,
            Box::new(Sphere),
            scaled_position(position, offset, scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(radius, radius * 0.82, radius), scale),
            &materials.blossom,
        );
    }
    objects
}

pub fn create_lantern(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::with_capacity(3);
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(0.0, 1.225, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.12, 2.45, 0.12), scale),
        &materials.metal,
    );
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(0.0, 2.55, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.32, 0.32, 0.32), scale),
        &materials.warm_glow,
    );
    append_primitive(
        &mut objects,
        Box::new(Cone),
        scaled_position(position, Vec3::new(0.0, 2.9, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.7, 0.7, 0.7), scale),
        &materials.metal,
    );
    objects
}

pub fn create_fence(position: Vec3, length: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::with_capacity(9);
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.65, 0.0), length),
        Vec3::zeros(),
        Vec3::new(length, 0.2, 0.24),
        &materials.wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.15, 0.0), length),
        Vec3::zeros(),
        Vec3::new(length, 0.16, 0.24),
        &materials.wood,
    );
    let post_count = (length / 0.8).ceil() as usize + 1;
    for index in 0..post_count {
        let x = -length * 0.5 + index as f32 * length / (post_count - 1) as f32;
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.4, 0.0), 1.0),
            Vec3::zeros(),
            Vec3::new(0.16, 0.9, 0.24),
            &materials.wood,
        );
    }
    objects
}

pub fn create_table(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::with_capacity(5);
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.85, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.8, 0.18, 1.2), scale),
        &materials.wood,
    );
    for x in [-0.72, 0.72] {
        for z in [-0.43, 0.43] {
            append_primitive(
                &mut objects,
                Box::new(Cylinder),
                scaled_position(position, Vec3::new(x, 0.38, z), scale),
                Vec3::zeros(),
                scaled_size(Vec3::new(0.1, 0.8, 0.1), scale),
                &materials.wood,
            );
        }
    }
    objects
}

pub fn create_chair(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::with_capacity(6);
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.55, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.9, 0.14, 0.82), scale),
        &materials.wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 1.08, -0.34), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.9, 1.05, 0.14), scale),
        &materials.wood,
    );
    for x in [-0.34, 0.34] {
        for z in [-0.3, 0.3] {
            append_primitive(
                &mut objects,
                Box::new(Cylinder),
                scaled_position(position, Vec3::new(x, 0.25, z), scale),
                Vec3::zeros(),
                scaled_size(Vec3::new(0.08, 0.55, 0.08), scale),
                &materials.wood,
            );
        }
    }
    objects
}

pub fn create_door(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    create_door_with_knob(position, scale, materials, 0.085)
}

pub fn create_back_door(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    create_door_with_knob(position, scale, materials, -0.085)
}

fn create_door_with_knob(
    position: Vec3,
    scale: f32,
    materials: &AssetMaterials,
    knob_offset_z: f32,
) -> Vec<Object> {
    let mut objects = Vec::with_capacity(2);
    append_primitive(
        &mut objects,
        Box::new(Cube),
        position,
        Vec3::zeros(),
        scaled_size(Vec3::new(0.78, 1.55, 0.08), scale),
        &materials.wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(0.27, 0.0, knob_offset_z), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.1, 0.1, 0.1), scale),
        &materials.metal,
    );
    objects
}

pub fn create_house(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::with_capacity(24);
    for x in [-1.79, 1.79] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.9, 0.0), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.22, 2.8, 2.45), scale),
            &materials.paper,
        );
    }
    for z in [-1.115, 1.115] {
        for x in [-1.145, 1.145] {
            append_primitive(
                &mut objects,
                Box::new(Cube),
                scaled_position(position, Vec3::new(x, 0.9, z), scale),
                Vec3::zeros(),
                scaled_size(Vec3::new(1.51, 2.8, 0.22), scale),
                &materials.paper,
            );
        }
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(0.0, 1.9, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.78, 1.8, 0.22), scale),
            &materials.paper,
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, -0.46, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(3.35, 0.08, 2.05), scale),
        &materials.wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Pyramid),
        scaled_position(position, Vec3::new(0.0, 2.7, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(4.35, 1.3, 2.85), scale),
        &materials.terracotta,
    );
    objects.extend(create_door(
        scaled_position(position, Vec3::new(0.0, 0.22, 1.255), scale),
        scale,
        materials,
    ));
    for x in [-1.2, 1.2] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.82, 1.24), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.86, 0.78, 0.08), scale),
            &materials.glass,
        );
        for (offset, size) in [
            (Vec3::new(x, 0.82, 1.31), Vec3::new(0.94, 0.08, 0.08)),
            (Vec3::new(x, 0.82, 1.31), Vec3::new(0.08, 0.86, 0.08)),
        ] {
            append_primitive(
                &mut objects,
                Box::new(Cube),
                scaled_position(position, offset, scale),
                Vec3::zeros(),
                scaled_size(size, scale),
                &materials.wood,
            );
        }
    }
    for (index, x) in [-1.5, -0.75, 0.0, 0.75, 1.5].into_iter().enumerate() {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 1.55, 1.5), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.74, 0.24, 0.62), scale),
            if index % 2 == 0 {
                &materials.terracotta
            } else {
                &materials.paper
            },
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(1.2, 3.5, -0.35), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.48, 1.4, 0.48), scale),
        &materials.stone,
    );
    objects
}
