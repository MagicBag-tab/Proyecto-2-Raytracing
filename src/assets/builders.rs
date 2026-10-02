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
        scaled_position(position, Vec3::new(0.0, -0.4, -2.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(9.0, 0.4, 12.0), scale),
        &materials.dark_wood,
    );

    // Deep inverted mountain/base
    let mut ground = materials.stone.clone();
    ground.diffuse = Color::new(40, 35, 30); // dark earth
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, -2.0, -2.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(8.6, 1.6, 11.6), scale),
        &ground,
    );

    // Add stairs descending into the dark
    let mut stone_step = materials.stone.clone();
    stone_step.diffuse = Color::new(80, 80, 80);
    for i in 0..7 {
        let step_y = 0.0 - (i as f32 * 0.25);
        let step_z = -1.1 - (i as f32 * 0.15);
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(2.5, step_y, step_z), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.8, 0.1, 0.3), scale),
            &stone_step,
        );
    }


    // Inner gravel area
    let mut gravel = materials.stone.clone();
    gravel.diffuse = Color::new(190, 190, 185);
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.0, -0.05, -2.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(8.6, 0.3, 11.6), scale),
        &gravel,
    );

    // Grass zones
    // Left Zone
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-2.6, 0.11, -2.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(3.2, 0.02, 11.2), scale),
        &materials.grass,
    );
    // Right Zone
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(3.5, 0.11, -2.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.4, 0.02, 11.2), scale),
        &materials.grass,
    );
    // Back Zone
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.6, 0.11, -4.95), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(3.2, 0.02, 6.3), scale),
        &materials.grass,
    );

    // Front patches to break up the gravel
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(2.4, 0.11, 2.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.6, 0.02, 1.4), scale),
        &materials.grass,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-0.8, 0.11, 0.8), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.0, 0.02, 1.2), scale),
        &materials.grass,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(1.0, 0.11, 2.8), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.2, 0.02, 0.8), scale),
        &materials.grass,
    );

    // Add tatami floor inside the cafe space
    let mut tatami = materials.grass.clone();
    tatami.texture = Some(std::sync::Arc::new(Texture::load_or_procedural(
        "assets/suelo/textura_piso_tatami_japones.png",
        ProceduralTexture::Wood,
    )));
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.5, 0.11, -1.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(2.8, 0.03, 1.7), scale),
        &tatami,
    );

    // Stone garden base
    let mut stone_base = materials.stone.clone();
    stone_base.diffuse = Color::new(165, 165, 170);
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-1.2, 0.105, 2.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.6, 0.02, 1.2), scale),
        &stone_base,
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
    // The rear leaf is supplied by build_back_door and registered in Scene.

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
    for panel_center_x in [0.93] { // Left panel open for interior view
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

    // Noren (Entrance curtain) added for identity
    let mut noren_mat = materials.paper.clone();
    noren_mat.diffuse = Color::new(50, 70, 140); // Indigo blue Noren
    for i in 0..3 {
        let nx = -0.3 + (i as f32 * 0.3);
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(nx, 1.25, 0.86), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.28, 0.6, 0.02), scale),
            &noren_mat,
        );
    }

    // Interior (Tatami, Table, Cups, Warm light)
    let tatami = materials.paper.clone().with_albedo(0.6).with_texture(Arc::new(Texture::load_or_procedural("assets/objetos_textura/tatami.png", ProceduralTexture::Paper)));
    let mut tatami_mat = tatami.clone();
    tatami_mat.diffuse = Color::new(200, 210, 160); // light green/yellow tatami fallback

    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-0.6, 0.23, 0.1), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.4, 0.04, 1.2), scale),
        &tatami_mat,
    );

    // Low table
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-0.6, 0.35, 0.1), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.6, 0.2, 0.4), scale),
        &materials.wood,
    );

    // Cups on table
    let ceramica = materials.stone.clone();
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(-0.7, 0.44, 0.1), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.04, 0.06, 0.04), scale),
        &ceramica,
    );
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(-0.5, 0.44, 0.1), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.04, 0.06, 0.04), scale),
        &ceramica,
    );

    // Interior warm lamp
    let glow = materials.warm_glow.clone().with_transparency(0.0, 1.0);
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(-1.0, 0.5, -0.3), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.12, 0.2, 0.12), scale),
        &glow,
    );

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
    let glowing_paper = materials
        .paper
        .clone()
        .with_emission(Color::new(255, 181, 76), 1.5);

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
            &glowing_paper,
        );
    }
    for x in [-0.145, 0.145] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.88, 0.0), scale),
            Vec3::zeros(),
            scaled_size(Vec3::new(0.018, 0.25, 0.24), scale),
            &glowing_paper,
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
    let mut path_mat = materials.stone.clone();
    path_mat.diffuse = Color::new(110, 115, 110); // Contrasting darker stepping stones

    // Continuous winding path of stepping stones
    let steps = 11;
    for i in 0..=steps {
        let t = i as f32 / steps as f32; // 0 to 1
        let z = 3.2 - 2.8 * t; // Starts at 3.2, ends at 0.4
        let x = 0.0 * (1.0 - t) + 0.4 * t + 0.15 * f32::sin(t * std::f32::consts::PI);

        let rot_y = (i as f32 * 2.4).sin() * 0.15; // Slight orientation variation

        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, Vec3::new(x, 0.105, z), scale),
            Vec3::new(0.0, rot_y, 0.0),
            scaled_size(Vec3::new(0.45, 0.02, 0.28), scale),
            &path_mat,
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

