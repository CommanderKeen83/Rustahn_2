use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionEvent {
    pub entity_a: EntityID,
    pub entity_b: EntityID,
}

/// Detects AABB collisions between active entities with colliders,
/// resolves solid collisions via minimum translation vector (MTV),
/// and returns a list of collision events.
pub fn update(world: &mut World, _dt: f32) -> Vec<CollisionEvent> {
    let mut events = Vec::new();
    let count = world.next_entity_id;

    for a in 0..count {
        if !world.alive[a] {
            continue;
        }
        let col_a = match world.colliders[a] {
            Some(c) => c,
            None => continue,
        };

        for b in (a + 1)..count {
            if !world.alive[b] {
                continue;
            }
            let col_b = match world.colliders[b] {
                Some(c) => c,
                None => continue,
            };

            let pos_a = world.positions[a];
            let pos_b = world.positions[b];

            let (a_min_x, a_min_y, a_max_x, a_max_y) = col_a.aabb(pos_a);
            let (b_min_x, b_min_y, b_max_x, b_max_y) = col_b.aabb(pos_b);

            let overlap_x = a_max_x.min(b_max_x) - a_min_x.max(b_min_x);
            let overlap_y = a_max_y.min(b_max_y) - a_min_y.max(b_min_y);

            if overlap_x > 0.0 && overlap_y > 0.0 {
                events.push(CollisionEvent {
                    entity_a: EntityID(a),
                    entity_b: EntityID(b),
                });

                if col_a.is_solid && col_b.is_solid {
                    let a_is_static = world.physics_data[a].as_ref().map_or(true, |p| p.is_static);
                    let b_is_static = world.physics_data[b].as_ref().map_or(true, |p| p.is_static);

                    if a_is_static && b_is_static {
                        continue;
                    }

                    let (a_weight, b_weight) = match (a_is_static, b_is_static) {
                        (false, true) => (1.0, 0.0),
                        (true, false) => (0.0, 1.0),
                        (false, false) => (0.5, 0.5),
                        (true, true) => (0.0, 0.0),
                    };

                    let center_a_x = pos_a.x + col_a.offset.x;
                    let center_a_y = pos_a.y + col_a.offset.y;
                    let center_b_x = pos_b.x + col_b.offset.x;
                    let center_b_y = pos_b.y + col_b.offset.y;

                    if overlap_y < overlap_x {
                        // Vertical resolution
                        if center_a_y < center_b_y {
                            // A is above B (e.g. entity landing on ground)
                            if a_weight > 0.0 {
                                world.positions[a].y -= overlap_y * a_weight;
                                if world.velocities[a].y > 0.0 {
                                    world.velocities[a].y = 0.0;
                                }
                                if world.rotations[a] > 0.0 {
                                    world.rotations[a] = 0.0;
                                }
                            }
                            if b_weight > 0.0 {
                                world.positions[b].y += overlap_y * b_weight;
                                if world.velocities[b].y < 0.0 {
                                    world.velocities[b].y = 0.0;
                                }
                            }
                        } else {
                            // A is below B
                            if a_weight > 0.0 {
                                world.positions[a].y += overlap_y * a_weight;
                                if world.velocities[a].y < 0.0 {
                                    world.velocities[a].y = 0.0;
                                }
                            }
                            if b_weight > 0.0 {
                                world.positions[b].y -= overlap_y * b_weight;
                                if world.velocities[b].y > 0.0 {
                                    world.velocities[b].y = 0.0;
                                }
                                if world.rotations[b] > 0.0 {
                                    world.rotations[b] = 0.0;
                                }
                            }
                        }
                    } else {
                        // Horizontal resolution
                        if center_a_x < center_b_x {
                            // A is left of B
                            if a_weight > 0.0 {
                                world.positions[a].x -= overlap_x * a_weight;
                                if world.velocities[a].x > 0.0 {
                                    world.velocities[a].x = 0.0;
                                }
                            }
                            if b_weight > 0.0 {
                                world.positions[b].x += overlap_x * b_weight;
                                if world.velocities[b].x < 0.0 {
                                    world.velocities[b].x = 0.0;
                                }
                            }
                        } else {
                            // A is right of B
                            if a_weight > 0.0 {
                                world.positions[a].x += overlap_x * a_weight;
                                if world.velocities[a].x < 0.0 {
                                    world.velocities[a].x = 0.0;
                                }
                            }
                            if b_weight > 0.0 {
                                world.positions[b].x -= overlap_x * b_weight;
                                if world.velocities[b].x > 0.0 {
                                    world.velocities[b].x = 0.0;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    events
}

/// Checks if an entity is in contact with or blocked by a solid collider directly below it.
pub fn is_blocked_down(world: &World, entity_index: usize) -> bool {
    if entity_index >= world.next_entity_id || !world.alive[entity_index] {
        return false;
    }

    let col_a = match world.colliders[entity_index] {
        Some(c) if c.is_solid => c,
        _ => return false,
    };

    let pos_a = world.positions[entity_index];
    let (a_min_x, a_min_y, a_max_x, a_max_y) = col_a.aabb(pos_a);

    let count = world.next_entity_id;
    for b in 0..count {
        if b == entity_index || !world.alive[b] {
            continue;
        }

        let col_b = match world.colliders[b] {
            Some(c) if c.is_solid => c,
            _ => continue,
        };

        let pos_b = world.positions[b];
        let (b_min_x, b_min_y, b_max_x, b_max_y) = col_b.aabb(pos_b);

        // Check horizontal overlap between entity a and solid collider b
        let overlap_x = a_max_x.min(b_max_x) - a_min_x.max(b_min_x);
        if overlap_x <= 0.0 {
            continue;
        }

        // Entity a must be above entity b
        let center_a_y = pos_a.y + col_a.offset.y;
        let center_b_y = pos_b.y + col_b.offset.y;
        if center_a_y >= center_b_y {
            continue;
        }

        // Check vertical contact: a's bottom is touching or penetrating b's top
        // (with a small 1.0 pixel tolerance to account for floating-point contact)
        let tolerance = 1.0;
        if a_max_y >= b_min_y - tolerance && a_min_y < b_max_y {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use sola_raylib::prelude::Vector2;

    #[test]
    fn test_collision_resolution_landing_on_ground() {
        let mut world = World::new();

        // Dynamic falling entity at (100, 100), 20x20
        // Bottom is at 100 + 10 = 110
        let falling = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 105.0),
                Vector2::new(0.0, 50.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .with_physics(PhysicsData::dynamic())
            .build();

        // Static ground at (100, 130), 100x40
        // Top is at 130 - 20 = 110.
        // Falling bottom was at 105 + 10 = 115, penetrating by 5px.
        let _ground = world
            .spawn(
                EntityType::Ground,
                Vector2::new(100.0, 130.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(100.0, 40.0))
            .with_physics(PhysicsData::static_body())
            .build();

        let events = update(&mut world, 0.016);

        assert_eq!(events.len(), 1);
        // Entity should be pushed up so its bottom is at 110.0 (center y = 100.0)
        assert!((world.positions[falling.0].y - 100.0).abs() < 1e-4);
        assert_eq!(world.velocities[falling.0].y, 0.0);
    }

    #[test]
    fn test_non_solid_trigger_does_not_push() {
        let mut world = World::new();

        let a = world
            .spawn(
                EntityType::Player,
                Vector2::new(50.0, 50.0),
                Vector2::new(10.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::sensor(20.0, 20.0))
            .build();

        let _b = world
            .spawn(
                EntityType::Gobbler,
                Vector2::new(55.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Gobbler,
            )
            .with_collider(Collider::sensor(20.0, 20.0))
            .build();

        let events = update(&mut world, 0.016);

        assert_eq!(events.len(), 1);
        assert_eq!(world.positions[a.0].x, 50.0);
    }

    #[test]
    fn test_is_blocked_down_detection() {
        let mut world = World::new();

        // Solid ground at y = 100.0, height 20.0 (top edge at 90.0)
        let _ground = world
            .spawn(
                EntityType::Ground,
                Vector2::new(50.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(100.0, 20.0))
            .with_physics(PhysicsData::static_body())
            .build();

        // Entity 1: resting on ground (bottom at 90.0)
        let resting = world
            .spawn(
                EntityType::Player,
                Vector2::new(50.0, 80.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .build();

        // Entity 2: in mid-air (bottom at 60.0)
        let airborne = world
            .spawn(
                EntityType::Player,
                Vector2::new(50.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .build();

        // Entity 3: to the side of ground horizontally (center x = 200.0)
        let side = world
            .spawn(
                EntityType::Player,
                Vector2::new(200.0, 80.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .build();

        assert!(is_blocked_down(&world, resting.0), "Resting entity must be blocked down");
        assert!(!is_blocked_down(&world, airborne.0), "Airborne entity must not be blocked down");
        assert!(!is_blocked_down(&world, side.0), "Entity not above ground must not be blocked down");
    }

    #[test]
    fn test_collision_landing_levels_downward_tilt() {
        let mut world = World::new();

        // Dynamic falling entity landing on ground with downward tilt (+12.0)
        let falling = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 105.0),
                Vector2::new(0.0, 50.0),
                12.0,
                true,
                SpriteID::Player,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .with_physics(PhysicsData::dynamic())
            .build();

        // Static ground at (100, 130), 100x40 (top at 110.0)
        let _ground = world
            .spawn(
                EntityType::Ground,
                Vector2::new(100.0, 130.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(100.0, 40.0))
            .with_physics(PhysicsData::static_body())
            .build();

        assert_eq!(world.rotations[falling.0], 12.0);

        let _ = update(&mut world, 0.016);

        // Landing on ground must resolve position and reset downward tilt to 0.0
        assert_eq!(world.rotations[falling.0], 0.0);
        assert_eq!(world.velocities[falling.0].y, 0.0);
    }
}
