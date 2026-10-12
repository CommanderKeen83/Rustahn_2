pub mod gobbler;

pub fn update(
    world: &mut crate::ecs::World,
    player_id: Option<crate::types::EntityID>,
    camera: &crate::camera::GameCamera,
    dt: f32,
) {
    gobbler::update(world, player_id, camera, dt);
}
