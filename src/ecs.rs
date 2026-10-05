use crate::prelude::*;
use sola_raylib::prelude::*;
pub struct World {
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
    #[must_use = "call .build() to finalize the entity — dropping the builder leaks the entity slot"]
    pub fn spawn(
        &mut self,
        tag: EntityType,
        position: Vector2,
        velocity: Vector2,
        rotation: f32,
        alive: bool,
        sprite_id: SpriteID,
    ) -> Entitybuilder<'_> {
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        self.tags.push(tag);
        self.positions.push(position);
        self.velocities.push(velocity);
        self.rotations.push(rotation);
        self.alive.push(alive);
        self.sprite_ids.push(sprite_id);

        self.animation_data.push(None);
        self.movement_data.push(None);
        self.mortality.push(None);
        self.weapon_set.push(None);
        self.physics_data.push(None);

        Entitybuilder { world: self, id }
    }
}
#[must_use = "You must call `.build()` to finalize the entity and get its EntityId!"]
pub struct Entitybuilder<'a> {
    world: &'a mut World,
    id: usize,
}
impl<'a> Entitybuilder<'a> {
    pub fn with_physics(self, physics_data: PhysicsData) -> Self {
        self.world.physics_data[self.id] = Some(physics_data);
        self
    }
    pub fn with_movement(self, movement_data: MovementData) -> Self {
        self.world.movement_data[self.id] = Some(movement_data);
        self
    }
    pub fn with_animation(self, animation_data: AnimationData) -> Self {
        self.world.animation_data[self.id] = Some(animation_data);
        self
    }
    pub fn with_weaponset(self, weapon_set: WeaponSet) -> Self {
        self.world.weapon_set[self.id] = Some(weapon_set);
        self
    }

    pub fn with_mortality(self, mortality: Mortality) -> Self {
        self.world.mortality[self.id] = Some(mortality);
        self
    }
    pub fn build(self) -> EntityID {
        EntityID(self.id)
    }
}
