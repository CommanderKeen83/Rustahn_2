use crate::prelude::*;
use sola_raylib::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl PlayerInput {
    pub fn from_raylib(rl: &RaylibHandle) -> Self {
        Self {
            up: rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W),
            down: rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S),
            left: rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A),
            right: rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D),
        }
    }
}

/// Updates player-controlled entities based on input:
/// - Arrow-Key Up applies upward thrust (counteracting gravity and lifting slowly).
/// - Arrow-Key Down applies downward thrust (descending faster).
/// - When no vertical keys are pressed, gravity pulls the aircraft down slowly.
pub fn update(world: &mut World, input: &PlayerInput, dt: f32) {
    if dt <= 0.0 {
        return;
    }

    for i in 0..world.next_entity_id {
        if !world.alive[i] || world.tags[i] != EntityType::Player {
            continue;
        }

        let movement = match &world.movement_data[i] {
            Some(m) => m,
            None => continue,
        };

        if input.up {
            // Apply upward thrust (counteracts gravity and accelerates upward)
            world.velocities[i].y -= movement.thrust_speed * dt;
        }

        if input.down {
            // Apply downward thrust (accelerates downward faster)
            world.velocities[i].y += movement.descent_speed * dt;
        }
    }
}

pub fn update_from_raylib(world: &mut World, rl: &RaylibHandle, dt: f32) {
    let input = PlayerInput::from_raylib(rl);
    update(world, &input, dt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_up_lifts_player() {
        let mut world = World::new();
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_movement(MovementData::new(
                player_stats::LIFT_THRUST,
                player_stats::MAX_SPEED,
                player_stats::ROTATION_SPEED,
            ))
            .with_physics(PhysicsData::dynamic())
            .build();

        let input = PlayerInput {
            up: true,
            down: false,
            left: false,
            right: false,
        };

        // Run 1 second of input and physics
        for _ in 0..60 {
            update(&mut world, &input, 1.0 / 60.0);
            crate::systems::physics::update(&mut world, 1.0 / 60.0);
        }

        // Upward thrust (180.0) overcomes gravity (80.0), resulting in negative velocity.y (upward)
        assert!(
            world.velocities[player.0].y < -20.0,
            "Player should have negative vertical velocity (moving up), but got {}",
            world.velocities[player.0].y
        );
        assert!(
            world.positions[player.0].y < 100.0,
            "Player position should have lifted above starting y=100, but was {}",
            world.positions[player.0].y
        );
    }

    #[test]
    fn test_input_down_descends_faster_than_gravity() {
        let mut world_gravity_only = World::new();
        let p_gravity = world_gravity_only
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_movement(MovementData::new(
                player_stats::LIFT_THRUST,
                player_stats::MAX_SPEED,
                player_stats::ROTATION_SPEED,
            ))
            .with_physics(PhysicsData::dynamic())
            .build();

        let mut world_down = World::new();
        let p_down = world_down
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_movement(MovementData::new(
                player_stats::LIFT_THRUST,
                player_stats::MAX_SPEED,
                player_stats::ROTATION_SPEED,
            ))
            .with_physics(PhysicsData::dynamic())
            .build();

        let no_input = PlayerInput::default();
        let down_input = PlayerInput {
            down: true,
            ..Default::default()
        };

        // Run 1 second
        for _ in 0..60 {
            update(&mut world_gravity_only, &no_input, 1.0 / 60.0);
            crate::systems::physics::update(&mut world_gravity_only, 1.0 / 60.0);

            update(&mut world_down, &down_input, 1.0 / 60.0);
            crate::systems::physics::update(&mut world_down, 1.0 / 60.0);
        }

        let vel_gravity = world_gravity_only.velocities[p_gravity.0].y;
        let vel_down = world_down.velocities[p_down.0].y;

        // Down input should produce significantly faster downward velocity than gravity alone
        assert!(
            vel_down > vel_gravity * 2.0,
            "Down input velocity ({}) should be more than 2x gravity velocity ({})",
            vel_down,
            vel_gravity
        );
        assert!(world_down.positions[p_down.0].y > world_gravity_only.positions[p_gravity.0].y);
    }
}
