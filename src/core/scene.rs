use crate::core::camera::Camera;
use crate::core::interaction::{GameState, InteractiveKind, ObjectId};
use crate::core::light::Light;
use crate::core::ray_intersect::RayIntersect;
use crate::materials::color::Color;
use crate::materials::texture::Texture;
use nalgebra_glm::Vec3;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPhase {
    Day,
    Sunset,
    Night,
    Dawn,
}

impl DayPhase {
    pub fn next(self) -> Self {
        match self {
            Self::Day => Self::Sunset,
            Self::Sunset => Self::Night,
            Self::Night => Self::Dawn,
            Self::Dawn => Self::Day,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Day => 0,
            Self::Sunset => 1,
            Self::Night => 2,
            Self::Dawn => 3,
        }
    }
}

pub struct Skybox {
    pub color: Color,
    horizon: Color,
    zenith: Color,
    phase_layers: [Vec<Texture>; 4],
    active_phase: DayPhase,
}

impl Skybox {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            horizon: Color::new(34, 48, 64),
            zenith: Color::new(106, 145, 181),
            phase_layers: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            active_phase: DayPhase::Day,
        }
    }

    pub fn load_phase_layers(
        &mut self,
        phase: DayPhase,
        paths: &[&str],
    ) -> Result<(), image::ImageError> {
        let mut layers = Vec::new();
        for path in paths {
            let texture = Texture::try_new(path)?;
            layers.push(texture);
        }
        self.phase_layers[phase.index()] = layers;
        Ok(())
    }

    fn set_phase(&mut self, phase: DayPhase) {
        self.active_phase = phase;
        let (horizon, zenith, tint) = match phase {
            DayPhase::Day => (
                Color::new(151, 198, 222),
                Color::new(62, 137, 202),
                Color::new(230, 242, 255),
            ),
            DayPhase::Sunset => (
                Color::new(239, 132, 91),
                Color::new(91, 78, 126),
                Color::new(255, 193, 136),
            ),
            DayPhase::Night => (
                Color::new(28, 39, 66),
                Color::new(5, 10, 31),
                Color::new(51, 67, 104),
            ),
            DayPhase::Dawn => (
                Color::new(192, 145, 145),
                Color::new(106, 115, 181),
                Color::new(245, 230, 240),
            ),
        };
        self.horizon = horizon;
        self.zenith = zenith;
        self.color = tint;
    }

    pub fn sample(&self, origin: &Vec3, direction: &Vec3) -> Color {
        let blend = ((direction.y + 0.15) / 0.85).clamp(0.0, 1.0);
        let mut current_color = Color::new(
            horizon_component(self.horizon, self.zenith, blend, 0) as u8,
            horizon_component(self.horizon, self.zenith, blend, 1) as u8,
            horizon_component(self.horizon, self.zenith, blend, 2) as u8,
        );
        let tint = self.color;
        current_color = current_color * 0.82 + tint * 0.18;

        let layers = &self.phase_layers[self.active_phase.index()];
        if layers.is_empty() {
            return current_color;
        }

        let radii = [80.0, 70.0, 60.0, 50.0];

        for (i, texture) in layers.iter().enumerate() {
            let radius = *radii.get(i).unwrap_or(&50.0);
            
            let ox = origin.x;
            let oz = origin.z;
            let dx = direction.x;
            let dz = direction.z;

            let a = dx * dx + dz * dz;
            if a < 1e-6 { continue; }
            
            let b = 2.0 * (ox * dx + oz * dz);
            let c = ox * ox + oz * oz - radius * radius;
            
            let discriminant = b * b - 4.0 * a * c;
            if discriminant > 0.0 {
                let t = (-b + discriminant.sqrt()) / (2.0 * a);
                if t > 0.0 {
                    let p = origin + direction * t;
                    let u = 0.5 + p.z.atan2(p.x) / (2.0 * PI);
                    let v = 0.5 - direction.y.clamp(-1.0, 1.0).asin() / PI;
                    
                    let tex_color = texture.get_color(u, v);
                    let tex_alpha = texture.get_alpha(u, v);
                    
                    current_color = current_color * (1.0 - tex_alpha) + tex_color * tex_alpha;
                }
            }
        }

        current_color
    }
}

fn horizon_component(horizon: Color, zenith: Color, blend: f32, channel: usize) -> f32 {
    let horizon = horizon.to_hex();
    let zenith = zenith.to_hex();
    let shift = 16 - channel * 8;
    let start = ((horizon >> shift) & 0xff) as f32;
    let end = ((zenith >> shift) & 0xff) as f32;
    start + (end - start) * blend
}

pub struct Scene {
    pub objects: Vec<Box<dyn RayIntersect>>,
    pub lights: Vec<Light>,
    pub camera: Camera,
    pub skybox: Skybox,
    pub day_phase: DayPhase,
    pub ambient_intensity: f32,
    pub lantern_emission: f32,
    pub game_state: GameState,
    interactive_objects: HashMap<ObjectId, InteractiveKind>,
    hidden_objects: HashSet<ObjectId>,
    door_objects: Vec<ObjectId>,
    secret_room_objects: Vec<ObjectId>,
}

impl Scene {
    pub fn new(
        objects: Vec<Box<dyn RayIntersect>>,
        lights: Vec<Light>,
        camera: Camera,
        skybox: Skybox,
    ) -> Self {
        let mut scene = Self {
            objects,
            lights,
            camera,
            skybox,
            day_phase: DayPhase::Day,
            ambient_intensity: 0.35,
            lantern_emission: 0.0,
            game_state: GameState::default(),
            interactive_objects: HashMap::new(),
            hidden_objects: HashSet::new(),
            door_objects: Vec::new(),
            secret_room_objects: Vec::new(),
        };
        scene.set_day_phase(DayPhase::Day);
        scene
    }

