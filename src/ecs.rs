use sola_raylib::prelude::*;
use crate::prelude::*;
pub struct World{
    pub next_entity_id: usize,
    pub tags: Vec<EntityType>,
    pub positions: Vec<Vector2>,
    pub velocities: Vec<Vector2>,
    pub rotations: Vec<f32>,
    pub alive: Vec<bool>,
    pub sprite_ids: Vec<SpriteID>,

    // --- OPTIONAL COMPONENTS (Sparse Arrays) ---
    // If index 5 is None, Entity 5 does not have this component.
    pub animation_data: Vec<Option<AnimationData>>,
    pub movement_data: Vec<Option<MovementData>>,
    pub mortality: Vec<Option<Mortality>>,
    pub weapon_set: Vec<Option<WeaponSet>>,
    pub physics_data: Vec<Option<PhysicsData>>,
}
impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            tags: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            rotations: Vec::new(),
            alive: Vec::new(),
            sprite_ids: Vec::new(),
            animation_data: Vec::new(),
            movement_data: Vec::new(),
            mortality: Vec::new(),
            weapon_set: Vec::new(),
            physics_data: Vec::new(),
        }
    }
    pub fn spawn(&mut self) -> Entitybuilder{
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        Entitybuilder{ world: self, id }
    }
}
pub struct Entitybuilder<'a>{
    world:&'a mut World,
    id: usize,
}
impl<'a> Entitybuilder<'a> {
    pub fn with_physics(self, physics_data: PhysicsData ) -> Self {
        self.world.physics_data[self.id] = Some(physics_data);
        self
    }
}