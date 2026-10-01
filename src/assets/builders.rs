use crate::core::object::{Object, Shape};
use crate::core::ray_intersect::Material;
use crate::core::transform::Transform;
use crate::materials::color::Color;
use crate::materials::presets::{self, Preset};
use crate::materials::texture::{ProceduralTexture, Texture};
use crate::shapes::cube::Cube;
use crate::shapes::cylinder::Cylinder;
use crate::shapes::pyramid::Pyramid;
use crate::shapes::sphere::Sphere;
use nalgebra_glm::Vec3;
use std::f32::consts::PI;
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
    pub dark_wood: Material,
    pub light_wood: Material,
    pub bamboo: Material,
    pub shoji: Material,
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
                        "assets/suelo/cesped.png",
                        ProceduralTexture::Grass,
                    ))),
                "grass",
            ),
            terracotta: Material::new(Color::new(50, 50, 60)) // Dark roof tiles
                .with_albedo(0.8)
                .with_specular(12.0, 0.1),
            foliage: Material::new(Color::new(44, 112, 54))
                .with_albedo(0.92)
                .with_specular(12.0, 0.04),
            blossom: Material::new(Color::new(245, 175, 205))
                .with_albedo(0.88)
                .with_specular(28.0, 0.16),
            warm_glow: Material::new(Color::new(255, 202, 112))
                .with_albedo(0.35)
                .with_specular(64.0, 0.5)
                .with_transparency(0.18, 1.15)
                .with_emission(Color::new(255, 180, 80), 2.2),
            dark_wood: Material::new(Color::new(55, 35, 25))
                .with_albedo(0.7)
                .with_specular(20.0, 0.08),
            light_wood: Material::new(Color::new(190, 160, 120))
                .with_albedo(0.9)
                .with_specular(15.0, 0.05)
                .with_texture(Arc::new(Texture::load_or_procedural(
                    "assets/suelo/textura_piso_tatami_japones.png",
                    ProceduralTexture::Wood,
                ))),
            bamboo: Material::new(Color::new(110, 150, 70))
                .with_albedo(0.9)
                .with_specular(30.0, 0.1),
            shoji: with_optional_maps(
                Material::new(Color::new(228, 218, 192))
                    .with_albedo(0.88)
                    .with_specular(10.0, 0.035)
                    .with_texture(Arc::new(Texture::load_or_procedural(
                        "assets/paredes/vecteezy_texture-from-japanese-paper_3162046.jpg",
                        ProceduralTexture::Paper,
                    ))),
                "shoji",
            ),
        }
    }
}

pub fn append_primitive(
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

// ---------------------------------------------------------
// DIORAMA COMPOSITION BUILDERS
// ---------------------------------------------------------

pub fn build_base_diorama(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    // Main floating platform (Wooden base)
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, -0.4, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(7.0, 0.4, 5.0), scale),
        &materials.dark_wood,
    );

    // Narrow moss strips frame the larger gravel garden and route.
    for x in [-2.56, 2.56] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, -0.05, 0.0), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.88, 0.3, 4.5), scale),
            &materials.grass,
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, -0.025, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(4.18, 0.31, 4.5), scale),
        &materials.stone,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, -0.025, 1.48), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.72, 0.34, 2.0), scale),
        &materials.light_wood,
    );

    objects
}

