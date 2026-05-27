
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
pub struct KnockbackData {
    pub impulse: f32,
}

impl Default for KnockbackData {
    fn default() -> Self {
        Self { impulse: 1.0 }
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