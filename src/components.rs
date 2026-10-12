#[derive(Debug, Clone, PartialEq)]
pub struct MovementData {
    pub thrust_speed: f32,
    pub max_speed: f32,
    pub rotation_speed: f32,
    pub descent_speed: f32,
    pub tilt_angle: f32,
}
impl MovementData {
    pub fn new(thrust_speed: f32, max_speed: f32, rotation_speed: f32) -> Self {
        MovementData {
            thrust_speed,
            max_speed,
            rotation_speed,
            descent_speed: player_stats::DESCENT_THRUST,
            tilt_angle: player_stats::TILT_ANGLE,
        }
    }

    pub fn with_descent(mut self, descent_speed: f32) -> Self {
        self.descent_speed = descent_speed;
        self
    }

    pub fn with_tilt_angle(mut self, tilt_angle: f32) -> Self {
        self.tilt_angle = tilt_angle;
        self
    }
}
#[derive(Debug, Clone)]
pub struct AnimationData {
    pub timer: f32,
    pub frame_duration: f32,
    pub current_frame: usize,
    pub framecount_per_row: usize,
    pub rows: usize,
    pub active_only_on_move: bool,
}

impl Default for AnimationData {
    fn default() -> Self {
        Self {
            timer: 0.0,
            frame_duration: 0.0,
            current_frame: 0,
            framecount_per_row: 8,
            rows: 0,
            active_only_on_move: false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct KnockbackData {
    pub impulse: f32,
}

impl Default for KnockbackData {
    fn default() -> Self {
        Self { impulse: 1.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Mortality {
    pub lifetime: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeaponType {
    PlayerBullet,
    EnemyBullet,
    EnemyFireball,
    EnemyFreezeBeam,
}
#[derive(Debug, Clone)]
pub struct WeaponSet {
    pub cooldown: f32,
    pub timer: f32,
    pub weapon: WeaponType,
}

/// Component representing enemy Gobbler AI state and configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GobblerData {
    /// Whether the Gobbler has been activated by entering the camera view.
    pub is_active: bool,
    /// Horizontal movement speed in virtual px/s.
    pub speed: f32,
}

impl GobblerData {
    pub const DEFAULT_SPEED: f32 = 40.0;
    pub const WIDTH: f32 = 20.0;
    pub const HEIGHT: f32 = 20.0;

    pub fn new(speed: f32) -> Self {
        Self {
            is_active: false,
            speed,
        }
    }
}

impl Default for GobblerData {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SPEED)
    }
}

use crate::types::EntityType;
use sola_raylib::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub offset: Vector2,
    pub is_solid: bool,
}

impl Collider {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            offset: Vector2::new(0.0, 0.0),
            is_solid: true,
        }
    }

    pub fn with_offset(width: f32, height: f32, offset: Vector2) -> Self {
        Self {
            width,
            height,
            offset,
            is_solid: true,
        }
    }

    pub fn sensor(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            offset: Vector2::new(0.0, 0.0),
            is_solid: false,
        }
    }

    /// Returns the world-space axis-aligned bounding box (min_x, min_y, max_x, max_y)
    /// given the entity center position.
    pub fn aabb(&self, position: Vector2) -> (f32, f32, f32, f32) {
        let cx = position.x + self.offset.x;
        let cy = position.y + self.offset.y;
        let hw = self.width * 0.5;
        let hh = self.height * 0.5;
        (cx - hw, cy - hh, cx + hw, cy + hh)
    }

    /// Returns a Raylib `Rectangle` in world coordinates.
    pub fn bounds(&self, position: Vector2) -> Rectangle {
        Rectangle::new(
            position.x + self.offset.x - self.width * 0.5,
            position.y + self.offset.y - self.height * 0.5,
            self.width,
            self.height,
        )
    }
}

/// Standard preset profiles for entity physics weight and falling dynamics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicsPreset {
    Light,
    Normal,
    Heavy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsData {
    pub gravity_scale: f32,
    pub drag: f32,
    pub is_static: bool,
    pub mass: f32,
    pub max_fall_speed: Option<f32>,
}

impl PhysicsData {
    pub const LIGHT_GRAVITY_SCALE: f32 = 0.5;
    pub const NORMAL_GRAVITY_SCALE: f32 = 6.0;
    pub const HEAVY_GRAVITY_SCALE: f32 = 9.5;
    pub const LIGHT_MAX_FALL_SPEED: f32 = 25.0;