pub fn build_japanese_cafe(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    // Raised tatami floor and the compact structural frame.
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.03, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(3.15, 0.16, 1.95), scale),
        &materials.dark_wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.16, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(2.92, 0.10, 1.74), scale),
        &materials.light_wood,
    );

    // A shallow entry step bridges the raised floor to the garden path.
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.16, 1.02), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.08, 0.10, 0.32), scale),
        &materials.wood,
    );

    let wall_center_y = 0.76;
    let wall_height = 1.10;
    let front_z = 0.84;
    let back_z = -0.84;

    for x in [-1.47, -0.45, 0.45, 1.47] {
        for z in [front_z, back_z] {
            append_primitive(
                &mut objects,
                Box::new(Cube),
                scaled_position(position, Vec3::new(x, wall_center_y, z), scale),
                Vec3::zeros(),
                scaled_size(Vec3::new(0.12, wall_height, 0.12), scale),
                &materials.dark_wood,
            );
        }
    }

    // Front and rear crossbeams define the eaves without filling the walls.
    for z in [front_z, back_z] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(0.0, 1.35, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(3.05, 0.12, 0.14), scale),
            &materials.dark_wood,
        );
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(0.0, 0.29, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(3.02, 0.12, 0.12), scale),
            &materials.wood,
        );
    }

    // The rear elevation has paper wall sections and a closed, visible back door.
    for x in [-0.93, 0.93] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, wall_center_y, -0.85), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.92, 1.06, 0.05), scale),
            &materials.shoji,
        );
    }
    objects.extend(create_door(
        scaled_position(position, Vec3::new(0.0, 0.76, -0.81), scale),
        scale,
        materials,
    ));

    for x in [-1.47, 1.47] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 1.35, 0.0), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.14, 0.12, 1.72), scale),
            &materials.dark_wood,
        );
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.29, 0.0), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.12, 0.12, 1.72), scale),
            &materials.wood,
        );
    }

    // Shoji side bays and a divided glass window give the side elevation depth.
    for z in [-0.34, 0.38] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(-1.45, 0.76, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.045, 0.82, 0.48), scale),
            &materials.shoji,
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(1.45, 0.91, 0.30), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.045, 0.52, 0.66), scale),
        &materials.glass,
    );
    for z in [0.02, 0.30, 0.58] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(1.49, 0.91, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.035, 0.52, 0.025), scale),
            &materials.wood,
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(1.49, 0.91, 0.30), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.035, 0.025, 0.66), scale),
        &materials.wood,
    );

    // Two paper shoji panels with narrow wooden lattice bars.
    for panel_center_x in [-0.93, 0.93] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(panel_center_x, 0.76, 0.86), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.86, 0.82, 0.035), scale),
            &materials.shoji,
        );
        for x_offset in [-0.32, -0.16, 0.0, 0.16, 0.32] {
            append_primitive(
                &mut objects,
                Box::new(Cube),
                scaled_position(
                    position,
                    Vec3::new(panel_center_x + x_offset, 0.76, 0.89),
                    scale,
                ),
                Vec3::zeros(),
                scaled_size(Vec3::new(0.022, 0.78, 0.025), scale),
                &materials.wood,
            );
        }
        for y_offset in [-0.25, 0.0, 0.25] {
            append_primitive(
                &mut objects,
                Box::new(Cube),
                scaled_position(
                    position,
                    Vec3::new(panel_center_x, 0.76 + y_offset, 0.89),
                    scale,
                ),
                Vec3::zeros(),
                scaled_size(Vec3::new(0.82, 0.022, 0.025), scale),
                &materials.wood,
            );
        }
    }

    // A proper framed front door is recessed behind three separated noren strips.
    objects.extend(create_door(
        scaled_position(position, Vec3::new(0.0, 0.74, 0.79), scale),
        scale,
        materials,
    ));
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 1.27, 0.94), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.86, 0.045, 0.045), scale),
        &materials.dark_wood,
    );
    for x in [-0.25, 0.0, 0.25] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 1.12, 0.94), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.21, 0.25, 0.025), scale),
            &materials.paper,
        );
    }

    // Three wider paper strips make the noren readable above the door.
    for x in [-0.32, 0.0, 0.32] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 1.07, 1.03), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.20, 0.28, 0.025), scale),
            &materials.paper,
        );
    }

    // Front plaque and cup emblem establish the cafe from the arrival view.
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-1.30, 0.99, 1.08), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.48, 0.20, 0.075), scale),
        &materials.dark_wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-1.30, 0.99, 1.125), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.39, 0.13, 0.018), scale),
        &materials.paper,
    );
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(-1.30, 0.99, 1.145), scale),
        Vec3::new(PI / 2.0, 0.0, 0.0),
        scaled_size(Vec3::new(0.055, 0.035, 0.055), scale),
        &materials.terracotta,
    );

    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 1.30, 1.02), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.62, 0.055, 0.045), scale),
        &materials.dark_wood,
    );

    // A narrow side shelf reads as a serving counter without adding an interior.
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(1.55, 0.58, 0.32), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.42, 0.07, 0.58), scale),
        &materials.wood,
    );

    // Two thick overlapping roof wings rise to a continuous ridge and extend as eaves.
    for (z, rotation_x) in [(0.38, 0.34), (-0.38, -0.34)] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(0.0, 1.47, z), scale),
            Vec3::new(rotation_x, 0.0, 0.0),
            scaled_size(Vec3::new(3.52, 0.15, 1.22), scale),
            &materials.terracotta,
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 1.54, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(3.30, 0.16, 0.24), scale),
        &materials.dark_wood,
    );
    for x in [-1.64, 1.64] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 1.44, 0.0), scale),
            Vec3::new(0.0, 0.0, if x < 0.0 { -0.08 } else { 0.08 }),
            scaled_size(Vec3::new(0.10, 0.13, 1.92), scale),
            &materials.dark_wood,
        );
    }

    objects
}

