use crate::camera::GameCamera;
use crate::ecs::World;
use crate::window::VIRTUAL_SCALE;
use sola_raylib::prelude::*;

/// Lifecycle and physical states of the grappling hook.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HookState {
    /// No hook is currently deployed.
    Inactive,
    /// Hook is anchored to a solid obstacle (wall or platform), forming a taut rope.
    Attached {
        /// World coordinates where the hook stuck to the obstacle surface.
        anchor: Vector2,
        /// Current length of the rope connecting anchor to player.
        rope_length: f32,
    },
    /// Hook missed any solid geometry; displays a brief fading line extending to max reach.
    Missed {
        /// World coordinates of the maximum reach tip.
        tip: Vector2,
        /// Remaining display timer in seconds before disappearing.
        timer: f32,
    },
}

/// Result returned from updating the hook physics this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HookUpdateResult {
    /// True if the player pressed jump, causing the rope to dissolve and launching the player into a jump.
    pub dissolved_and_jumped: bool,
    /// True if the hook was attached during this frame.
    pub was_attached: bool,
}

/// Computes the intersection between a directed line segment from `start` to `end`
/// and an axis-aligned bounding box `rect`. Returns the closest intersection point and distance.
pub fn ray_segment_intersect_aabb(
    start: Vector2,
    end: Vector2,
    rect: Rectangle,
) -> Option<(Vector2, f32)> {
    let min_x = rect.x;
    let max_x = rect.x + rect.width;
    let min_y = rect.y;
    let max_y = rect.y + rect.height;

    let dx = end.x - start.x;
    let dy = end.y - start.y;

    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = 1.0;

    // Check X slab
    if dx.abs() < 1e-6 {
        if start.x < min_x || start.x > max_x {
            return None;
        }
    } else {
        let t1 = (min_x - start.x) / dx;
        let t2 = (max_x - start.x) / dx;
        let (tx_min, tx_max) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        t_min = t_min.max(tx_min);
        t_max = t_max.min(tx_max);
        if t_min > t_max {
            return None;
        }
    }

    // Check Y slab
    if dy.abs() < 1e-6 {
        if start.y < min_y || start.y > max_y {
            return None;
        }
    } else {
        let t1 = (min_y - start.y) / dy;
        let t2 = (max_y - start.y) / dy;
        let (ty_min, ty_max) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        t_min = t_min.max(ty_min);
        t_max = t_max.min(ty_max);
        if t_min > t_max {
            return None;
        }
    }

    if t_min <= 1.0 && t_max >= 0.0 {
        let t_hit = if t_min >= 0.0 { t_min } else { 0.0 };
        let hit_x = start.x + dx * t_hit;
        let hit_y = start.y + dy * t_hit;
        let dist = ((hit_x - start.x).powi(2) + (hit_y - start.y).powi(2)).sqrt();
        Some((Vector2::new(hit_x, hit_y), dist))
    } else {
        None
    }
}

/// Finds the closest solid obstacle hit by a ray segment in the world, excluding the given entity.
pub fn find_closest_obstacle_hit(
    start: Vector2,
    end: Vector2,
    world: &World,
    exclude_entity: usize,
) -> Option<(Vector2, f32)> {
    let mut closest: Option<(Vector2, f32)> = None;

    for i in 0..world.next_entity_id {
        if !world.alive[i] || i == exclude_entity {
            continue;
        }

        let col = match &world.colliders[i] {
            Some(c) if c.is_solid => c,
            _ => continue,
        };

        let bounds = col.bounds(world.positions[i]);
        if let Some((hit_pos, dist)) = ray_segment_intersect_aabb(start, end, bounds) {
            match closest {
                None => closest = Some((hit_pos, dist)),
                Some((_, best_dist)) if dist < best_dist => {
                    closest = Some((hit_pos, dist));
                }
                _ => {}
            }
        }
    }

    closest
}

/// Controller and physical simulation component for the grappling hook rope mechanic.
#[derive(Debug, Clone, PartialEq)]
pub struct GrapplingHook {
    pub state: HookState,
    /// Maximum throw length in virtual pixels (defaults to 30.0 px).
    pub max_length: f32,
    /// Rate at which player can pull themselves up on the rope in px/s.
    pub pull_speed: f32,
    /// Minimum rope length when pulled in completely.
    pub min_rope_length: f32,
    /// Tangential acceleration applied when pressing Arrow Left or Arrow Right to swing.
    pub swing_accel: f32,
    /// Maximum velocity cap when swinging to ensure stability.
    pub max_swing_speed: f32,
    /// Light air damping applied during swinging motion.
    pub swing_damping: f32,
}

