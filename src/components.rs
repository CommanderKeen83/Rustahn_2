use sola_raylib::prelude::*;


#[derive(Debug, Clone, PartialEq)]
pub enum EntityType{
    Rabbit,
    Gobbler,
    EvilEye,
    Fireworm,
    Player,
    Bullet,
    Fireball,
    Bomb
}

impl EntityType{
    pub fn is_enemy(&self) -> bool{
        matches!(self, EntityType::Gobbler | EntityType::EvilEye | EntityType::Fireworm)
    }
    pub fn is_weapon(&self) -> bool{
        matches!(self, EntityType::Bullet | EntityType::Fireball | EntityType::Bomb)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpriteID{
    Rabbit,
    Gobbler,
    EvilEye,
    Fireworm,
    Player,
    Bullet,
    Fireball,
    Bomb,
    MenuBackground,
    GameIntroBackground,
    GameIntroStormCloud,
    GameIntroWindowLamp
}

#[derive(Debug, Clone)]
pub struct Entity{
    // Header - every entity has these properties
    pub tag: EntityType,
    pub pos: Vector2,
    pub vel: Vector2,
    pub rot: f32,
    pub alive: bool,
    pub sprite_id: SpriteID,

    // optional components
    pub movement_data: Option<MovementData>,
    pub physics_data: Option<PhysicsData>,
    pub weapon_set: Option<WeaponSet>,
    pub mortality: Option<Mortality>,
    pub animation_data: Option<AnimationData>,

}

impl Entity{
    pub fn new(tag: EntityType, pos: Vector2, sprite_id: SpriteID) -> Self {
        Entity{
            tag,
            pos,
            vel: Vector2::new(0.0, 0.0 ),
            rot: 0.0,
            alive: true,
            sprite_id,
            movement_data: None,
            physics_data: None,
            weapon_set: None,
            mortality: None,
            animation_data: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MovementData{
    pub thrust_speed: f32,
    pub max_speed: f32,
    pub rotation_speed: f32,
}
impl MovementData{
    pub fn new(thrust_speed: f32, max_speed: f32, rotation_speed: f32) -> Self{
        MovementData{
            thrust_speed,
            max_speed,
            rotation_speed,
        }
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
pub struct Mortality{
    pub lifetime: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeaponType{
    PlayerBullet,
    EnemyBullet,
    EnemyFireball,
    EnemyFreezeBeam,

}
#[derive(Debug, Clone)]
pub struct WeaponSet{
    pub cooldown: f32,
    pub timer: f32,
    pub weapon: WeaponType
}

#[derive(Debug, Clone)]
pub struct PhysicsData{

}

pub mod fireball_stats{
    pub const MAX_SPEED: f32 = 850.0;
    pub const LIFETIME: f32 = 2.0;
}

pub mod player_stats{
    pub const MAX_SPEED: f32 = 12550.0;
    pub const ROTATION_SPEED: f32 = 200.0;
    pub const THRUST_POWER: f32 = 800.0;
}

pub mod world_stats{
    pub const DRAG: f32 = 3.0;
}