pub fn build_sakura_tree(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    build_sakura_tree_variant(position, scale, materials, 0)
}

pub fn build_sakura_tree_variant(
    position: Vec3,
    scale: f32,
    materials: &AssetMaterials,
    variant: u8,
) -> Vec<Object> {
    let mut objects = Vec::new();

    let variation = if variant % 2 == 0 { 1.0 } else { 0.82 };
    let lean = if variant % 2 == 0 { 0.0 } else { -0.16 };

    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(lean * 0.35, 0.78, 0.0), scale),
        Vec3::new(0.0, 0.0, -lean),
        scaled_size(Vec3::new(0.095, 1.56, 0.095), scale),
        &materials.dark_wood,
    );

    let branches = [
        (
            Vec3::new(-0.28, 1.28, 0.0),
            Vec3::new(-0.42, 0.58, 0.12),
            0.045,
        ),
        (
            Vec3::new(0.28, 1.40, 0.02),
            Vec3::new(0.45, 0.62, -0.08),
            0.042,
        ),
        (
            Vec3::new(-0.18, 1.63, 0.0),
            Vec3::new(-0.34, 0.52, -0.18),
            0.035,
        ),
        (
            Vec3::new(0.20, 1.78, 0.0),
            Vec3::new(0.32, 0.48, 0.20),
            0.032,
        ),
        (
            Vec3::new(0.0, 1.52, 0.0),
            Vec3::new(0.02, 0.56, -0.28),
            0.034,
        ),
    ];
    for (offset, rotation, thickness) in branches {
        append_primitive(
            &mut objects,
            Box::new(Cylinder),
            scaled_position(position, offset, scale),
            rotation,
            scaled_size(Vec3::new(thickness, 0.72, thickness), scale),
            &materials.dark_wood,
        );
    }

    let clusters = if variant % 2 == 0 {
        [
            (Vec3::new(-0.48, 1.92, 0.05), 0.28),
            (Vec3::new(-0.18, 2.12, -0.18), 0.31),
            (Vec3::new(0.22, 2.06, 0.16), 0.30),
            (Vec3::new(0.52, 1.88, -0.05), 0.25),
            (Vec3::new(-0.05, 2.30, 0.03), 0.27),
            (Vec3::new(-0.38, 1.65, -0.22), 0.22),
            (Vec3::new(0.36, 1.68, 0.28), 0.24),
        ]
    } else {
        [
            (Vec3::new(-0.42, 1.82, -0.12), 0.24),
            (Vec3::new(-0.12, 2.04, 0.18), 0.28),
            (Vec3::new(0.24, 1.96, -0.18), 0.26),
            (Vec3::new(0.48, 1.72, 0.10), 0.21),
            (Vec3::new(-0.28, 2.24, 0.02), 0.23),
            (Vec3::new(0.16, 2.22, 0.30), 0.22),
            (Vec3::new(0.0, 1.62, -0.30), 0.20),
        ]
    };

    for (offset, size) in clusters {
        append_primitive(
            &mut objects,
            Box::new(Sphere),
            scaled_position(position, offset, scale),
            Vec3::zeros(),
            scaled_size(
                Vec3::new(
                    size * variation,
                    size * 0.78 * variation,
                    size * 0.92 * variation,
                ),
                scale,
            ),
            &materials.blossom,
        );
    }

    objects
}

