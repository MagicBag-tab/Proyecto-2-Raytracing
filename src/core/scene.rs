use crate::core::camera::Camera;
use crate::core::light::Light;
use crate::core::ray_intersect::RayIntersect;
use crate::materials::color::Color;
use nalgebra_glm::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPhase {
    Day,
    Sunset,
    Night,
}

impl DayPhase {
    pub fn next(self) -> Self {
        match self {
            Self::Day => Self::Sunset,
            Self::Sunset => Self::Night,
            Self::Night => Self::Day,
        }
    }
}

pub struct Skybox {
    pub color: Color,
    horizon: Color,
    zenith: Color,
}

impl Skybox {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            horizon: Color::new(34, 48, 64),
            zenith: Color::new(106, 145, 181),
        }
    }

    fn set_phase(&mut self, phase: DayPhase) {
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
        };
        self.horizon = horizon;
        self.zenith = zenith;
        self.color = tint;
    }

    pub fn sample(&self, direction: &Vec3) -> Color {
        let blend = ((direction.y + 0.15) / 0.85).clamp(0.0, 1.0);
        let base = Color::new(
            horizon_component(self.horizon, self.zenith, blend, 0) as u8,
            horizon_component(self.horizon, self.zenith, blend, 1) as u8,
            horizon_component(self.horizon, self.zenith, blend, 2) as u8,
        );
        let tint = self.color;
        base * 0.82 + tint * 0.18
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
        };
        scene.set_day_phase(DayPhase::Day);
        scene
    }

    pub fn advance_day_phase(&mut self) {
        self.set_day_phase(self.day_phase.next());
    }

    pub fn set_day_phase(&mut self, phase: DayPhase) {
        self.day_phase = phase;
        self.skybox.set_phase(phase);

        let (sun_color, sun_intensity, lantern_intensity, ambient, lantern_emission) = match phase {
            DayPhase::Day => (Color::new(255, 247, 226), 2.4, 0.0, 0.35, 0.0),
            DayPhase::Sunset => (Color::new(255, 143, 91), 1.65, 0.48, 0.24, 0.65),
            DayPhase::Night => (Color::new(91, 121, 191), 0.38, 1.25, 0.13, 1.0),
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
    }
}
