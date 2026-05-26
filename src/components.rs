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
    Menu_Background,
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

}

impl Entity{
    pub fn new(tag: EntityType, pos: Vector2, rot: f32, sprite_id: SpriteID) -> Entity {
        Entity{
            tag,
            pos,
            vel: Vector2::new(0.0, 0.0 ),
            rot: 0.0,
            alive: true,
            sprite_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MovementData{

}

#[derive(Debug, Clone)]
pub struct AnimationData{

}

#[derive(Debug, Clone)]
pub struct Mortality{

}

#[derive(Debug, Clone, PartialEq)]
pub enum WeaponType{
    Player_Bullet,
    Enemy_Bullet,
    Enemy_Fireball,
    Enemy_FreezeBeam,

}
#[derive(Debug, Clone)]
pub struct WeaponSet{
    pub cooldown: f32,
    pub timer: f32,
    pub weapon: WeaponType
}