/// Open-backed miniature room, revealed as a cutaway behind the cafe.
pub fn build_secret_room_interior(
    position: Vec3,
    scale: f32,
    materials: &AssetMaterials,
) -> Vec<Object> {
    let mut objects = Vec::new();
    let wall = materials
        .stone
        .clone()
        .with_texture(Arc::new(Texture::load_or_procedural(
            "assets/paredes/pared_sótano.png",
            ProceduralTexture::Stone,
        )));
    for (offset, size) in [
        (Vec3::new(0.0, 0.12, 0.0), Vec3::new(2.2, 0.12, 1.8)),
        (Vec3::new(-1.06, 0.52, 0.0), Vec3::new(0.08, 0.8, 1.8)),
        (Vec3::new(1.06, 0.52, 0.0), Vec3::new(0.08, 0.8, 1.8)),
        (Vec3::new(0.0, 0.72, 0.86), Vec3::new(2.2, 1.2, 0.08)),
    ] {
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(position, offset, scale),
            Vec3::zeros(),
            scaled_size(size, scale),
            &wall,
        );
    }
    // Wooden frame and the existing metal mirror, polished for readable reflections.
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-0.73, 0.78, 0.78), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.65, 1.0, 0.07), scale),
        &materials.wood,
    );
    let mut mirror = materials.metal.clone().with_reflectivity(0.94);
    mirror.texture = None;
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-0.73, 0.78, 0.73), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.57, 0.86, 0.025), scale),
        &mirror,
    );
    // An optically curved vessel in front of a recognizable paper panel.
    let mut glass = materials.glass.clone();
    glass.texture = None;
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(0.48, 0.52, -0.10), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.27, 0.34, 0.27), scale),
        &glass,
    );
    let picture = materials
        .paper
        .clone()
        .with_texture(Arc::new(Texture::load_or_procedural(
            "assets/objetos_textura/flores_jardín/cuadro_1.png",
            ProceduralTexture::Paper,
        )));
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.48, 0.60, 0.48), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.66, 0.80, 0.04), scale),
        &picture,
    );
    // Red seal on the panel: straight edges make the glass distortion visible.
    let seal = Material::new(Color::new(220, 48, 45)).with_albedo(0.9);
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.43, 0.50, 0.44), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.09, 0.54, 0.03), scale),
        &seal,
    );
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(0.48, 0.58, 0.43), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.52, 0.07, 0.03), scale),
        &seal,
    );
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(-0.70, 0.35, -0.30), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.16, 0.17, 0.16), scale),
        &materials.metal,
    );
    let glow = materials.warm_glow.clone().with_transparency(0.0, 1.0);
    append_primitive(
        &mut objects,
        Box::new(Sphere),
        scaled_position(position, Vec3::new(-0.60, 0.46, -0.18), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.14, 0.26, 0.14), scale),
        &glow,
    );
    // Three paper keepsakes echo the collected clues, and appear in the mirror.
    for (i, color) in [
        Color::new(220, 55, 65),
        Color::new(80, 155, 205),
        Color::new(240, 206, 115),
    ]
    .into_iter()
    .enumerate()
    {
        let paper = materials.paper.clone().with_emission(color, 0.12);
        let mut paper = paper;
        paper.texture = None;
        paper.diffuse = color;
        append_primitive(
            &mut objects,
            Box::new(Cube),
            scaled_position(
                position,
                Vec3::new(-0.68 + i as f32 * 0.22, 0.27, -0.65),
                scale,
            ),
            Vec3::new(-0.3, 0.0, 0.0),
            scaled_size(Vec3::new(0.16, 0.20, 0.04), scale),
            &paper,
        );
    }
    objects
}