impl GrapplingHook {
    /// Default maximum reach of the thrown hook in virtual pixels.
    pub const DEFAULT_MAX_LENGTH: f32 = 90.0;
    /// Default rope retraction speed when pressing Arrow Up.
    pub const DEFAULT_PULL_SPEED: f32 = 75.0;
    /// Minimum distance to anchor when reeled in.
    pub const DEFAULT_MIN_ROPE_LENGTH: f32 = 8.0;
    /// Swing acceleration along the circular tangent in virtual px/s^2.
    pub const DEFAULT_SWING_ACCEL: f32 = 650.0;
    /// Maximum swing velocity in virtual px/s.
    pub const DEFAULT_MAX_SWING_SPEED: f32 = 350.0;
    /// Drag factor applied while swinging.
    pub const DEFAULT_SWING_DAMPING: f32 = 0.25;

    pub fn new() -> Self {
        Self {
            state: HookState::Inactive,
            max_length: Self::DEFAULT_MAX_LENGTH,
            pull_speed: Self::DEFAULT_PULL_SPEED,
            min_rope_length: Self::DEFAULT_MIN_ROPE_LENGTH,
            swing_accel: Self::DEFAULT_SWING_ACCEL,
            max_swing_speed: Self::DEFAULT_MAX_SWING_SPEED,
            swing_damping: Self::DEFAULT_SWING_DAMPING,
        }
    }

    pub fn with_max_length(mut self, len: f32) -> Self {
        self.max_length = len.max(1.0);
        self
    }

    pub fn with_pull_speed(mut self, speed: f32) -> Self {
        self.pull_speed = speed.max(0.0);
        self
    }

    pub fn with_swing_accel(mut self, accel: f32) -> Self {
        self.swing_accel = accel.max(0.0);
        self
    }

