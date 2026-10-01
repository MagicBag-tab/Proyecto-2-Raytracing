import re

builders_path = r"C:\Users\sarah\Universidad\GRAFICAS_POR_COMPUTADOR\Proyecto-2-Raytracing\src\assets\builders.rs"
main_path = r"C:\Users\sarah\Universidad\GRAFICAS_POR_COMPUTADOR\Proyecto-2-Raytracing\src\main.rs"

with open(builders_path, "r", encoding="utf-8") as f:
    builders_code = f.read()

# 1. Edit AssetMaterials::new() paths
materials_new = """pub struct AssetMaterials {
    pub wood: Material,
    pub dark_wood: Material,
    pub light_wood: Material,
    pub paper: Material,
    pub stone: Material,
    pub terracotta: Material,
    pub grass: Material,
    pub glass: Material,
    pub metal: Material,
}

impl AssetMaterials {
    pub fn new() -> Self {
        let mut wood = Material::new_solid(Color::new(120, 80, 50), 0.05, 0.05);
        wood.texture = Some("assets/paredes/madera_techo_interior.png".to_string());

        let mut dark_wood = Material::new_solid(Color::new(70, 40, 30), 0.05, 0.05);
        dark_wood.texture = Some("assets/paredes/madera_fachada.png".to_string());

        let mut light_wood = Material::new_solid(Color::new(180, 150, 100), 0.05, 0.05);
        light_wood.texture = Some("assets/objetos_textura/bambú.png".to_string());

        let mut paper = Material::new_solid(Color::new(245, 240, 230), 0.05, 0.05);
        paper.texture = Some("assets/paredes/japanese-style-room.jpg".to_string());

        let mut grass = Material::new_solid(Color::new(100, 150, 80), 0.05, 0.05);
        grass.texture = Some("assets/suelo/cesped.png".to_string());

        let mut stone = Material::new_solid(Color::new(140, 140, 145), 0.0, 0.0);
        let mut terracotta = Material::new_solid(Color::new(180, 90, 70), 0.1, 0.05);
        
        let glass = Material::new_glass(1.5, 0.1);
        let metal = Material::new_mirror(0.9);

        Self {
            wood,
            dark_wood,
            light_wood,
            paper,
            stone,
            terracotta,
            grass,
            glass,
            metal,
        }
    }
}"""

builders_code = re.sub(r"pub struct AssetMaterials \{.*?\n    }\n}", materials_new, builders_code, flags=re.DOTALL)

# 2. Modify `build_base_diorama`
base_diorama_new = """pub fn build_base_diorama(pos: Vec3, scale: f32, mats: &AssetMaterials) -> Vec<Primitive> {
    let mut prims = Vec::new();
    let dx = pos.x;
    let dy = pos.y;
    let dz = pos.z;

    // Wooden base frame (18x14)
    prims.push(Primitive::new_cube(
        Vec3::new(9.0 * scale, 0.4 * scale, 7.0 * scale),
        mats.wood.clone(),
        Vec3::new(dx, dy, dz),
    ));

    // Inner gravel / grass area
    prims.push(Primitive::new_cube(
        Vec3::new(8.6 * scale, 0.1 * scale, 6.6 * scale),
        mats.stone.clone(), // or grass if they prefer, wait, gravel
        Vec3::new(dx, dy + 0.45 * scale, dz),
    ));
    
    // Add tatami floor inside the cafe space
    // Assuming cafe is at x=1.0, z=-1.5, size is roughly 2x1.5
    let mut tatami = mats.grass.clone();
    tatami.texture = Some("assets/suelo/textura_piso_tatami_japones.png".to_string());
    prims.push(Primitive::new_cube(
        Vec3::new(2.4 * scale, 0.1 * scale, 1.9 * scale),
        tatami,
        Vec3::new(dx + 1.0, dy + 0.55 * scale, dz - 1.5),
    ));

    prims
}"""

builders_code = re.sub(r"pub fn build_base_diorama.*?^}", base_diorama_new, builders_code, flags=re.DOTALL | re.MULTILINE)

# 3. Path Curve
path_new = """pub fn build_path(pos: Vec3, scale: f32, mats: &AssetMaterials) -> Vec<Primitive> {
    let mut prims = Vec::new();
    
    let path_points = [
        Vec3::new(-2.0, 0.0, 6.0),
        Vec3::new(-1.8, 0.0, 4.5),
        Vec3::new(-1.0, 0.0, 3.0),
        Vec3::new(-0.2, 0.0, 1.5),
        Vec3::new(0.3, 0.0, 0.5),
        Vec3::new(0.5, 0.0, -0.2),
    ];
    
    for (i, p) in path_points.iter().enumerate() {
        let mut stone_mat = mats.stone.clone();
        if i % 2 == 0 { stone_mat.diffuse = Color::new(120, 120, 120); }
        let slab = Primitive::new_cube(
            Vec3::new(0.5 * scale, 0.05 * scale, 0.3 * scale),
            stone_mat,
            pos + *p,
        );
        prims.push(slab);
    }
    prims
}"""

builders_code = re.sub(r"pub fn build_path.*?^}", path_new, builders_code, flags=re.DOTALL | re.MULTILINE)