pub fn build_back_door(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();
    // Trapdoor horizontal in the garden
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(2.5, 0.125, -1.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(1.0, 0.05, 1.0), scale),
        &materials.dark_wood,
    );
    objects
}

pub fn build_day2_flowers(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();
    let mut asagao = materials.paper.clone();
    asagao.texture = Some(std::sync::Arc::new(Texture::load_or_procedural(
        "assets/objetos_textura/flores_jardín/asagao.png",
        ProceduralTexture::Paper,
    )));
    let mut ayame = materials.paper.clone();
    ayame.texture = Some(std::sync::Arc::new(Texture::load_or_procedural(
        "assets/objetos_textura/flores_jardín/ayame.png",
        ProceduralTexture::Paper,
    )));

    // Front left garden flower
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-1.8, 0.15, 1.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.4, 0.4, 0.02), scale),
        &asagao,
    );
    // Right side garden flower
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(3.0, 0.15, 1.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.4, 0.4, 0.02), scale),
        &ayame,
    );

    objects
}

pub fn build_day3_decorations(
    position: Vec3,
    scale: f32,
    materials: &AssetMaterials,
) -> Vec<Object> {
    let mut objects = Vec::new();
    let mut ceramica = materials.stone.clone();
    ceramica.texture = Some(std::sync::Arc::new(Texture::load_or_procedural(
        "assets/objetos_textura/cerámica_1.png",
        ProceduralTexture::Paper,
    )));
    let mut maceta = materials.stone.clone();
    maceta.texture = Some(std::sync::Arc::new(Texture::load_or_procedural(
        "assets/objetos_textura/maceta.png",
        ProceduralTexture::Paper,
    )));

    // Cups on the cafe bench
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(0.8, 0.5, 1.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.04, 0.08, 0.04), scale),
        &ceramica,
    );
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(1.1, 0.5, 1.0), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.05, 0.06, 0.05), scale),
        &ceramica,
    );

    // Pot near the back side
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(-0.8, 0.15, -1.5), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.2, 0.2, 0.2), scale),
        &maceta,
    );

    objects
}

pub fn build_day5_higanbana(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();
    let mut higanbana = materials.paper.clone();
    higanbana.texture = Some(std::sync::Arc::new(Texture::load_or_procedural(
        "assets/objetos_textura/flores_jardín/higanbana.png",
        ProceduralTexture::Paper,
    )));

    // Special flower near the back door
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(2.15, 0.40, -2.1), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.5, 0.5, 0.02), scale),
        &higanbana,
    );

    objects
}

pub fn build_day6_clue(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();
    let mut mat = materials.paper.clone();
    mat.texture = Some(std::sync::Arc::new(Texture::load_or_procedural(
        "assets/objetos_textura/flores_jardín/cuadro_1.png",
        ProceduralTexture::Paper,
    )));
    // Plaque near the right lantern
    append_primitive(
        &mut objects,
        Box::new(Cube),
        scaled_position(position, Vec3::new(3.2, 0.46, 0.95), scale),
        Vec3::zeros(),
        scaled_size(Vec3::new(0.44, 0.44, 0.035), scale),
        &mat,
    );
    objects
}

pub fn build_day7_clue(position: Vec3, scale: f32, materials: &AssetMaterials) -> Vec<Object> {
    let mut objects = Vec::new();
    let mut mat = materials.paper.clone();
    mat.diffuse = Color::new(220, 210, 180);
    // Scroll on the cafe patio
    append_primitive(
        &mut objects,
        Box::new(Cylinder),
        scaled_position(position, Vec3::new(2.6, 0.38, 1.1), scale),
        Vec3::new(0.0, 0.0, 1.57),
        scaled_size(Vec3::new(0.10, 0.52, 0.10), scale),
        &mat,
    );
    objects
}
