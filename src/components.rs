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

