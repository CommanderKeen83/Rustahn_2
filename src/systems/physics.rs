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

            // Apply gravity: World::DRAG is the global baseline downward pull, multiplied by entity gravity_scale
            world.velocities[i].y += World::DRAG * physics.gravity_scale * dt;

            // Apply linear drag (exponential decay for framerate independence)
            if physics.drag > 0.0 {
                let drag_factor = (-physics.drag * dt).exp();
                world.velocities[i].x *= drag_factor;
                world.velocities[i].y *= drag_factor;
            }

            // Cap downward velocity if max_fall_speed is specified so light objects stay floaty
            if let Some(max_fall) = physics.max_fall_speed {
                let cap = max_fall.abs();
                if world.velocities[i].y > cap {
                    world.velocities[i].y = cap;
                }
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

        // vel_y = 80 * 1.0 = 80, pos_y = 100 + 80 = 180
        assert!((world.velocities[entity.0].y - World::DRAG).abs() < 1e-4);
        assert!((world.positions[entity.0].y - (100.0 + World::DRAG)).abs() < 1e-4);
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

    #[test]
    fn test_different_gravity_scale_reach_ground_at_visibly_different_times() {
        let mut world = World::new();

        // Solid ground at y = 100.0, height 40.0 (top edge at 80.0)
        let _ground = world
            .spawn(
                EntityType::Ground,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(300.0, 40.0))
            .with_physics(PhysicsData::static_body())
            .build();

        // Heavy object (gravity_scale = 1.5): drops rapidly
        let heavy = world
            .spawn(
                EntityType::Bomb,
                Vector2::new(50.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Bomb,
            )
            .with_collider(Collider::new(10.0, 10.0))
            .with_gravity_scale(1.5)
            .build();

        // Normal object (gravity_scale = 1.0): default fall rate
        let normal = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::new(10.0, 10.0))
            .with_gravity_scale(1.0)
            .build();

        // Light object (gravity_scale = 0.2): sinks slowly
        let light = world
            .spawn(
                EntityType::Rabbit,
                Vector2::new(150.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Rabbit,
            )
            .with_collider(Collider::new(10.0, 10.0))
            .with_gravity_scale(0.2)
            .build();

        let dt = 1.0 / 60.0;
        let mut heavy_landed_frame = None;
        let mut normal_landed_frame = None;
        let mut light_landed_frame = None;

        // Simulate up to 600 frames (10 seconds)
        // Resting y on top of ground (80.0 - 5.0) is 75.0
        for frame in 1..=600 {
            update(&mut world, dt);
            crate::systems::collision::update(&mut world, dt);

            if heavy_landed_frame.is_none() && world.positions[heavy.0].y >= 74.9 {
                heavy_landed_frame = Some(frame);
            }
            if normal_landed_frame.is_none() && world.positions[normal.0].y >= 74.9 {
                normal_landed_frame = Some(frame);
            }
            if light_landed_frame.is_none() && world.positions[light.0].y >= 74.9 {
                light_landed_frame = Some(frame);
            }

            if heavy_landed_frame.is_some() && normal_landed_frame.is_some() && light_landed_frame.is_some() {
                break;
            }
        }

        let heavy_frame = heavy_landed_frame.expect("Heavy object should reach ground");
        let normal_frame = normal_landed_frame.expect("Normal object should reach ground");
        let light_frame = light_landed_frame.expect("Light object should reach ground");

        // Heavy object must hit the ground noticeably faster than normal, which hits faster than light
        assert!(
            heavy_frame < normal_frame,
            "Heavy object ({heavy_frame} frames) must land earlier than normal object ({normal_frame} frames)"
        );
        assert!(
            normal_frame < light_frame,
            "Normal object ({normal_frame} frames) must land earlier than light object ({light_frame} frames)"
        );

        // Visibly different times: heavy hits in ~1.2s, light takes > 3.0s (more than 2x the time)
        assert!(
            light_frame > heavy_frame * 2,
            "Light object ({light_frame} frames) should take over 2x longer than heavy object ({heavy_frame} frames)"
        );
    }

    #[test]
    fn test_entities_without_new_value_behave_identically_to_before() {
        let mut world_default = World::new();
        let e_default = world_default
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_physics(PhysicsData::dynamic())
            .build();

        // Default entity should have gravity_scale 1.0 and None for max_fall_speed
        let phys = world_default.physics_data[e_default.0].as_ref().unwrap();
        assert_eq!(phys.gravity_scale, 1.0);
        assert_eq!(phys.max_fall_speed, None);

        let dt = 1.0 / 60.0;
        for _ in 0..60 {
            update(&mut world_default, dt);
        }

        // Behavior should match exact integration of World::DRAG (80.0) with default drag (3.0)
        assert!(world_default.velocities[e_default.0].y > 0.0);
        assert!(world_default.positions[e_default.0].y > 100.0);
    }

    #[test]
    fn test_max_fall_speed_caps_downward_velocity() {
        let mut world = World::new();

        let cap_speed = 25.0;
        let floaty = world
            .spawn(
                EntityType::Rabbit,
                Vector2::new(50.0, 0.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Rabbit,
            )
            .with_physics(
                PhysicsData::dynamic()
                    .with_gravity_scale(1.5)
                    .with_max_fall_speed(cap_speed),
            )
            .build();

        let uncapped = world
            .spawn(
                EntityType::Bomb,
                Vector2::new(100.0, 0.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Bomb,
            )
            .with_physics(PhysicsData::dynamic().with_gravity_scale(1.5))
            .build();

        let dt = 1.0 / 60.0;
        // Simulate 3 seconds
        for _ in 0..180 {
            update(&mut world, dt);
            // Floaty entity downward velocity must NEVER exceed cap_speed
            assert!(
                world.velocities[floaty.0].y <= cap_speed + 1e-5,
                "Floaty velocity ({}) exceeded max fall speed ({cap_speed})",
                world.velocities[floaty.0].y
            );
        }

        // Floaty object reached and stayed at cap
        assert!((world.velocities[floaty.0].y - cap_speed).abs() < 1e-4);

        // Uncapped object has accelerated to a much higher velocity
        assert!(
            world.velocities[uncapped.0].y > cap_speed,
            "Uncapped velocity ({}) should exceed floaty capped velocity ({cap_speed})",
            world.velocities[uncapped.0].y
        );

        // Upward velocity is not capped by max_fall_speed
        world.velocities[floaty.0].y = -100.0;
        update(&mut world, 0.01);
        assert!(
            world.velocities[floaty.0].y < 0.0,
            "Upward velocity should not be clamped to max_fall_speed"
        );
    }

    #[test]
    fn test_physics_presets_and_config_tuning() {
        // Light preset defaults
        let light = PhysicsData::light();
        assert_eq!(light.gravity_scale, 0.2);
        assert_eq!(light.max_fall_speed, Some(25.0));

        // Normal preset defaults
        let normal = PhysicsData::normal();
        assert_eq!(normal.gravity_scale, 1.0);
        assert_eq!(normal.max_fall_speed, None);

        // Heavy preset defaults
        let heavy = PhysicsData::heavy();
        assert_eq!(heavy.gravity_scale, 1.5);
        assert_eq!(heavy.max_fall_speed, None);

        // Config file string parsing for data-driven tuning without code changes
        let config_text = r#"
            # Game balance tuning
            light.gravity_scale = 0.15
            light.max_fall_speed = 18.0
            normal.gravity_scale = 1.1
            heavy.gravity_scale = 2.2
            heavy.max_fall_speed = 90.0

            # Entity type overrides
            Rabbit.gravity_scale = 0.1
            Rabbit.max_fall_speed = 12.0
            Bomb.gravity_scale = 3.0
        "#;

        let cfg = PhysicsConfig::parse_str(config_text);
        assert_eq!(cfg.light_gravity_scale, 0.15);
        assert_eq!(cfg.light_max_fall_speed, Some(18.0));
        assert_eq!(cfg.normal_gravity_scale, 1.1);
        assert_eq!(cfg.heavy_gravity_scale, 2.2);
        assert_eq!(cfg.heavy_max_fall_speed, Some(90.0));

        let rabbit_phys = cfg.get_for_type(EntityType::Rabbit).unwrap();
        assert_eq!(rabbit_phys.gravity_scale, 0.1);
        assert_eq!(rabbit_phys.max_fall_speed, Some(12.0));

        let bomb_phys = cfg.get_for_type(EntityType::Bomb).unwrap();
        assert_eq!(bomb_phys.gravity_scale, 3.0);
    }

    #[test]
    fn test_entity_builder_convenience_methods() {
        let mut world = World::new();

        let entity = world
            .spawn(
                EntityType::Rabbit,
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Rabbit,
            )
            .with_gravity_scale(0.3)
            .with_max_fall_speed(35.0)
            .build();

        let phys = world.physics_data[entity.0].as_ref().unwrap();
        assert_eq!(phys.gravity_scale, 0.3);
        assert_eq!(phys.max_fall_speed, Some(35.0));

        let preset_entity = world
            .spawn(
                EntityType::Bomb,
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Bomb,
            )
            .with_physics_preset(PhysicsPreset::Heavy)
            .build();

        let phys_preset = world.physics_data[preset_entity.0].as_ref().unwrap();
        assert_eq!(phys_preset.gravity_scale, 1.5);
    }
}