    pub fn advance_day_phase(&mut self) {
        self.set_day_phase(self.day_phase.next());
    }

    pub fn register_interactive(
        &mut self,
        object_index: usize,
        kind: InteractiveKind,
        visible: bool,
    ) -> ObjectId {
        let id = ObjectId(object_index);
        self.interactive_objects.insert(id, kind);
        self.set_object_visible(id, visible && self.kind_is_visible(kind));
        id
    }

    pub fn register_door_part(&mut self, object_index: usize) -> ObjectId {
        let id = self.register_interactive(object_index, InteractiveKind::BackDoor, true);
        self.door_objects.push(id);
        id
    }

    pub fn register_secret_room_object(&mut self, object_index: usize) -> ObjectId {
        let id = ObjectId(object_index);
        self.secret_room_objects.push(id);
        self.set_object_visible(id, false);
        id
    }

    pub fn interaction_for(&self, id: ObjectId) -> Option<InteractiveKind> {
        self.interactive_objects.get(&id).copied()
    }

    pub fn set_object_visible(&mut self, id: ObjectId, visible: bool) {
        if visible {
            self.hidden_objects.remove(&id);
        } else {
            self.hidden_objects.insert(id);
        }
    }

    pub fn is_object_visible(&self, id: ObjectId) -> bool {
        !self.hidden_objects.contains(&id)
    }

    pub fn discover_clue(&mut self, id: ObjectId) -> bool {
        let Some(InteractiveKind::Clue(index)) = self.interaction_for(id) else {
            return false;
        };
        if !self.game_state.found_clue(index) {
            return false;
        }
        self.set_object_visible(id, false);
        true
    }

    pub fn toggle_secret_room(&mut self) -> bool {
        if !self.game_state.door_unlocked {
            return false;
        }
        let opening = !self.game_state.secret_room_open;
        self.game_state.secret_room_open = opening;
        let offset = if opening { 1.0 } else { -1.0 };
        for id in self.door_objects.iter().copied() {
            if let Some(object) = self.objects.get_mut(id.0) {
                object.translate_by(&Vec3::new(offset, 0.0, 0.0));
            }
        }
        let secret_room_objects = self.secret_room_objects.clone();
        for id in secret_room_objects {
            self.set_object_visible(id, opening);
        }
        true
    }

    pub fn move_interactive(&mut self, id: ObjectId, delta: Vec3) -> bool {
        if self.interaction_for(id) != Some(InteractiveKind::Movable) {
            return false;
        }
        self.objects
            .get_mut(id.0)
            .is_some_and(|object| object.translate_by(&delta))
    }

    fn kind_is_visible(&self, kind: InteractiveKind) -> bool {
        match kind {
            InteractiveKind::Clue(index) => {
                !self
                    .game_state
                    .clues_found
                    .get(index)
                    .copied()
                    .unwrap_or(true)
                    && match index {
                        0 => true,
                        1 => self.day_phase != DayPhase::Day,
                        2 => self.day_phase == DayPhase::Night,
                        _ => false,
                    }
            }
            InteractiveKind::BackDoor | InteractiveKind::Movable => true,
        }
    }

    fn refresh_clue_visibility(&mut self) {
        let clues: Vec<_> = self
            .interactive_objects
            .iter()
            .filter_map(|(&id, &kind)| {
                matches!(kind, InteractiveKind::Clue(_)).then_some((id, kind))
            })
            .collect();
        for (id, kind) in clues {
            let visible = self.kind_is_visible(kind);
            self.set_object_visible(id, visible);
        }
    }

    pub fn set_day_phase(&mut self, phase: DayPhase) {
        self.day_phase = phase;
        self.skybox.set_phase(phase);

        let (sun_color, sun_intensity, lantern_intensity, ambient, lantern_emission) = match phase {
            DayPhase::Day => (Color::new(255, 247, 226), 2.4, 0.0, 0.35, 0.0),
            DayPhase::Sunset => (Color::new(255, 143, 91), 1.65, 0.48, 0.24, 0.65),
            DayPhase::Night => (Color::new(91, 121, 191), 0.38, 1.25, 0.13, 1.0),
            DayPhase::Dawn => (Color::new(255, 185, 140), 1.2, 0.2, 0.2, 0.3),
        };
        if let Some(sun) = self.lights.first_mut() {
            sun.color = sun_color;
            sun.intensity = sun_intensity;
        }
        for lantern in self.lights.iter_mut().skip(1) {
            lantern.intensity = lantern_intensity;
        }
        self.ambient_intensity = ambient;
        self.lantern_emission = lantern_emission;
        self.refresh_clue_visibility();
    }
}

#[cfg(test)]
mod tests {
    use super::{DayPhase, Scene, Skybox};
    use crate::core::camera::Camera;
    use crate::core::interaction::InteractiveKind;
    use crate::materials::color::Color;
    use nalgebra_glm::Vec3;

    #[test]
    fn clue_visibility_changes_with_phase_and_collection() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::zeros(),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let mut scene = Scene::new(
            Vec::new(),
            Vec::new(),
            camera,
            Skybox::new(Color::new(0, 0, 0)),
        );
        let note = scene.register_interactive(0, InteractiveKind::Clue(1), true);

        assert!(!scene.is_object_visible(note));
        scene.set_day_phase(DayPhase::Sunset);
        assert!(scene.is_object_visible(note));
        assert!(scene.discover_clue(note));
        assert!(!scene.is_object_visible(note));
    }
}
