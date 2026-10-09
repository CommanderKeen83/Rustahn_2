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
/// - Arrow-Key Up sets vertical velocity to lift upward slowly (-thrust_speed).
/// - Arrow-Key Down sets vertical velocity to descend faster (+descent_speed).
/// - When neither vertical key is pressed, existing velocity is preserved so downstream
///   systems (such as physics applying gravity) can govern motion.
pub fn update(world: &mut World, input: &PlayerInput, _dt: f32) {
    for i in 0..world.next_entity_id {
        if !world.alive[i] || world.tags[i] != EntityType::Player {
            continue;
        }

        let movement = match &world.movement_data[i] {
            Some(m) => m,
            None => continue,
        };

        if input.up {
            world.velocities[i].y = -movement.thrust_speed;
        } else if input.down {
            world.velocities[i].y = movement.descent_speed;
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
    fn test_input_up_sets_upward_velocity() {
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
            .build();

        let input = PlayerInput {
            up: true,
            down: false,
            left: false,
            right: false,
        };

        update(&mut world, &input, 1.0 / 60.0);

        assert_eq!(world.velocities[player.0].y, -player_stats::LIFT_THRUST);
    }

    #[test]
    fn test_input_down_sets_downward_velocity() {
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
            .build();

        let input = PlayerInput {
            up: false,
            down: true,
            left: false,
            right: false,
        };

        update(&mut world, &input, 1.0 / 60.0);

        assert_eq!(world.velocities[player.0].y, player_stats::DESCENT_THRUST);
    }

    #[test]
    fn test_no_input_preserves_existing_velocity() {
        let mut world = World::new();
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 15.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_movement(MovementData::new(
                player_stats::LIFT_THRUST,
                player_stats::MAX_SPEED,
                player_stats::ROTATION_SPEED,
            ))
            .build();

        let input = PlayerInput::default();

        update(&mut world, &input, 1.0 / 60.0);

        // Input system does not overwrite velocity when no key is pressed
        assert_eq!(world.velocities[player.0].y, 15.0);
    }
}