pub fn build_bamboo_cluster(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    let stalks = [
        (Vec3::new(0.0, 0.0, 0.0), 1.2, 0.05),
        (Vec3::new(0.2, 0.0, 0.15), 1.4, -0.05),
        (Vec3::new(-0.15, 0.0, 0.2), 1.1, 0.08),
        (Vec3::new(0.1, 0.0, -0.2), 1.5, -0.02),
        (Vec3::new(-0.25, 0.0, -0.1), 1.0, 0.0),
    ];

    for (offset, height, tilt) in stalks {
        append_primitive(
            &mut objects,
            Box::new(Cylinder),
            scaled_position(position, Vec3::new(offset.x, height / 2.0, offset.z), scale),
            Vec3::new(tilt, 0.0, tilt * 0.5),
            scaled_size(Vec3::new(0.04, height, 0.04), scale),
            &materials.bamboo,
        );
    }

    for (offset, leaf_y, side) in [
        (Vec3::new(0.0, 0.0, 0.0), 1.0, -1.0),
        (Vec3::new(0.2, 0.0, 0.15), 1.18, 1.0),
        (Vec3::new(-0.15, 0.0, 0.2), 0.88, -1.0),
    ] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(
                position,
                Vec3::new(offset.x + 0.12 * side, leaf_y, offset.z),
                scale,
            ),
            Vec3::new(0.0, 0.0, side * 0.42),
            scaled_size(Vec3::new(0.25, 0.025, 0.07), scale),
            &materials.foliage,
        );
    }

    objects
}

pub fn build_toro_lantern(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.07, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.44, 0.14, 0.44), scale),
        &materials.stone,
    );
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(0.0, 0.42, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.13, 0.58, 0.13), scale),
        &materials.stone,
    );

    // Open lantern body with four wood posts and paper panels.
    for x in [-0.14, 0.14] {
        for z in [-0.14, 0.14] {
            append_primitive(
                &mut objects,
                Box::new(Cube),
                scaled_position(position, Vec3::new(x, 0.88, z), scale),
                Vec3::zeros(),
                scaled_size(Vec3::new(0.035, 0.34, 0.035), scale),
                &materials.dark_wood,
            );
        }
    }
    for z in [-0.145, 0.145] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(0.0, 0.88, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.24, 0.25, 0.018), scale),
            &materials.paper,
        );
    }
    for x in [-0.145, 0.145] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.88, 0.0), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.018, 0.25, 0.24), scale),
            &materials.paper,
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(0.0, 0.88, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.075, 0.10, 0.075), scale),
        &materials.warm_glow,
    );

    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(0.0, 1.08, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.07, 0.07, 0.07), scale),
        &materials.metal,
    );

    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 1.05, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.50, 0.07, 0.50), scale),
        &materials.stone,
    );
    append_primitive(
        &mut objects,
        Box::new(Pyramid),
        scaled_position(position, Vec3::new(0.0, 1.19, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.62, 0.24, 0.62), scale),
        &materials.stone,
    );
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(0.0, 1.42, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.035, 0.10, 0.035), scale),
        &materials.metal,
    );

    objects
}

pub fn build_cafe_patio(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.39, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.88, 0.07, 0.62), scale),
        &materials.wood,
    );
    for x in [-0.36, 0.36] {
        for z in [-0.23, 0.23] {
            append_primitive(
                &mut objects,
                Box::new(Cylinder),
                scaled_position(position, Vec3::new(x, 0.18, z), scale),
                Vec3::zeros(),
                scaled_size(Vec3::new(0.045, 0.36, 0.045), scale),
                &materials.dark_wood,
            );
        }
    }

    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-0.78, 0.24, 0.08), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.72, 0.08, 0.25), scale),
        &materials.wood,
    );
    for x in [-1.08, -0.48] {
        append_primitive(
            &mut objects,
            Box::new(Cylinder),
            scaled_position(position, Vec3::new(x, 0.11, 0.08), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.04, 0.22, 0.04), scale),
            &materials.dark_wood,
        );
    }

    for x in [-0.18, 0.18] {
        append_primitive(
            &mut objects,
            Box::new(Cylinder),
            scaled_position(position, Vec3::new(x, 0.46, 0.0), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.05, 0.07, 0.05), scale),
            &materials.paper,
        );
    }

    objects
}