# 4. Remove build_secret_room_interior as it shouldn't be built yet
builders_code = re.sub(r"pub fn build_secret_room_interior.*?^}", "", builders_code, flags=re.DOTALL | re.MULTILINE)


with open(builders_path, "w", encoding="utf-8") as f:
    f.write(builders_code)


with open(main_path, "r", encoding="utf-8") as f:
    main_code = f.read()

# Update create_scene in main.rs
scene_new = """fn create_scene(camera: Camera) -> Scene {
    let materials = AssetMaterials::new();
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    let mut appears_on_day = std::collections::HashMap::new();

    // The entire platform and path (Day 1)
    let p_base = append_asset(&mut objects, build_base_diorama(Vec3::zeros(), 1.0, &materials));
    appears_on_day.insert(p_base, 1);
    let p_path = append_asset(&mut objects, build_path(Vec3::new(0.0, 0.5, 0.0), 1.0, &materials));
    appears_on_day.insert(p_path, 1);

    // ================= CAFÉ (Day 1) =================
    // Shifted cafe
    let p_cafe = append_asset(&mut objects, build_japanese_cafe(Vec3::new(1.0, 0.5, -1.5), 1.0, &materials));
    appears_on_day.insert(p_cafe, 1);
    
    let door_indices = append_asset_indices(&mut objects, build_back_door(Vec3::new(1.0, 0.5, -1.5), 1.0, &materials));
    for idx in &door_indices { appears_on_day.insert(*idx, 1); }

    // ================= VEGETATION & GARDEN (Day 1) =================
    // Left Rock Garden
    let p_rock = append_asset(&mut objects, build_rock_garden(Vec3::new(-4.5, 0.5, -0.5), 1.2, &materials));
    appears_on_day.insert(p_rock, 1);
    
    // Large Sakura (Left)
    let p_sak1 = append_asset(&mut objects, build_sakura_tree_variant(Vec3::new(-4.5, 0.5, -1.0), 1.4, &materials, 0));
    appears_on_day.insert(p_sak1, 1);
    
    // Medium Sakura (Right)
    let p_sak2 = append_asset(&mut objects, build_sakura_tree_variant(Vec3::new(5.0, 0.5, -1.0), 1.0, &materials, 1));
    appears_on_day.insert(p_sak2, 1);
    
    // Bamboo behind the large sakura
    let p_bam1 = append_asset(&mut objects, build_bamboo_cluster(Vec3::new(-5.0, 0.5, -4.0), 1.1, &materials));
    appears_on_day.insert(p_bam1, 1);

    // Bamboo scattered left front
    let p_bam2 = append_asset(&mut objects, build_bamboo_cluster(Vec3::new(-4.5, 0.5, 3.0), 0.9, &materials));
    appears_on_day.insert(p_bam2, 1);
    
    // Bamboo back right framing the door from distance
    let p_bam3 = append_asset(&mut objects, build_bamboo_cluster(Vec3::new(6.0, 0.5, -4.0), 1.0, &materials));
    appears_on_day.insert(p_bam3, 1);
    
    // Bamboo directly behind the cafe framing the door
    let p_bam4 = append_asset(&mut objects, build_bamboo_cluster(Vec3::new(-1.0, 0.5, -5.0), 1.0, &materials));
    appears_on_day.insert(p_bam4, 1);

    // Small flower clusters
    let p_fl1 = append_asset(&mut objects, build_flower_cluster(Vec3::new(-3.5, 0.5, 2.0), 1.0, &materials));
    appears_on_day.insert(p_fl1, 1);
    let p_fl2 = append_asset(&mut objects, build_flower_cluster(Vec3::new(-2.5, 0.5, -1.0), 1.0, &materials));
    appears_on_day.insert(p_fl2, 1);

    // ================= LANTERNS (Day 1) =================
    let mut toro_materials = materials.stone.clone();
    toro_materials.emission = Color::new(0, 0, 0); // off during day
    
    let p_toro1 = append_asset(&mut objects, build_toro_lantern(Vec3::new(-1.0, 0.5, 2.5), 0.8, &toro_materials));
    appears_on_day.insert(p_toro1, 1);
    let p_toro2 = append_asset(&mut objects, build_toro_lantern(Vec3::new(-5.5, 0.5, -1.0), 0.6, &toro_materials));
    appears_on_day.insert(p_toro2, 1);

    // ================= LIGHTS =================
    let mut lights = Vec::new();
    lights.push(Light {
        position: Vec3::new(-10.0, 15.0, 10.0),
        color: Color::new(255, 240, 220),
        intensity: 1.2,
    });
    lights.push(Light {
        position: Vec3::new(-1.0, 1.2, 2.5),
        color: Color::new(255, 140, 50),
        intensity: 0.0,
    });
    lights.push(Light {
        position: Vec3::new(-5.5, 1.0, -1.0),
        color: Color::new(255, 140, 50),
        intensity: 0.0,
    });

    Scene::new(camera, objects, lights, appears_on_day, vec![], 0, door_indices)
}"""

main_code = re.sub(r"fn create_scene.*?^}", scene_new, main_code, flags=re.DOTALL | re.MULTILINE)

with open(main_path, "w", encoding="utf-8") as f:
    f.write(main_code)