    pub fn new() -> Self {
        Self {
            gravity_scale: 1.0,
            drag: world_stats::DRAG,
            is_static: false,
            mass: 1.0,
            max_fall_speed: None,
        }
    }

    pub fn dynamic() -> Self {
        Self::new()
    }

    pub fn static_body() -> Self {
        Self {
            gravity_scale: 0.0,
            drag: 0.0,
            is_static: true,
            mass: 0.0,
            max_fall_speed: None,
        }
    }

    pub fn with_gravity(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    pub fn with_max_fall_speed(mut self, max_speed: f32) -> Self {
        self.max_fall_speed = Some(max_speed);
        self
    }

    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag;
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Creates a physics component based on standard preset profiles:
    /// - `Light`: `gravity_scale = 0.2`, `max_fall_speed = Some(25.0)` (floats gently)
    /// - `Normal`: `gravity_scale = 1.0`, `max_fall_speed = None` (standard fall)
    /// - `Heavy`: `gravity_scale = 1.5`, `max_fall_speed = None` (drops rapidly)
    /// Reads from configuration file overrides if present, otherwise uses default constants.
    /// Creates a physics component based on standard preset profiles:
    /// - `Light`: `gravity_scale = LIGHT_GRAVITY_SCALE` (0.2), `max_fall_speed = Some(25.0)` (floats gently)
    /// - `Normal`: `gravity_scale = NORMAL_GRAVITY_SCALE` (1.0), `max_fall_speed = None` (standard fall)
    /// - `Heavy`: `gravity_scale = HEAVY_GRAVITY_SCALE` (1.5), `max_fall_speed = None` (drops rapidly)
    pub fn preset(preset: PhysicsPreset) -> Self {
        let config = PhysicsConfig::global();
        Self::from_config(&config, preset)
    }

    pub fn light() -> Self {
        Self::preset(PhysicsPreset::Light)
    }

    pub fn normal() -> Self {
        Self::preset(PhysicsPreset::Normal)
    }

    pub fn heavy() -> Self {
        Self::preset(PhysicsPreset::Heavy)
    }

    /// Creates a physics component from a dynamic `PhysicsConfig` (e.g. loaded from `physics.cfg`).
    pub fn from_config(config: &PhysicsConfig, preset: PhysicsPreset) -> Self {
        match preset {
            PhysicsPreset::Light => {
                let mut data = Self::new().with_gravity_scale(config.light_gravity_scale);
                if let Some(spd) = config.light_max_fall_speed {
                    data = data.with_max_fall_speed(spd);
                }
                data
            }
            PhysicsPreset::Normal => {
                let mut data = Self::new().with_gravity_scale(config.normal_gravity_scale);
                if let Some(spd) = config.normal_max_fall_speed {
                    data = data.with_max_fall_speed(spd);
                }
                data
            }
            PhysicsPreset::Heavy => {
                let mut data = Self::new().with_gravity_scale(config.heavy_gravity_scale);
                if let Some(spd) = config.heavy_max_fall_speed {
                    data = data.with_max_fall_speed(spd);
                }
                data
            }
        }
    }

    /// Obtains a physics component preset for a given entity type, checking
    /// for configuration file overrides before falling back to default type presets.
    pub fn for_entity_type(tag: EntityType) -> Self {
        let config = PhysicsConfig::global();
        if let Some(custom) = config.get_for_type(tag) {
            return custom;
        }
        match tag {
            EntityType::Ground | EntityType::Obstacle => Self::static_body(),
            EntityType::Rabbit => Self::light(),
            EntityType::Bomb => Self::heavy(),
            _ => Self::dynamic(),
        }
    }
}

impl Default for PhysicsData {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic configuration for physics presets and per-entity-type settings.
/// Can be populated from file (e.g. `resources/physics.cfg`) or string.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsConfig {
    pub light_gravity_scale: f32,
    pub light_max_fall_speed: Option<f32>,
    pub normal_gravity_scale: f32,
    pub normal_max_fall_speed: Option<f32>,
    pub heavy_gravity_scale: f32,
    pub heavy_max_fall_speed: Option<f32>,
    pub type_overrides: HashMap<String, (f32, Option<f32>)>,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            light_gravity_scale: PhysicsData::LIGHT_GRAVITY_SCALE,
            light_max_fall_speed: Some(PhysicsData::LIGHT_MAX_FALL_SPEED),
            normal_gravity_scale: PhysicsData::NORMAL_GRAVITY_SCALE,
            normal_max_fall_speed: None,
            heavy_gravity_scale: PhysicsData::HEAVY_GRAVITY_SCALE,
            heavy_max_fall_speed: None,
            type_overrides: HashMap::new(),
        }
    }
}

