use crate::ecs;

pub struct Gamestate{
    world: ecs::World,

}
impl Gamestate {
    pub fn new() -> Self {
        Self {
            world: ecs::World::new(),
        }
    }
}