use crate::prelude::*;
use crate::scenes::{Scene, SceneAction, SceneTag};
use crate::systems::collision;
use sola_raylib::prelude::*;

/// Player input state for Act Two platformer mechanics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActTwoInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub jump_held: bool,
}

impl ActTwoInput {
    pub fn new(left: bool, right: bool, jump: bool) -> Self {
        Self {
            left,
            right,
            jump,
            jump_held: jump,
        }
    }

    pub fn with_jump_held(mut self, held: bool) -> Self {
        self.jump_held = held;
        self
    }

    pub fn from_raylib(rl: &RaylibHandle) -> Self {
        Self {
            left: rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A),
            right: rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D),
            jump: rl.is_key_pressed(KeyboardKey::KEY_SPACE),
            jump_held: rl.is_key_down(KeyboardKey::KEY_SPACE),
        }
    }
}

/// Act Two scene: testing ground for gameplay mechanics.
///
/// Features a solid green collision rectangle spanning the entire floor of the camera viewport
/// and a purple player rectangle in the center of the viewport that naturally falls under gravity,
/// moves horizontally with left/right arrow keys, jumps on pressing Space, and swings slowly
/// upwards in a smooth curve powered by thrusters when Space remains pressed.
pub struct ActTwoScene {
    world: World,
    player_id: Option<EntityID>,
    floor_id: Option<EntityID>,
    pub camera: GameCamera,
    max_thrust_duration: f32,
    thrust_fuel: f32,
    is_thrusting: bool,
    thrust_activation_delay: f32,
    thrust_delay_timer: f32,
    jump_held: bool,
}

impl ActTwoScene {
    /// Viewport floor collision rectangle dimensions and center position.
    /// Spans the entire virtual viewport width (320 px) along the bottom (y = 160.0..180.0).
    pub const FLOOR_WIDTH: f32 = VIRTUAL_WIDTH as f32; // 320.0
    pub const FLOOR_HEIGHT: f32 = 20.0;
    pub const FLOOR_X: f32 = VIRTUAL_WIDTH as f32 * 0.5; // 160.0
    pub const FLOOR_Y: f32 = VIRTUAL_HEIGHT as f32 - Self::FLOOR_HEIGHT * 0.5; // 170.0

    /// Player rectangle dimensions and initial center position in the middle of the camera viewport.
    pub const PLAYER_WIDTH: f32 = 20.0;
    pub const PLAYER_HEIGHT: f32 = 20.0;
    pub const PLAYER_SPAWN_X: f32 = VIRTUAL_WIDTH as f32 * 0.5; // 160.0
    pub const PLAYER_SPAWN_Y: f32 = VIRTUAL_HEIGHT as f32 * 0.5; // 90.0

    /// Expected resting Y coordinate for the player center once landed on top of the floor collider:
    /// floor top edge (160.0) - player half height (10.0) = 150.0.
    pub const PLAYER_RESTING_Y: f32 = (Self::FLOOR_Y - Self::FLOOR_HEIGHT * 0.5) - Self::PLAYER_HEIGHT * 0.5;

    /// Horizontal walking speed in virtual px/s.
    pub const PLAYER_MOVE_SPEED: f32 = 80.0;

    /// Upward vertical impulse applied when jumping.
    pub const PLAYER_JUMP_IMPULSE: f32 = 260.0;

    /// Default maximum thruster duration in seconds (upgradeable).
    pub const DEFAULT_MAX_THRUST_DURATION: f32 = 1.5;

    /// Upward thruster acceleration in virtual px/s^2.
    /// Overcomes downward gravity (240.0 px/s^2) to smoothly curve out of falls and climb upward.
    pub const THRUST_ACCELERATION: f32 = 560.0;

    /// Maximum upward speed (virtual px/s) while climbing under thruster power.
    /// Keeps the upward climb gentle, controlled, and visibly curving.
    pub const MAX_THRUST_SPEED: f32 = 65.0;

    /// Default delay (in seconds) before thruster activation when Space is pressed continuously from a ground jump.
    /// Prevents standard jumps from consuming thruster fuel during the initial upward trajectory.
    pub const DEFAULT_THRUST_ACTIVATION_DELAY: f32 = 0.25;