impl PhysicsConfig {
    pub fn parse_str(content: &str) -> Self {
        let mut config = Self::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with('[') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "light.gravity_scale" => {
                        if let Ok(v) = value.parse::<f32>() {
                            config.light_gravity_scale = v;
                        }
                    }
                    "light.max_fall_speed" => {
                        if value.is_empty() || value.eq_ignore_ascii_case("none") {
                            config.light_max_fall_speed = None;
                        } else if let Ok(v) = value.parse::<f32>() {
                            config.light_max_fall_speed = Some(v);
                        }
                    }
                    "normal.gravity_scale" => {
                        if let Ok(v) = value.parse::<f32>() {
                            config.normal_gravity_scale = v;
                        }
                    }
                    "normal.max_fall_speed" => {
                        if value.is_empty() || value.eq_ignore_ascii_case("none") {
                            config.normal_max_fall_speed = None;
                        } else if let Ok(v) = value.parse::<f32>() {
                            config.normal_max_fall_speed = Some(v);
                        }
                    }
                    "heavy.gravity_scale" => {
                        if let Ok(v) = value.parse::<f32>() {
                            config.heavy_gravity_scale = v;
                        }
                    }
                    "heavy.max_fall_speed" => {
                        if value.is_empty() || value.eq_ignore_ascii_case("none") {
                            config.heavy_max_fall_speed = None;
                        } else if let Ok(v) = value.parse::<f32>() {
                            config.heavy_max_fall_speed = Some(v);
                        }
                    }
                    other => {
                        if let Some((entity_name, prop)) = other.split_once('.') {
                            let entry = config
                                .type_overrides
                                .entry(entity_name.to_string())
                                .or_insert((1.0, None));
                            match prop {
                                "gravity_scale" => {
                                    if let Ok(v) = value.parse::<f32>() {
                                        entry.0 = v;
                                    }
                                }
                                "max_fall_speed" => {
                                    if value.is_empty() || value.eq_ignore_ascii_case("none") {
                                        entry.1 = None;
                                    } else if let Ok(v) = value.parse::<f32>() {
                                        entry.1 = Some(v);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        config
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        std::fs::read_to_string(path).ok().map(|content| Self::parse_str(&content))
    }

    pub fn global() -> Self {
        if let Some(cfg) = Self::load_from_file("resources/physics.cfg") {
            return cfg;
        }
        if let Some(cfg) = Self::load_from_file("physics.cfg") {
            return cfg;
        }
        Self::default()
    }

    pub fn get_for_type(&self, tag: EntityType) -> Option<PhysicsData> {
        let name = format!("{:?}", tag);
        if let Some(&(scale, max_fall)) = self.type_overrides.get(&name) {
            let mut data = PhysicsData::dynamic().with_gravity_scale(scale);
            if let Some(spd) = max_fall {
                data = data.with_max_fall_speed(spd);
            }
            Some(data)
        } else {
            None
        }
    }
}

pub mod fireball_stats {
    pub const MAX_SPEED: f32 = 850.0;
    pub const LIFETIME: f32 = 2.0;
}

pub mod player_stats {
    pub const MAX_SPEED: f32 = 12550.0;
    pub const ROTATION_SPEED: f32 = 45.0;
    pub const THRUST_POWER: f32 = 800.0;
    pub const LIFT_THRUST: f32 = 35.0;
    pub const DESCENT_THRUST: f32 = 75.0;
    pub const TILT_ANGLE: f32 = 6.0;
}

pub mod world_stats {
    pub const DRAG: f32 = 3.0;
    pub const GRAVITY: f32 = 80.0;
    /// Baseline global downward pull (80.0), referenced as World::Drag in specifications.
    pub const BASELINE_GRAVITY: f32 = 80.0;
}
