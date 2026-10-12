use crate::camera::GameCamera;
use crate::components::{Collider, GobblerData};
use crate::ecs::World;
use crate::types::{EntityID, EntityType};
use sola_raylib::prelude::*;

pub const GOBBLER_WIDTH: f32 = GobblerData::WIDTH;
pub const GOBBLER_HEIGHT: f32 = GobblerData::HEIGHT;
pub const DEFAULT_GOBBLER_SPEED: f32 = GobblerData::DEFAULT_SPEED;

/// Checks whether an entity's axis-aligned bounding box intersects the camera viewport.
pub fn is_inside_camera(camera: &GameCamera, pos: Vector2, collider: Option<&Collider>) -> bool {
    let cam_min_x = camera.x();
    let cam_max_x = camera.x() + camera.viewport_width;
    let cam_min_y = camera.y();
    let cam_max_y = camera.y() + camera.viewport_height;

    let (min_x, min_y, max_x, max_y) = if let Some(col) = collider {
        col.aabb(pos)
    } else {
        let hw = GOBBLER_WIDTH * 0.5;
        let hh = GOBBLER_HEIGHT * 0.5;
        (pos.x - hw, pos.y - hh, pos.x + hw, pos.y + hh)
    };

    max_x >= cam_min_x && min_x <= cam_max_x && max_y >= cam_min_y && min_y <= cam_max_y
}

/// Updates Gobbler enemy behaviour:
/// - Inactive Gobblers remain idle (x velocity = 0) until their bounding box enters the camera view.
/// - Once inside the camera view, a Gobbler activates and starts moving horizontally (x-wise)
///   towards the player.
/// - Once active, they continue to track and move towards the player's X coordinate.
/// - Vertical movement (gravity, falling off platforms, landing) is handled by the physics/collision systems.
pub fn update(world: &mut World, player_id: Option<EntityID>, camera: &GameCamera, _dt: f32) {
    let target_pos = player_id
        .filter(|id| id.0 < world.next_entity_id && world.alive[id.0])
        .map(|id| world.positions[id.0])
        .or_else(|| {
            (0..world.next_entity_id)
                .find(|&i| world.alive[i] && world.tags[i] == EntityType::Player)
                .map(|i| world.positions[i])
        });

    for i in 0..world.next_entity_id {
        if !world.alive[i] || world.tags[i] != EntityType::Gobbler {
            continue;
        }

        let pos = world.positions[i];
        let col = world.colliders[i];

        let mut data = world.gobbler_data[i].unwrap_or_default();

        // Activate once inside camera viewport
        if !data.is_active && is_inside_camera(camera, pos, col.as_ref()) {
            data.is_active = true;
        }

        // Active Gobblers always move horizontally towards the player
        if data.is_active {
            if let Some(target) = target_pos {
                let dx = target.x - pos.x;
                if dx > 0.5 {
                    world.velocities[i].x = data.speed;
                } else if dx < -0.5 {
                    world.velocities[i].x = -data.speed;
                } else {
                    world.velocities[i].x = 0.0;
                }
            } else {
                world.velocities[i].x = 0.0;
            }
        } else {
            world.velocities[i].x = 0.0;
        }

        world.gobbler_data[i] = Some(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::PhysicsPreset;

    #[test]
    fn test_gobbler_remains_idle_outside_camera_view() {
        let mut world = World::new();
        let camera = GameCamera::at(0.0, 0.0); // viewport 0..320, 0..180

        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(50.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Player,
            )
            .build();

        // Gobbler spawned far off-screen at x = 500
        let gobbler = world
            .spawn(
                EntityType::Gobbler,
                Vector2::new(500.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Gobbler,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .with_physics_preset(PhysicsPreset::Normal)
            .with_gobbler(DEFAULT_GOBBLER_SPEED)
            .build();

        update(&mut world, Some(player), &camera, 0.016);

        assert!(!world.gobbler_data[gobbler.0].unwrap().is_active);
        assert_eq!(world.velocities[gobbler.0].x, 0.0);
    }

    #[test]
    fn test_gobbler_activates_and_moves_toward_player_when_in_camera_view() {
        let mut world = World::new();
        let camera = GameCamera::at(0.0, 0.0); // viewport 0..320, 0..180

        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(50.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Player,
            )
            .build();

        // Gobbler inside viewport at x = 200 (to the right of player)
        let gobbler = world
            .spawn(
                EntityType::Gobbler,
                Vector2::new(200.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Gobbler,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .with_physics_preset(PhysicsPreset::Normal)
            .with_gobbler(DEFAULT_GOBBLER_SPEED)
            .build();

        update(&mut world, Some(player), &camera, 0.016);

        // Gobbler should activate and move left towards player (x = 50 < 200)
        assert!(world.gobbler_data[gobbler.0].unwrap().is_active);
        assert_eq!(world.velocities[gobbler.0].x, -DEFAULT_GOBBLER_SPEED);
    }

    #[test]
    fn test_gobbler_moves_right_when_player_is_to_the_right() {
        let mut world = World::new();
        let camera = GameCamera::at(0.0, 0.0);

        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(250.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Player,
            )
            .build();

        // Gobbler inside viewport at x = 100 (to the left of player)
        let gobbler = world
            .spawn(
                EntityType::Gobbler,
                Vector2::new(100.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Gobbler,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .with_physics_preset(PhysicsPreset::Normal)
            .with_gobbler(DEFAULT_GOBBLER_SPEED)
            .build();

        update(&mut world, Some(player), &camera, 0.016);

        assert!(world.gobbler_data[gobbler.0].unwrap().is_active);
        assert_eq!(world.velocities[gobbler.0].x, DEFAULT_GOBBLER_SPEED);
    }

    #[test]
    fn test_gobbler_reverses_direction_when_player_passes() {
        let mut world = World::new();
        let camera = GameCamera::at(0.0, 0.0);

        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(50.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Player,
            )
            .build();

        let gobbler = world
            .spawn(
                EntityType::Gobbler,
                Vector2::new(150.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                crate::types::SpriteID::Gobbler,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .build();

        // First frame: player is to the left (50 < 150) -> moves left (-speed)
        update(&mut world, Some(player), &camera, 0.016);
        assert_eq!(world.velocities[gobbler.0].x, -DEFAULT_GOBBLER_SPEED);

        // Player jumps over gobbler to the right (x = 250)
        world.positions[player.0].x = 250.0;
        update(&mut world, Some(player), &camera, 0.016);
        assert_eq!(world.velocities[gobbler.0].x, DEFAULT_GOBBLER_SPEED);
    }
}