    pub fn new() -> Self {
        let mut world = World::new();

        // 1. Solid floor collision rectangle spanning the whole camera viewport from left to right
        let floor_id = world
            .spawn(
                EntityType::Ground,
                Vector2::new(Self::FLOOR_X, Self::FLOOR_Y),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(Self::FLOOR_WIDTH, Self::FLOOR_HEIGHT))
            .with_physics(PhysicsData::static_body())
            .build();

        // 2. Player entity positioned in the middle of the view
        let player_id = world
            .spawn(
                EntityType::Player,
                Vector2::new(Self::PLAYER_SPAWN_X, Self::PLAYER_SPAWN_Y),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_physics_preset(PhysicsPreset::Normal)
            .with_collider(Collider::new(Self::PLAYER_WIDTH, Self::PLAYER_HEIGHT))
            .build();

        Self {
            world,
            player_id: Some(player_id),
            floor_id: Some(floor_id),
            camera: GameCamera::new(),
            max_thrust_duration: Self::DEFAULT_MAX_THRUST_DURATION,
            thrust_fuel: Self::DEFAULT_MAX_THRUST_DURATION,
            is_thrusting: false,
            thrust_activation_delay: Self::DEFAULT_THRUST_ACTIVATION_DELAY,
            thrust_delay_timer: 0.0,
            jump_held: false,
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn camera(&self) -> &GameCamera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut GameCamera {
        &mut self.camera
    }

    pub fn player_id(&self) -> Option<EntityID> {
        self.player_id
    }

    pub fn player_position(&self) -> Option<Vector2> {
        self.player_id.map(|id| self.world.positions[id.0])
    }

    pub fn floor_id(&self) -> Option<EntityID> {
        self.floor_id
    }

    pub fn floor_position(&self) -> Option<Vector2> {
        self.floor_id.map(|id| self.world.positions[id.0])
    }

    pub fn ground_id(&self) -> Option<EntityID> {
        self.floor_id
    }

    pub fn ground_position(&self) -> Option<Vector2> {
        self.floor_position()
    }

    /// Maximum thruster capacity in seconds (upgradeable).
    pub fn max_thrust_duration(&self) -> f32 {
        self.max_thrust_duration
    }

    /// Sets the maximum thruster capacity in seconds (upgrade hook).
    pub fn set_max_thrust_duration(&mut self, duration: f32) {
        self.max_thrust_duration = duration.max(0.0);
        self.thrust_fuel = self.thrust_fuel.min(self.max_thrust_duration);
    }

    /// Current configured delay (in seconds) before thrusters ignite when Space is held continuously from a jump.
    pub fn thrust_activation_delay(&self) -> f32 {
        self.thrust_activation_delay
    }

    /// Sets the delay (in seconds) before thrusters ignite when Space is held continuously from a jump.
    pub fn set_thrust_activation_delay(&mut self, delay: f32) {
        self.thrust_activation_delay = delay.max(0.0);
    }

    /// Current remaining delay timer before thruster activation.
    pub fn thrust_delay_timer(&self) -> f32 {
        self.thrust_delay_timer
    }

    /// Current remaining thruster fuel in seconds.
    pub fn thrust_fuel(&self) -> f32 {
        self.thrust_fuel
    }

    /// Directly sets current thruster fuel in seconds.
    pub fn set_thrust_fuel(&mut self, fuel: f32) {
        self.thrust_fuel = fuel.clamp(0.0, self.max_thrust_duration);
    }

    /// Returns true if the thrusters are actively firing this frame.
    pub fn is_thrusting(&self) -> bool {
        self.is_thrusting
    }

    /// Returns true if the thrusters are actively firing this frame.
    pub fn is_player_thrusting(&self) -> bool {
        self.is_thrusting
    }

    /// Returns true if the player is in contact with solid ground directly underneath.
    pub fn is_player_grounded(&self) -> bool {
        if let Some(player_id) = self.player_id {
            if player_id.0 < self.world.next_entity_id && self.world.alive[player_id.0] {
                return collision::is_blocked_down(&self.world, player_id.0);
            }
        }
        false
    }

    /// Handles platformer inputs for the player:
    /// - Arrow-Left / A: move left
    /// - Arrow-Right / D: move right
    /// - Space pressed on ground: jump (initiates delayed thruster window)
    /// - Space held continuously: engages thrusters once the delay window elapses
    /// - Space pressed while airborne (after release): engages thrusters immediately
    pub fn handle_input(&mut self, input: &ActTwoInput) {
        let Some(player_id) = self.player_id else { return };
        if player_id.0 >= self.world.next_entity_id || !self.world.alive[player_id.0] {
            return;
        }

        self.jump_held = input.jump_held;

        // Horizontal movement: left and right arrow keys
        if input.left && !input.right {
            self.world.velocities[player_id.0].x = -Self::PLAYER_MOVE_SPEED;
        } else if input.right && !input.left {
            self.world.velocities[player_id.0].x = Self::PLAYER_MOVE_SPEED;
        } else {
            self.world.velocities[player_id.0].x = 0.0;
        }

        let grounded = self.is_player_grounded();

        // Jump when pressing Space and resting on the ground
        if input.jump && grounded {
            self.world.velocities[player_id.0].y = -Self::PLAYER_JUMP_IMPULSE;
            // Start the activation delay so the initial upward trajectory of the jump does not consume thrust
            self.thrust_delay_timer = self.thrust_activation_delay;
            self.is_thrusting = false;
        }

        // Thruster activation:
        // - If Space is released, reset delay timer and deactivate thrusters
        // - If Space is held while airborne and fuel remains:
        //   - Ignites if delay timer has elapsed (or if Space was pressed in mid-air with timer at 0)
        //   - Otherwise remains off during the initial jump delay
        if !input.jump_held {
            self.thrust_delay_timer = 0.0;
            self.is_thrusting = false;
        } else if !grounded && self.thrust_fuel > 0.0 {
            if self.thrust_delay_timer <= 0.0 {
                self.is_thrusting = true;
            } else {
                self.is_thrusting = false;
            }
        } else {
            self.is_thrusting = false;
        }
    }

    /// Resets the player entity back to the center of the viewport with zero velocity and full fuel.
    pub fn reset_player(&mut self) {
        self.is_thrusting = false;
        self.thrust_delay_timer = 0.0;
        self.jump_held = false;
        self.thrust_fuel = self.max_thrust_duration;
        if let Some(player_id) = self.player_id {
            if player_id.0 < self.world.next_entity_id {
                self.world.positions[player_id.0] = Vector2::new(Self::PLAYER_SPAWN_X, Self::PLAYER_SPAWN_Y);
                self.world.velocities[player_id.0] = Vector2::new(0.0, 0.0);
                self.world.rotations[player_id.0] = 0.0;
            }
        }
    }

    /// Updates physics integration, thruster forces, fuel consumption, and boundary clamping.
    pub fn update_physics(&mut self, dt: f32) {
        let Some(player_id) = self.player_id else {
            physics::update(&mut self.world, dt);
            collision::update(&mut self.world, dt);
            return;
        };

        // Count down thruster delay if Space is held continuously from a ground jump
        if self.thrust_delay_timer > 0.0 && dt > 0.0 {
            self.thrust_delay_timer = (self.thrust_delay_timer - dt).max(0.0);
            if self.thrust_delay_timer == 0.0 && self.jump_held && !self.is_player_grounded() && self.thrust_fuel > 0.0 {
                self.is_thrusting = true;
            }
        }

        if self.is_thrusting && self.thrust_fuel > 0.0 && dt > 0.0 {
            // Deplete thruster fuel
            self.thrust_fuel = (self.thrust_fuel - dt).max(0.0);
            if self.thrust_fuel == 0.0 {
                self.is_thrusting = false;
            }

            // Apply upward thrust acceleration to smoothly curve vertical trajectory
            if self.world.velocities[player_id.0].y > -Self::MAX_THRUST_SPEED {
                self.world.velocities[player_id.0].y -= Self::THRUST_ACCELERATION * dt;
                if self.world.velocities[player_id.0].y < -Self::MAX_THRUST_SPEED {
                    self.world.velocities[player_id.0].y = -Self::MAX_THRUST_SPEED;
                }
            }
        }

        // Standard physics integration and collision resolution
        physics::update(&mut self.world, dt);
        collision::update(&mut self.world, dt);

        if player_id.0 < self.world.next_entity_id && self.world.alive[player_id.0] {
            // Recharge thruster fuel and reset delay when resting on the solid floor
            if collision::is_blocked_down(&self.world, player_id.0) {
                self.thrust_fuel = self.max_thrust_duration;
                self.is_thrusting = false;
                self.thrust_delay_timer = 0.0;
            }

            // Clamp player within the camera viewport boundaries
            let half_w = Self::PLAYER_WIDTH * 0.5;
            let half_h = Self::PLAYER_HEIGHT * 0.5;
            let min_x = self.camera.x() + half_w;
            let max_x = self.camera.x() + self.camera.viewport_width - half_w;
            let min_y = self.camera.y() + half_h;

            let pos = &mut self.world.positions[player_id.0];
            if pos.x < min_x {
                pos.x = min_x;
                if self.world.velocities[player_id.0].x < 0.0 {
                    self.world.velocities[player_id.0].x = 0.0;
                }
            } else if pos.x > max_x {
                pos.x = max_x;
                if self.world.velocities[player_id.0].x > 0.0 {
                    self.world.velocities[player_id.0].x = 0.0;
                }
            }

            // Ceiling collision clamp
            if pos.y < min_y {
                pos.y = min_y;
                if self.world.velocities[player_id.0].y < 0.0 {
                    self.world.velocities[player_id.0].y = 0.0;
                }
            }
        }
    }
}

impl Default for ActTwoScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for ActTwoScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _resources: &mut ResourceManager) {}

    fn on_exit(&mut self) {}

    fn update(&mut self, ctx: &mut Context) -> Option<SceneAction> {
        // Press R to reset the player to center for easy repeatable fall testing
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_R) {
            self.reset_player();
        }

        let input = ActTwoInput::from_raylib(ctx.rl);
        self.handle_input(&input);

        self.update_physics(ctx.dt);
        None
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager) {
        canvas.clear_background(Color::BLACK);

        for i in 0..self.world.next_entity_id {
            if !self.world.alive[i] {
                continue;
            }
            let col = match self.world.colliders[i] {
                Some(c) => c,
                None => continue,
            };

            let color = if Some(EntityID(i)) == self.floor_id || self.world.tags[i].is_solid_obstacle() {
                Color::GREEN
            } else if Some(EntityID(i)) == self.player_id || self.world.tags[i] == EntityType::Player {
                Color::PURPLE
            } else {
                Color::GRAY
            };

            let pos = self.world.positions[i];
            let rotation = self.world.rotations[i];
            let screen_w = col.width * VIRTUAL_SCALE;
            let screen_h = col.height * VIRTUAL_SCALE;

            let dest = Rectangle::new(
                (pos.x + col.offset.x - self.camera.x()) * VIRTUAL_SCALE,
                (pos.y + col.offset.y - self.camera.y()) * VIRTUAL_SCALE,
                screen_w,
                screen_h,
            );
            let origin = Vector2::new(dest.width * 0.5, dest.height * 0.5);
            canvas.draw_rectangle_pro(dest, origin, rotation, color);
        }

        // Draw thruster flame beneath player when firing
        if self.is_thrusting {
            if let Some(player_id) = self.player_id {
                if player_id.0 < self.world.next_entity_id && self.world.alive[player_id.0] {
                    let pos = self.world.positions[player_id.0];
                    let flame_w = 8.0 * VIRTUAL_SCALE;
                    let flame_h = 6.0 * VIRTUAL_SCALE;
                    let flame_dest = Rectangle::new(
                        (pos.x - self.camera.x()) * VIRTUAL_SCALE,
                        (pos.y + Self::PLAYER_HEIGHT * 0.5 - self.camera.y()) * VIRTUAL_SCALE + flame_h * 0.5,
                        flame_w,
                        flame_h,
                    );
                    let origin = Vector2::new(flame_dest.width * 0.5, flame_dest.height * 0.5);
                    canvas.draw_rectangle_pro(flame_dest, origin, 0.0, Color::ORANGE);

                    // Inner flame core
                    let core_w = 4.0 * VIRTUAL_SCALE;
                    let core_h = 3.0 * VIRTUAL_SCALE;
                    let core_dest = Rectangle::new(
                        (pos.x - self.camera.x()) * VIRTUAL_SCALE,
                        (pos.y + Self::PLAYER_HEIGHT * 0.5 - self.camera.y()) * VIRTUAL_SCALE + core_h * 0.5,
                        core_w,
                        core_h,
                    );
                    let core_origin = Vector2::new(core_dest.width * 0.5, core_dest.height * 0.5);
                    canvas.draw_rectangle_pro(core_dest, core_origin, 0.0, Color::GOLD);
                }
            }
        }

        // Draw minimal HUD thruster fuel gauge in upper-left corner
        let bar_x = 12.0 * VIRTUAL_SCALE;
        let bar_y = 10.0 * VIRTUAL_SCALE;
        let bar_total_w = 50.0 * VIRTUAL_SCALE;
        let bar_h = 4.0 * VIRTUAL_SCALE;

        // Background / frame
        canvas.draw_rectangle_pro(
            Rectangle::new(bar_x + bar_total_w * 0.5, bar_y + bar_h * 0.5, bar_total_w, bar_h),
            Vector2::new(bar_total_w * 0.5, bar_h * 0.5),
            0.0,
            Color::new(40, 40, 50, 200),
        );

        // Fuel fill
        let fuel_pct = (self.thrust_fuel / self.max_thrust_duration.max(0.001)).clamp(0.0, 1.0);
        let fill_w = bar_total_w * fuel_pct;
        if fill_w > 0.0 {
            let fuel_color = if fuel_pct > 0.3 { Color::GOLD } else { Color::ORANGE };
            canvas.draw_rectangle_pro(
                Rectangle::new(bar_x + fill_w * 0.5, bar_y + bar_h * 0.5, fill_w, bar_h),
                Vector2::new(fill_w * 0.5, bar_h * 0.5),
                0.0,
                fuel_color,
            );
        }
    }

