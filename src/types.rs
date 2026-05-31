
#[derive(Debug, Clone, PartialEq)]
pub enum StateType{
    Menu,
    GameIntro,
    Game,
    Options,
    Pause,
    GameOver,
}
pub struct EntityID(pub usize);


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