    pub fn is_attached(&self) -> bool {
        matches!(self.state, HookState::Attached { .. })
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, HookState::Inactive)
    }

    pub fn anchor(&self) -> Option<Vector2> {
        match self.state {
            HookState::Attached { anchor, .. } => Some(anchor),
            _ => None,
        }
    }

    pub fn rope_length(&self) -> Option<f32> {
        match self.state {
            HookState::Attached { rope_length, .. } => Some(rope_length),
            _ => None,
        }
    }

    pub fn set_rope_length(&mut self, len: f32) {
        if let HookState::Attached {
            ref mut rope_length,
            ..
        } = self.state
        {
            *rope_length = len.max(self.min_rope_length);
        }
    }

    /// Immediately dissolves the rope and detaches the hook.
    pub fn dissolve(&mut self) {
        self.state = HookState::Inactive;
    }

    /// Throws the grappling hook from `player_pos` towards `target_world`.
    ///
    /// Raycasts up to `self.max_length` (default 30.0 px). If it hits a solid wall or platform,
    /// it sticks to it, establishing an attached rope. Returns true if attached, false if missed.
    pub fn throw_hook(
        &mut self,
        player_pos: Vector2,
        target_world: Vector2,
        world: &World,
        player_id: usize,
    ) -> bool {
        let diff = Vector2::new(target_world.x - player_pos.x, target_world.y - player_pos.y);
        let dist = (diff.x * diff.x + diff.y * diff.y).sqrt();
        let dir = if dist < 1e-4 {
            Vector2::new(0.0, -1.0)
        } else {
            Vector2::new(diff.x / dist, diff.y / dist)
        };

        let start = player_pos;
        let end = Vector2::new(
            start.x + dir.x * self.max_length,
            start.y + dir.y * self.max_length,
        );

        if let Some((hit_point, hit_dist)) = find_closest_obstacle_hit(start, end, world, player_id) {
            self.state = HookState::Attached {
                anchor: hit_point,
                rope_length: hit_dist.max(self.min_rope_length),
            };
            true
        } else {
            self.state = HookState::Missed {
                tip: end,
                timer: 0.15,
            };
            false
        }
    }

    /// Updates the miss visual effect timer when not attached.
    pub fn update_miss(&mut self, dt: f32) {
        if let HookState::Missed { ref mut timer, .. } = self.state {
            *timer -= dt;
            if *timer <= 0.0 {
                self.state = HookState::Inactive;
            }
        }
    }

    /// Updates the rope physics, swinging momentum, pulling up, and jump/dissolve.
    pub fn update(
        &mut self,
        world: &mut World,
        player_id: usize,
        pull_up: bool,
        swing_left: bool,
        swing_right: bool,
        jump: bool,
        jump_impulse: f32,
        dt: f32,
    ) -> HookUpdateResult {
        if dt <= 0.0 {
            return HookUpdateResult::default();
        }

        if let HookState::Missed { ref mut timer, .. } = self.state {
            *timer -= dt;
            if *timer <= 0.0 {
                self.state = HookState::Inactive;
            }
            return HookUpdateResult::default();
        }

        let (anchor, mut rope_len) = match self.state {
            HookState::Attached { anchor, rope_length } => (anchor, rope_length),
            _ => return HookUpdateResult::default(),
        };

        // When pressing Space, dissolve the rope and jump at the same time
        if jump {
            self.dissolve();
            let current_vy = world.velocities[player_id].y;
            world.velocities[player_id].y = (current_vy - jump_impulse).min(-jump_impulse * 0.75);
            return HookUpdateResult {
                dissolved_and_jumped: true,
                was_attached: true,
            };
        }

        let mut pos = world.positions[player_id];
        let mut vel = world.velocities[player_id];

        // 1. Pulling up on the rope with Arrow Up
        if pull_up {
            rope_len = (rope_len - self.pull_speed * dt).max(self.min_rope_length);
        }

        // 2. Swinging with Arrow Left and Arrow Right
        let rx = pos.x - anchor.x;
        let ry = pos.y - anchor.y;
        let dist = (rx * rx + ry * ry).sqrt();

        let (ur_x, ur_y) = if dist < 1e-4 {
            (0.0, 1.0)
        } else {
            (rx / dist, ry / dist)
        };

        // Tangent vector perpendicular to radial vector (u_theta)
        let ut_x = ur_y;
        let ut_y = -ur_x;

        // Ensure tangent_right points towards positive X
        let (tr_x, tr_y) = if ut_x >= 0.0 {
            (ut_x, ut_y)
        } else {
            (-ut_x, -ut_y)
        };
        let (tl_x, tl_y) = (-tr_x, -tr_y);

        if swing_right && !swing_left {
            vel.x += tr_x * self.swing_accel * dt;
            vel.y += tr_y * self.swing_accel * dt;
        } else if swing_left && !swing_right {
            vel.x += tl_x * self.swing_accel * dt;
            vel.y += tl_y * self.swing_accel * dt;
        }

        // 3. Gravity
        let grav_scale = world.physics_data[player_id]
            .as_ref()
            .map_or(1.0, |p| p.gravity_scale);
        vel.y += World::DRAG * grav_scale * dt;

        // 4. Integrate position
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;

        // 5. Enforce rope constraint: player cannot be further than rope_len from anchor
        let new_rx = pos.x - anchor.x;
        let new_ry = pos.y - anchor.y;
        let new_dist = (new_rx * new_rx + new_ry * new_ry).sqrt();

        if new_dist > rope_len && new_dist > 1e-4 {
            let nur_x = new_rx / new_dist;
            let nur_y = new_ry / new_dist;

            // Project position onto the circle of radius rope_len
            pos.x = anchor.x + nur_x * rope_len;
            pos.y = anchor.y + nur_y * rope_len;

            // Remove outward radial velocity
            let v_radial = vel.x * nur_x + vel.y * nur_y;
            if v_radial > 0.0 {
                vel.x -= nur_x * v_radial;
                vel.y -= nur_y * v_radial;
            }

            // Apply light swing air drag
            let drag_factor = (-self.swing_damping * dt).exp();
            vel.x *= drag_factor;
            vel.y *= drag_factor;
        }

        // 6. Cap maximum swing speed to avoid instability
        let speed = (vel.x * vel.x + vel.y * vel.y).sqrt();
        if speed > self.max_swing_speed {
            let inv_speed = self.max_swing_speed / speed;
            vel.x *= inv_speed;
            vel.y *= inv_speed;
        }

        // Write back updated position and velocity
        world.positions[player_id] = pos;
        world.velocities[player_id] = vel;
        self.state = HookState::Attached {
            anchor,
            rope_length: rope_len,
        };

        HookUpdateResult {
            dissolved_and_jumped: false,
            was_attached: true,
        }
    }

    /// Post-collision adjustment: adapts rope length if collision pushed the player closer to anchor.
    pub fn post_collision_adjust(&mut self, player_pos: Vector2) {
        if let HookState::Attached {
            anchor,
            ref mut rope_length,
        } = self.state
        {
            let current_dist =
                ((player_pos.x - anchor.x).powi(2) + (player_pos.y - anchor.y).powi(2)).sqrt();
            if current_dist < *rope_length {
                *rope_length = current_dist.max(self.min_rope_length);
            }
        }
    }

    /// Renders the grappling hook rope, anchor point on wall, and miss effect to canvas.
    pub fn draw(
        &self,
        canvas: &mut RaylibTextureMode<RaylibHandle>,
        camera: &GameCamera,
        player_pos: Vector2,
    ) {
        match self.state {
            HookState::Attached { anchor, .. } => {
                let p_canvas = Vector2::new(
                    (player_pos.x - camera.x()) * VIRTUAL_SCALE,
                    (player_pos.y - camera.y()) * VIRTUAL_SCALE,
                );
                let a_canvas = Vector2::new(
                    (anchor.x - camera.x()) * VIRTUAL_SCALE,
                    (anchor.y - camera.y()) * VIRTUAL_SCALE,
                );

                // Draw outer rope line
                canvas.draw_line_ex(p_canvas, a_canvas, 2.5 * VIRTUAL_SCALE, Color::BROWN);
                // Draw inner highlighted rope strand
                canvas.draw_line_ex(p_canvas, a_canvas, 1.2 * VIRTUAL_SCALE, Color::GOLD);

                // Draw hook anchor point on wall
                let hook_size = 4.0 * VIRTUAL_SCALE;
                let hook_rect = Rectangle::new(
                    a_canvas.x - hook_size * 0.5,
                    a_canvas.y - hook_size * 0.5,
                    hook_size,
                    hook_size,
                );
                canvas.draw_rectangle_rec(hook_rect, Color::LIGHTGRAY);
                // Hook metallic center
                let core_size = 2.0 * VIRTUAL_SCALE;
                let core_rect = Rectangle::new(
                    a_canvas.x - core_size * 0.5,
                    a_canvas.y - core_size * 0.5,
                    core_size,
                    core_size,
                );
                canvas.draw_rectangle_rec(core_rect, Color::DARKGRAY);
            }
            HookState::Missed { tip, timer } => {
                let alpha = (timer / 0.15).clamp(0.0, 1.0);
                let color = Color::new(220, 220, 220, (alpha * 200.0) as u8);
                let p_canvas = Vector2::new(
                    (player_pos.x - camera.x()) * VIRTUAL_SCALE,
                    (player_pos.y - camera.y()) * VIRTUAL_SCALE,
                );
                let t_canvas = Vector2::new(
                    (tip.x - camera.x()) * VIRTUAL_SCALE,
                    (tip.y - camera.y()) * VIRTUAL_SCALE,
                );
                canvas.draw_line_ex(p_canvas, t_canvas, 1.5 * VIRTUAL_SCALE, color);
            }
            HookState::Inactive => {}
        }
    }
}