    fn tag(&self) -> Option<SceneTag> {
        Some(SceneTag::ActTwo)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_two_scene_initialization() {
        let scene = ActTwoScene::new();
        assert_eq!(scene.tag(), Some(SceneTag::ActTwo));
        assert_eq!(scene.camera().x(), 0.0);
        assert_eq!(scene.camera().y(), 0.0);
        assert!(!scene.is_thrusting());
        assert_eq!(scene.max_thrust_duration(), ActTwoScene::DEFAULT_MAX_THRUST_DURATION);
        assert_eq!(scene.thrust_fuel(), ActTwoScene::DEFAULT_MAX_THRUST_DURATION);

        let floor_id = scene.floor_id().expect("Floor entity should exist");
        assert_eq!(scene.world().tags[floor_id.0], EntityType::Ground);
        assert_eq!(
            scene.world().positions[floor_id.0],
            Vector2::new(ActTwoScene::FLOOR_X, ActTwoScene::FLOOR_Y)
        );
        let floor_col = scene.world().colliders[floor_id.0].expect("Floor should have collider");
        assert_eq!(floor_col.width, ActTwoScene::FLOOR_WIDTH);
        assert_eq!(floor_col.height, ActTwoScene::FLOOR_HEIGHT);
        assert!(floor_col.is_solid);
        let floor_phys = scene.world().physics_data[floor_id.0].as_ref().expect("Floor should have physics");
        assert!(floor_phys.is_static);

        let player_id = scene.player_id().expect("Player entity should exist");
        assert_eq!(scene.world().tags[player_id.0], EntityType::Player);
        assert_eq!(
            scene.world().positions[player_id.0],
            Vector2::new(ActTwoScene::PLAYER_SPAWN_X, ActTwoScene::PLAYER_SPAWN_Y)
        );
        let player_col = scene.world().colliders[player_id.0].expect("Player should have collider");
        assert_eq!(player_col.width, ActTwoScene::PLAYER_WIDTH);
        assert_eq!(player_col.height, ActTwoScene::PLAYER_HEIGHT);
        assert!(player_col.is_solid);
        let player_phys = scene.world().physics_data[player_id.0].as_ref().expect("Player should have physics");
        assert!(!player_phys.is_static);
    }

    #[test]
    fn test_floor_spans_entire_camera_viewport() {
        let scene = ActTwoScene::new();
        let floor_id = scene.floor_id().unwrap();
        let floor_col = scene.world().colliders[floor_id.0].unwrap();
        let floor_pos = scene.world().positions[floor_id.0];

        let (min_x, min_y, max_x, max_y) = floor_col.aabb(floor_pos);
        // Spans from left side (0.0) to right side (320.0) of the viewport
        assert_eq!(min_x, scene.camera().x());
        assert_eq!(max_x, scene.camera().x() + scene.camera().viewport_width);
        // Floor bottom aligns with viewport bottom (180.0)
        assert_eq!(max_y, scene.camera().y() + scene.camera().viewport_height);
        assert_eq!(min_y, scene.camera().y() + scene.camera().viewport_height - ActTwoScene::FLOOR_HEIGHT);
    }

    #[test]
    fn test_player_spawn_in_middle_of_view() {
        let scene = ActTwoScene::new();
        let player_pos = scene.player_position().unwrap();
        assert_eq!(player_pos.x, scene.camera().viewport_width * 0.5);
        assert_eq!(player_pos.y, scene.camera().viewport_height * 0.5);
    }

    #[test]
    fn test_player_naturally_falls_and_lands_on_ground() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        let initial_y = scene.world().positions[player_id.0].y;
        assert_eq!(initial_y, ActTwoScene::PLAYER_SPAWN_Y);

        // Step physics forward a bit; player should be moving downwards
        scene.update_physics(0.1);
        let falling_y = scene.world().positions[player_id.0].y;
        assert!(falling_y > initial_y, "Player should fall under gravity");
        assert!(scene.world().velocities[player_id.0].y > 0.0, "Downward velocity should be positive");

        // Simulate falling until the player lands on the floor
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        let settled_y = scene.world().positions[player_id.0].y;
        assert!(
            (settled_y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3,
            "Player should settle at resting Y ({}), but was at {}",
            ActTwoScene::PLAYER_RESTING_Y,
            settled_y
        );
        assert_eq!(
            scene.world().velocities[player_id.0].y,
            0.0,
            "Velocity should be 0 once settled on ground"
        );

        // Continue running physics; player must stay resting on the ground and not fall through
        for _ in 0..100 {
            scene.update_physics(0.016);
        }

        let still_settled_y = scene.world().positions[player_id.0].y;
        assert!(
            (still_settled_y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3,
            "Player should remain at resting Y ({}), but was at {}",
            ActTwoScene::PLAYER_RESTING_Y,
            still_settled_y
        );
    }

    #[test]
    fn test_player_jumps_when_pressing_space() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Let player land on the floor first
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        assert!(scene.is_player_grounded());
        assert!((scene.world().positions[player_id.0].y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3);

        // Press Space to jump (tap, not held)
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(false));
        assert_eq!(scene.world().velocities[player_id.0].y, -ActTwoScene::PLAYER_JUMP_IMPULSE);

        // Advance physics: player should ascend into the air
        scene.update_physics(0.016);
        let jumped_y = scene.world().positions[player_id.0].y;
        assert!(
            jumped_y < ActTwoScene::PLAYER_RESTING_Y,
            "Player Y ({}) should be above resting Y ({}) after jump",
            jumped_y,
            ActTwoScene::PLAYER_RESTING_Y
        );
        assert!(!scene.is_player_grounded(), "Player should not be grounded during jump ascent");

        // Simulate flight until peak and eventual landing back on floor
        for _ in 0..300 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(false));
            scene.update_physics(0.016);
        }
        let landed_y = scene.world().positions[player_id.0].y;
        assert!(
            (landed_y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3,
            "Player should have landed back on the ground, expected {}, got {}",
            ActTwoScene::PLAYER_RESTING_Y,
            landed_y
        );
        assert!(scene.is_player_grounded());
    }

