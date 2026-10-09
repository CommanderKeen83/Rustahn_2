#[derive(Debug, Clone, PartialEq)]
pub struct MovementData {
    pub thrust_speed: f32,
    pub max_speed: f32,
    pub rotation_speed: f32,
    pub descent_speed: f32,
}
impl MovementData {
    pub fn new(thrust_speed: f32, max_speed: f32, rotation_speed: f32) -> Self {
        MovementData {
            thrust_speed,
            max_speed,
            rotation_speed,
            descent_speed: player_stats::DESCENT_THRUST,
        }
    }

    pub fn with_descent(mut self, descent_speed: f32) -> Self {
        self.descent_speed = descent_speed;
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

use sola_raylib::prelude::*;

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

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsData {
    pub gravity_scale: f32,
    pub drag: f32,
    pub is_static: bool,
    pub mass: f32,
}

impl PhysicsData {
    pub fn new() -> Self {
        Self {
            gravity_scale: 1.0,
            drag: world_stats::DRAG,
            is_static: false,
            mass: 1.0,
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
        }
    }

    pub fn with_gravity(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
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
}

impl Default for PhysicsData {
    fn default() -> Self {
        Self::new()
    }
}

pub mod fireball_stats {
    pub const MAX_SPEED: f32 = 850.0;
    pub const LIFETIME: f32 = 2.0;
}

pub mod player_stats {
    pub const MAX_SPEED: f32 = 12550.0;
    pub const ROTATION_SPEED: f32 = 200.0;
    pub const THRUST_POWER: f32 = 800.0;
    pub const LIFT_THRUST: f32 = 35.0;
    pub const DESCENT_THRUST: f32 = 75.0;
}

pub mod world_stats {
    pub const DRAG: f32 = 3.0;
    pub const GRAVITY: f32 = 80.0;
}
