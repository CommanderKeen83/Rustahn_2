#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Rabbit,
    Gobbler,
    EvilEye,
    Fireworm,
    Player,
    Bullet,
    Fireball,
    Bomb,
    Ground,
    Obstacle,
}

impl EntityType {
    pub fn is_enemy(&self) -> bool {
        matches!(self, EntityType::Gobbler | EntityType::EvilEye | EntityType::Fireworm)
    }
    pub fn is_weapon(&self) -> bool {
        matches!(self, EntityType::Bullet | EntityType::Fireball | EntityType::Bomb)
    }
    pub fn is_solid_obstacle(&self) -> bool {
        matches!(self, EntityType::Ground | EntityType::Obstacle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpriteID {
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
    GameIntroWindowLamp,
    LevelOneBackground,
    ActOneForeground,
    Platforms,
    None,
}
