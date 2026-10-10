use crate::prelude::*;
use crate::systems::collision;
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
/// - Arrow-Key Up sets vertical velocity to lift upward slowly (-thrust_speed),
///   and tilts the spaceship upward on its right side (-tilt_angle).
/// - Arrow-Key Down sets vertical velocity to descend faster (+descent_speed),
///   and tilts the spaceship downward on its right side (+tilt_angle).
/// - When neither vertical key is pressed (or both are pressed), existing velocity
///   is preserved so downstream systems (such as physics applying gravity) can govern
///   motion, and rotation smoothly returns to initial horizontally balanced mode (0.0).
pub fn update(world: &mut World, input: &PlayerInput, dt: f32) {
    for i in 0..world.next_entity_id {
        if !world.alive[i] || world.tags[i] != EntityType::Player {
            continue;
        }

        let movement = match &world.movement_data[i] {
            Some(m) => m,
            None => continue,
        };

        let blocked_down = collision::is_blocked_down(world, i);

        if input.up {
            world.velocities[i].y = -movement.thrust_speed;
        } else if input.down {
            if !blocked_down {
                world.velocities[i].y = movement.descent_speed;
            } else if world.velocities[i].y > 0.0 {
                world.velocities[i].y = 0.0;
            }
        }

        // Determine target tilt rotation:
        // - Arrow-Up: spaceship pitches upwards on its right side (-tilt_angle)
        // - Arrow-Down: spaceship tip pitches downwards (+tilt_angle), unless blocked by collision below (e.g. resting on bottom)
        // - Neither direction / balanced: return to horizontal (0.0)
        let target_rotation = if input.up && !input.down {
            -movement.tilt_angle
        } else if input.down && !input.up && !blocked_down {
            movement.tilt_angle
        } else {
            0.0
        };

        if blocked_down && world.rotations[i] > 0.0 {
            world.rotations[i] = 0.0;
        }

        if dt > 0.0 && movement.rotation_speed > 0.0 {
            let diff = target_rotation - world.rotations[i];
            let max_step = movement.rotation_speed * dt;
            if diff.abs() <= max_step {
                world.rotations[i] = target_rotation;
            } else {
                world.rotations[i] += diff.signum() * max_step;
            }
        } else if movement.rotation_speed == 0.0 {
            world.rotations[i] = target_rotation;
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

    #[test]
    fn test_input_up_tilts_upward() {
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

        // 1 frame of up input starts tilting upward (negative rotation in Raylib coordinate system)
        update(&mut world, &input, 1.0 / 60.0);
        assert!(
            world.rotations[player.0] < 0.0,
            "Expected negative rotation (upward tilt), got {}",
            world.rotations[player.0]
        );

        // Multiple frames reach full tilt angle (-TILT_ANGLE)
        for _ in 0..10 {
            update(&mut world, &input, 1.0 / 60.0);
        }
        assert_eq!(world.rotations[player.0], -player_stats::TILT_ANGLE);
    }

    #[test]
    fn test_input_down_tilts_downward() {
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

        // 1 frame of down input starts tilting downward (positive rotation in Raylib coordinate system)
        update(&mut world, &input, 1.0 / 60.0);
        assert!(
            world.rotations[player.0] > 0.0,
            "Expected positive rotation (downward tilt), got {}",
            world.rotations[player.0]
        );

        // Multiple frames reach full tilt angle (+TILT_ANGLE)
        for _ in 0..10 {
            update(&mut world, &input, 1.0 / 60.0);
        }
        assert_eq!(world.rotations[player.0], player_stats::TILT_ANGLE);
    }

    #[test]
    fn test_no_input_returns_to_balanced_mode() {
        let mut world = World::new();
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                -player_stats::TILT_ANGLE,
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

        // Starts at -TILT_ANGLE
        assert_eq!(world.rotations[player.0], -player_stats::TILT_ANGLE);

        // Multiple frames of no input returns smoothly to 0.0 (horizontally balanced mode)
        for _ in 0..10 {
            update(&mut world, &input, 1.0 / 60.0);
        }
        assert_eq!(world.rotations[player.0], 0.0);
    }

    #[test]
    fn test_opposing_input_returns_to_balanced_mode() {
        let mut world = World::new();
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                player_stats::TILT_ANGLE,
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
            down: true,
            left: false,
            right: false,
        };

        for _ in 0..10 {
            update(&mut world, &input, 1.0 / 60.0);
        }
        assert_eq!(world.rotations[player.0], 0.0);
    }

    #[test]
    fn test_input_down_blocked_by_collision_does_not_tilt_down() {
        let mut world = World::new();

        // Solid ground at y = 120.0, height 40.0 (top edge at 100.0)
        let _ground = world
            .spawn(
                EntityType::Ground,
                Vector2::new(100.0, 120.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(100.0, 40.0))
            .with_physics(PhysicsData::static_body())
            .build();

        // Player resting on ground: center at y = 85.0, height 30.0 (bottom edge at 100.0)
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 85.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::new(40.0, 30.0))
            .with_movement(MovementData::new(
                player_stats::LIFT_THRUST,
                player_stats::MAX_SPEED,
                player_stats::ROTATION_SPEED,
            ))
            .build();

        let down_input = PlayerInput {
            down: true,
            ..Default::default()
        };

        // When pressing Arrow-Down on ground, tilt slightly down should NOT occur
        for _ in 0..10 {
            update(&mut world, &down_input, 1.0 / 60.0);
        }

        assert_eq!(
            world.rotations[player.0], 0.0,
            "Downward tilt must not occur when blocked by collision below"
        );
        assert_eq!(world.velocities[player.0].y, 0.0);
    }
}