    #[test]
    fn test_thruster_starts_delayed_when_space_held_continuously_after_jump() {
        let mut scene = ActTwoScene::new();

        // Let player land on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        assert_eq!(scene.thrust_fuel(), scene.max_thrust_duration());

        // Jump and hold Space continuously
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(true));
        scene.update_physics(0.016);

        // During the initial jump delay (e.g. first 0.16s < 0.25s delay), thrusters must NOT fire and fuel must NOT burn
        for _ in 0..10 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(true));
            scene.update_physics(0.016);
        }
        assert!(
            !scene.is_thrusting(),
            "Thrusters should not be active during the initial jump upward trajectory delay"
        );
        assert_eq!(
            scene.thrust_fuel(),
            scene.max_thrust_duration(),
            "Jump upward trajectory must not consume any thruster fuel"
        );

        // Advance past the 0.25s delay threshold (another 8 steps = ~0.29s total)
        for _ in 0..8 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(true));
            scene.update_physics(0.016);
        }
        assert!(
            scene.is_thrusting(),
            "Thrusters should now be active after continuous Space hold exceeds delay"
        );

        let fuel_before = scene.thrust_fuel();
        scene.update_physics(0.1);
        assert!(scene.thrust_fuel() < fuel_before, "Fuel should deplete during thrust");
    }

    #[test]
    fn test_jump_tap_costs_zero_thrust() {
        let mut scene = ActTwoScene::new();

        // Settle on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        // Tap jump: Space pressed and released
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(false));
        for _ in 0..60 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(false));
            scene.update_physics(0.016);
            assert!(!scene.is_thrusting(), "Thrusters must never fire on a standard jump tap");
            assert_eq!(
                scene.thrust_fuel(),
                scene.max_thrust_duration(),
                "Standard jump tap must cost 0 thruster fuel"
            );
        }
    }

    #[test]
    fn test_thruster_fires_immediately_when_space_pressed_in_mid_air() {
        let mut scene = ActTwoScene::new();

        // Settle on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        // Jump with tap (Space released)
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(false));
        for _ in 0..10 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(false));
            scene.update_physics(0.016);
        }
        assert!(!scene.is_player_grounded());
        assert_eq!(scene.thrust_fuel(), scene.max_thrust_duration());

        // Press Space while airborne: thruster must engage immediately without delay
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(true));
        assert!(scene.is_thrusting(), "Mid-air thruster press must engage immediately");

        let fuel_before = scene.thrust_fuel();
        scene.update_physics(0.05);
        assert!(scene.thrust_fuel() < fuel_before, "Fuel should burn when thrusting in air");
    }

    #[test]
    fn test_thruster_curves_downward_fall_smoothly_upward() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Set player descending in mid-air at y = 100 with downward velocity 40.0
        scene.world_mut().positions[player_id.0].y = 100.0;
        scene.world_mut().velocities[player_id.0].y = 40.0;
        assert!(!scene.is_player_grounded());

        // Engage thrusters by holding Space
        scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(true));
        assert!(scene.is_thrusting());

        // Step physics forward: downward velocity should decelerate smoothly
        scene.update_physics(0.1);
        let vy_mid = scene.world().velocities[player_id.0].y;
        assert!(vy_mid < 40.0, "Downward velocity should decrease under upward thrust");

        // Continue thrusting until trajectory reverses into upward velocity (vy < 0)
        for _ in 0..30 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(true));
            scene.update_physics(0.016);
            if scene.world().velocities[player_id.0].y < 0.0 {
                break;
            }
        }

        let vy_upward = scene.world().velocities[player_id.0].y;
        assert!(vy_upward < 0.0, "Player should be swinging upward in a curve, got vy = {}", vy_upward);
        assert!(
            vy_upward >= -ActTwoScene::MAX_THRUST_SPEED,
            "Upward speed should be capped at max thrust speed (-{}), but was {}",
            ActTwoScene::MAX_THRUST_SPEED,
            vy_upward
        );
    }

    #[test]
    fn test_thruster_fuel_depletes_after_1_5_seconds_and_cuts_out() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Set player high up with 1.5s fuel
        scene.world_mut().positions[player_id.0].y = 50.0;
        scene.world_mut().velocities[player_id.0].y = 0.0;

        assert_eq!(scene.thrust_fuel(), 1.5);

        // Hold Space continuously for ~1.68 seconds (105 steps of 0.016s)
        for _ in 0..105 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(true));
            scene.update_physics(0.016);
        }

        assert_eq!(scene.thrust_fuel(), 0.0, "Fuel should be completely spent");
        assert!(!scene.is_thrusting(), "Thruster should stop when fuel is empty");

        // Advance physics: without thrusters, gravity accelerates the player downwards
        let vy_before = scene.world().velocities[player_id.0].y;
        scene.update_physics(0.1);
        assert!(scene.world().velocities[player_id.0].y > vy_before, "Player should accelerate downward under gravity");
    }

    #[test]
    fn test_thruster_recharges_when_grounded() {
        let mut scene = ActTwoScene::new();
        let _player_id = scene.player_id().unwrap();

        // Settle on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        // Jump and burn fuel past the 0.25s delay
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(true));
        for _ in 0..45 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(true));
            scene.update_physics(0.016);
        }

        let burned_fuel = scene.thrust_fuel();
        assert!(burned_fuel < 1.5, "Fuel should be partially depleted");

        // Release space and let player land on floor
        for _ in 0..300 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(false));
            scene.update_physics(0.016);
        }

        assert!(scene.is_player_grounded());
        assert_eq!(scene.thrust_fuel(), scene.max_thrust_duration(), "Fuel should fully recharge upon landing");
    }

    #[test]
    fn test_thruster_duration_is_upgradeable() {
        let mut scene = ActTwoScene::new();
        assert_eq!(scene.max_thrust_duration(), 1.5);

        // Upgrade thruster to 3.5 seconds
        scene.set_max_thrust_duration(3.5);
        assert_eq!(scene.max_thrust_duration(), 3.5);

        // Settle on ground and verify fuel recharges to the upgraded maximum
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        assert_eq!(scene.thrust_fuel(), 3.5);
    }

    #[test]
    fn test_player_cannot_jump_in_mid_air() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Player is mid-air at spawn (y = 90.0)
        assert!(!scene.is_player_grounded());
        let vy_before = scene.world().velocities[player_id.0].y;

        // Try pressing jump in mid-air
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(false));

        // Velocity should not change to jump impulse
        assert_eq!(scene.world().velocities[player_id.0].y, vy_before);
    }

    #[test]
    fn test_player_moves_left_and_right_with_arrow_keys() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Settle on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        let center_x = scene.world().positions[player_id.0].x;
        assert_eq!(center_x, ActTwoScene::PLAYER_SPAWN_X);

        // Move Left
        scene.handle_input(&ActTwoInput::new(true, false, false));
        assert_eq!(scene.world().velocities[player_id.0].x, -ActTwoScene::PLAYER_MOVE_SPEED);
        for _ in 0..30 {
            scene.handle_input(&ActTwoInput::new(true, false, false));
            scene.update_physics(0.016);
        }
        let left_x = scene.world().positions[player_id.0].x;
        assert!(left_x < center_x, "Player X ({}) should have moved left from {}", left_x, center_x);

        // Releasing left stops horizontal velocity
        scene.handle_input(&ActTwoInput::new(false, false, false));
        assert_eq!(scene.world().velocities[player_id.0].x, 0.0);

        // Move Right
        scene.handle_input(&ActTwoInput::new(false, true, false));
        assert_eq!(scene.world().velocities[player_id.0].x, ActTwoScene::PLAYER_MOVE_SPEED);
        for _ in 0..60 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }
        let right_x = scene.world().positions[player_id.0].x;
        assert!(right_x > left_x, "Player X ({}) should have moved right from {}", right_x, left_x);
    }

    #[test]
    fn test_player_jump_and_horizontal_motion_combined() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Settle on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        let start_x = scene.world().positions[player_id.0].x;

        // Jump while moving right
        scene.handle_input(&ActTwoInput::new(false, true, true));
        assert_eq!(scene.world().velocities[player_id.0].x, ActTwoScene::PLAYER_MOVE_SPEED);
        assert_eq!(scene.world().velocities[player_id.0].y, -ActTwoScene::PLAYER_JUMP_IMPULSE);

        // Simulate flight
        for _ in 0..20 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }

        let mid_pos = scene.world().positions[player_id.0];
        assert!(mid_pos.x > start_x, "Player should have traveled right during jump");
        assert!(mid_pos.y < ActTwoScene::PLAYER_RESTING_Y, "Player should be airborne");

        // Complete jump landing
        for _ in 0..300 {
            scene.handle_input(&ActTwoInput::new(false, false, false));
            scene.update_physics(0.016);
        }

        let final_pos = scene.world().positions[player_id.0];
        assert!((final_pos.y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3);
        assert!(final_pos.x > start_x);
    }

    #[test]
    fn test_player_clamped_to_viewport_boundaries() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Settle on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        let half_w = ActTwoScene::PLAYER_WIDTH * 0.5;
        let min_x = scene.camera().x() + half_w;
        let max_x = scene.camera().x() + scene.camera().viewport_width - half_w;

        // Move Left continuously past boundary
        for _ in 0..300 {
            scene.handle_input(&ActTwoInput::new(true, false, false));
            scene.update_physics(0.016);
        }
        let clamped_left = scene.world().positions[player_id.0].x;
        assert_eq!(clamped_left, min_x, "Player should be clamped at left boundary {}", min_x);

        // Move Right continuously past boundary
        for _ in 0..600 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }
        let clamped_right = scene.world().positions[player_id.0].x;
        assert_eq!(clamped_right, max_x, "Player should be clamped at right boundary {}", max_x);
    }

    #[test]
    fn test_player_falls_through_without_floor_collider() {
        let mut scene = ActTwoScene::new();
        let floor_id = scene.floor_id().unwrap();
        // Remove floor collider
        scene.world_mut().colliders[floor_id.0] = None;

        for _ in 0..600 {
            scene.update_physics(0.016);
        }

        let player_id = scene.player_id().unwrap();
        let fallen_y = scene.world().positions[player_id.0].y;
        assert!(
            fallen_y > ActTwoScene::FLOOR_Y + 20.0,
            "Player should fall through the floor when no floor collider exists, got {}",
            fallen_y
        );
    }

    #[test]
    fn test_no_objects_with_physics_and_collider_fall_through() {
        let mut scene = ActTwoScene::new();

        // Spawn multiple test objects at different horizontal locations and heights (distinct from player x=160)
        let test_positions = [
            Vector2::new(30.0, 20.0),
            Vector2::new(100.0, 50.0),
            Vector2::new(180.0, 70.0),
            Vector2::new(230.0, 30.0),
            Vector2::new(290.0, 60.0),
        ];

        let mut spawned_ids = Vec::new();
        for &pos in &test_positions {
            let id = scene
                .world_mut()
                .spawn(
                    EntityType::Obstacle,
                    pos,
                    Vector2::new(0.0, 0.0),
                    0.0,
                    true,
                    SpriteID::None,
                )
                .with_physics_preset(PhysicsPreset::Normal)
                .with_collider(Collider::new(16.0, 16.0))
                .build();
            spawned_ids.push(id);
        }

        // Run physics simulation until all objects have landed
        for _ in 0..600 {
            scene.update_physics(0.016);
        }

        // Expected resting Y for a 16x16 collider:
        // floor top (160.0) - half height (8.0) = 152.0
        let expected_resting_y = (ActTwoScene::FLOOR_Y - ActTwoScene::FLOOR_HEIGHT * 0.5) - 8.0;

        for id in spawned_ids {
            let y = scene.world().positions[id.0].y;
            assert!(
                (y - expected_resting_y).abs() < 1e-3,
                "Object {} should be resting on ground at {}, but was at {}",
                id.0,
                expected_resting_y,
                y
            );
        }
    }

    #[test]
    fn test_reset_player() {
        let mut scene = ActTwoScene::new();
        // Fall to ground
        for _ in 0..200 {
            scene.update_physics(0.016);
        }
        let player_id = scene.player_id().unwrap();
        assert!((scene.world().positions[player_id.0].y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3);

        // Reset
        scene.reset_player();
        assert_eq!(
            scene.world().positions[player_id.0],
            Vector2::new(ActTwoScene::PLAYER_SPAWN_X, ActTwoScene::PLAYER_SPAWN_Y)
        );
        assert_eq!(scene.world().velocities[player_id.0], Vector2::new(0.0, 0.0));
        assert_eq!(scene.thrust_fuel(), scene.max_thrust_duration());
        assert!(!scene.is_thrusting());
    }
}
