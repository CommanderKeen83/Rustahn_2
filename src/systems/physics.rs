use crate::prelude::*;

/// Updates entity physics: applies gravity and drag, and integrates velocity into position.
pub fn update(world: &mut World, dt: f32) {
    if dt <= 0.0 {
        return;
    }

    for i in 0..world.next_entity_id {
        if !world.alive[i] {
            continue;
        }

        if let Some(physics) = &world.physics_data[i] {
            if physics.is_static {
                continue;
            }

            // Apply gravity
            world.velocities[i].y += world_stats::GRAVITY * physics.gravity_scale * dt;

            // Apply linear drag (exponential decay for framerate independence)
            if physics.drag > 0.0 {
                let drag_factor = (-physics.drag * dt).exp();
                world.velocities[i].x *= drag_factor;
                world.velocities[i].y *= drag_factor;
            }

            // Integrate velocity into position
            world.positions[i].x += world.velocities[i].x * dt;
            world.positions[i].y += world.velocities[i].y * dt;
        } else if world.velocities[i].x != 0.0 || world.velocities[i].y != 0.0 {
            // Kinematic movement for entities without PhysicsData
            world.positions[i].x += world.velocities[i].x * dt;
            world.positions[i].y += world.velocities[i].y * dt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sola_raylib::prelude::Vector2;

    #[test]
    fn test_physics_gravity_acceleration() {
        let mut world = World::new();
        let entity = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_physics(PhysicsData::dynamic().with_drag(0.0))
            .build();

        update(&mut world, 1.0);

        // vel_y = 500 * 1.0 = 500, pos_y = 100 + 500 = 600
        assert!((world.velocities[entity.0].y - world_stats::GRAVITY).abs() < 1e-4);
        assert!((world.positions[entity.0].y - (100.0 + world_stats::GRAVITY)).abs() < 1e-4);
    }

    #[test]
    fn test_static_body_does_not_move() {
        let mut world = World::new();
        let entity = world
            .spawn(
                EntityType::Ground,
                Vector2::new(100.0, 200.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_physics(PhysicsData::static_body())
            .build();

        update(&mut world, 1.0);

        assert_eq!(world.positions[entity.0].y, 200.0);
        assert_eq!(world.velocities[entity.0].y, 0.0);
    }

    #[test]
    fn test_kinematic_velocity_without_gravity() {
        let mut world = World::new();
        let entity = world
            .spawn(
                EntityType::Bullet,
                Vector2::new(50.0, 50.0),
                Vector2::new(100.0, -50.0),
                0.0,
                true,
                SpriteID::Bullet,
            )
            .build();

        update(&mut world, 0.5);

        assert_eq!(world.positions[entity.0].x, 100.0);
        assert_eq!(world.positions[entity.0].y, 25.0);
        assert_eq!(world.velocities[entity.0].y, -50.0);
    }
}