impl Default for GrapplingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Collider, PhysicsData};
    use crate::types::{EntityType, SpriteID};

    #[test]
    fn test_ray_segment_intersect_aabb_vertical_hit() {
        let start = Vector2::new(100.0, 100.0);
        let end = Vector2::new(100.0, 60.0); // Shoots up 40 px
        let rect = Rectangle::new(80.0, 50.0, 40.0, 20.0); // Platform from y=50 to y=70

        let hit = ray_segment_intersect_aabb(start, end, rect);
        assert!(hit.is_some());
        let (hit_pos, dist) = hit.unwrap();
        assert_eq!(hit_pos.x, 100.0);
        assert_eq!(hit_pos.y, 70.0); // Hits bottom edge of platform
        assert!((dist - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_segment_intersect_aabb_misses_too_short() {
        let start = Vector2::new(100.0, 100.0);
        let end = Vector2::new(100.0, 75.0); // Shoots up 25 px
        let rect = Rectangle::new(80.0, 50.0, 40.0, 20.0); // Platform bottom at y=70 (distance 30)

        let hit = ray_segment_intersect_aabb(start, end, rect);
        assert!(hit.is_none(), "Should miss because platform is 30 px away and ray length is 25 px");
    }

    #[test]
    fn test_throw_hook_attaches_to_wall() {
        let mut world = World::new();

        // Solid platform at y = 70 (bottom at 75)
        let _platform = world
            .spawn(
                EntityType::Ground,
                Vector2::new(100.0, 60.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(60.0, 20.0)) // y=50..70
            .with_physics(PhysicsData::static_body())
            .build();

        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 95.0), // 25 px below platform bottom
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .build();

        let mut hook = GrapplingHook::new();
        assert!(!hook.is_attached());

        // Throw hook upwards toward (100.0, 0.0)
        let attached = hook.throw_hook(
            world.positions[player.0],
            Vector2::new(100.0, 0.0),
            &world,
            player.0,
        );

        assert!(attached);
        assert!(hook.is_attached());
        let anchor = hook.anchor().unwrap();
        assert_eq!(anchor.x, 100.0);
        assert_eq!(anchor.y, 70.0); // Hit platform bottom
        assert!((hook.rope_length().unwrap() - 25.0).abs() < 1e-4);
    }

    #[test]
    fn test_throw_hook_misses_when_out_of_reach() {
        let mut world = World::new();

        // Platform far away at y = 40 (bottom at 50, distance 50 px)
        let _platform = world
            .spawn(
                EntityType::Ground,
                Vector2::new(100.0, 40.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(60.0, 20.0))
            .build();

        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .build();

        let mut hook = GrapplingHook::new(); // default max_length 30.0
        let attached = hook.throw_hook(
            world.positions[player.0],
            Vector2::new(100.0, 0.0),
            &world,
            player.0,
        );

        assert!(!attached);
        assert!(!hook.is_attached());
        assert!(matches!(hook.state, HookState::Missed { .. }));
    }

    #[test]
    fn test_arrow_up_pulls_player_up_on_rope() {
        let mut world = World::new();
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 95.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .build();

        let mut hook = GrapplingHook::new();
        hook.state = HookState::Attached {
            anchor: Vector2::new(100.0, 70.0),
            rope_length: 25.0,
        };

        // Pull up with Arrow Up for 0.1s
        let dt = 0.1;
        hook.update(
            &mut world,
            player.0,
            true,  // pull_up
            false, // left
            false, // right
            false, // jump
            300.0,
            dt,
        );

        let new_len = hook.rope_length().unwrap();
        assert!(new_len < 25.0, "Rope length should shorten when pulling up: new_len = {}", new_len);
        let player_y = world.positions[player.0].y;
        assert!(player_y < 95.0, "Player should be pulled upward closer to anchor: player_y = {}", player_y);
    }

    #[test]
    fn test_arrow_left_right_swings_and_builds_momentum() {
        let mut world = World::new();
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(100.0, 95.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .build();

        let mut hook = GrapplingHook::new();
        hook.state = HookState::Attached {
            anchor: Vector2::new(100.0, 70.0),
            rope_length: 25.0,
        };

        // Swing right
        for _ in 0..10 {
            hook.update(
                &mut world,
                player.0,
                false, // pull_up
                false, // left
                true,  // right
                false, // jump
                300.0,
                0.016,
            );
        }

        let vel = world.velocities[player.0];
        assert!(vel.x > 0.0, "Player should build up rightward velocity: vx = {}", vel.x);
        let pos = world.positions[player.0];
        assert!(pos.x > 100.0, "Player should swing to the right: px = {}", pos.x);

        // Distance to anchor should stay constrained by rope length
        let dist = ((pos.x - 100.0).powi(2) + (pos.y - 70.0).powi(2)).sqrt();
        assert!((dist - 25.0).abs() < 1e-2, "Player should stay on circular arc: dist = {}", dist);
    }

    #[test]
    fn test_space_dissolves_rope_and_jumps() {
        let mut world = World::new();
        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(115.0, 90.0),
                Vector2::new(120.0, -50.0), // Swinging up and right
                0.0,
                true,
                SpriteID::Player,
            )
            .build();

        let mut hook = GrapplingHook::new();
        hook.state = HookState::Attached {
            anchor: Vector2::new(100.0, 70.0),
            rope_length: 25.0,
        };

        let res = hook.update(
            &mut world,
            player.0,
            false,
            false,
            false,
            true, // jump!
            300.0,
            0.016,
        );

        assert!(res.dissolved_and_jumped);
        assert!(!hook.is_attached());
        assert_eq!(hook.state, HookState::Inactive);

        // Jump impulse added: upward velocity is boosted, horizontal momentum preserved
        assert_eq!(world.velocities[player.0].x, 120.0);
        assert!(world.velocities[player.0].y <= -300.0);
    }
}