pub fn build_rock_garden(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    let rocks = [
        (
            Vec3::new(-0.20, 0.10, 0.00),
            Vec3::new(0.36, 0.16, 0.30),
            0.20,
        ),
        (
            Vec3::new(0.22, 0.08, 0.16),
            Vec3::new(0.24, 0.12, 0.22),
            -0.40,
        ),
        (
            Vec3::new(0.12, 0.07, -0.24),
            Vec3::new(0.30, 0.13, 0.20),
            0.80,
        ),
    ];

    for (offset, size, rot) in rocks {
        append_primitive(
            &mut objects,
            Box::new(Sphere), // low-poly style sphere looks good as a rock
            scaled_position(position, offset, scale),
            Vec3::new(0.0, rot, 0.0),
            scaled_size(size, scale),
            &materials.stone,
        );
    }

    for (index, (x, z)) in [(-0.28, -0.10), (0.0, 0.18), (0.28, -0.04)]
        .into_iter()
        .enumerate()
    {
        append_primitive(
            &mut objects,
            Box::new(Cylinder),
            scaled_position(position, Vec3::new(x, 0.14, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.022, 0.28, 0.022), scale),
            &materials.foliage,
        );
        append_primitive(
            &mut objects,
            Box::new(Sphere),
            scaled_position(position, Vec3::new(x, 0.31, z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.09, 0.075, 0.09), scale),
            if index == 1 {
                &materials.paper
            } else {
                &materials.blossom
            },
        );
    }

    objects
}

pub fn build_path(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    let stones = [
        (Vec3::new(0.00, 0.12, 0.78), 0.02),
        (Vec3::new(-0.04, 0.12, 1.09), -0.04),
        (Vec3::new(0.03, 0.12, 1.40), 0.035),
        (Vec3::new(0.00, 0.12, 1.71), -0.025),
        (Vec3::new(0.04, 0.12, 2.02), 0.015),
        (Vec3::new(-0.02, 0.12, 2.33), -0.035),
    ];

    for (offset, rotation) in stones {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, offset, scale),
            Vec3::new(0.0, rotation, 0.0),
            scaled_size(Vec3::new(0.54, 0.09, 0.28), scale),
            &materials.stone,
        );
    }

    objects
}

pub fn create_door(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();

    // Framed leaf sized for the cafe entrance; components meet at their edges.
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.0, -0.012), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.68, 1.06, 0.045), scale),
        &materials.wood,
    );
    for x in [-0.37, 0.37] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.0, 0.018), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.08, 1.14, 0.08), scale),
            &materials.dark_wood,
        );
    }
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.55, 0.018), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.82, 0.08, 0.08), scale),
        &materials.dark_wood,
    );
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(0.22, -0.02, 0.065), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.055, 0.055, 0.055), scale),
        &materials.metal,
    );
    objects
}

pub fn build_secret_room_interior(
    position: Vec3,
    scale: f32,
    materials: &AssetMaterials,
) -> Vec<Object> {
    let mut objects = Vec::new();

    // Secret room base (darker stone)
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.1, 0.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.8, 0.05, 1.8), scale),
        &materials.stone,
    );

    // Magical reflecting mirror (metal)
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, 0.8, -0.85), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.8, 1.0, 0.05), scale),
        &materials.metal,
    );

    // Glass potion/bottle (Refraction)
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(-0.4, 0.4, -0.4), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.15, 0.4, 0.15), scale),
        &materials.glass,
    );

    // Small warm light source inside
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(0.4, 0.4, -0.4), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.15, 0.15, 0.15), scale),
        &materials.warm_glow,
    );

    objects
}
