use crate::prelude::*;
use crate::scenes::{Scene, SceneAction, SceneTag};
use crate::systems::collision;
use sola_raylib::prelude::*;

/// Player input state for Act Two platformer mechanics.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ActTwoInput {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub jump: bool,
    pub jump_held: bool,
    pub hook_click: bool,
    pub mouse_pos: Vector2,
}

impl ActTwoInput {
    pub fn new(left: bool, right: bool, jump: bool) -> Self {
        Self {
            left,
            right,
            up: false,
            jump,
            jump_held: jump,
            hook_click: false,
            mouse_pos: Vector2::new(0.0, 0.0),
        }
    }

    pub fn with_jump_held(mut self, held: bool) -> Self {
        self.jump_held = held;
        self
    }

    pub fn with_up(mut self, up: bool) -> Self {
        self.up = up;
        self
    }

    pub fn with_hook_click(mut self, click: bool, mouse_pos: Vector2) -> Self {
        self.hook_click = click;
        self.mouse_pos = mouse_pos;
        self
    }

    pub fn from_raylib(rl: &RaylibHandle) -> Self {
        Self {
            left: rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A),
            right: rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D),
            up: rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W),
            jump: rl.is_key_pressed(KeyboardKey::KEY_SPACE),
            jump_held: rl.is_key_down(KeyboardKey::KEY_SPACE),
            hook_click: rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
            mouse_pos: rl.get_mouse_position(),
        }
    }
}

/// Act Two scene: testing ground for gameplay mechanics.
///
/// Features a solid green collision rectangle spanning the entire floor of the camera viewport,
/// a solid green mid-air platform rectangle to test jumping and landing mechanics,
/// and a purple player rectangle in the center of the viewport that naturally falls under gravity,
/// moves horizontally with left/right arrow keys, jumps on pressing Space, and swings slowly
/// upwards in a smooth curve powered by thrusters when Space remains pressed.
pub struct ActTwoScene {
    world: World,
    player_id: Option<EntityID>,
    floor_id: Option<EntityID>,
    platform_id: Option<EntityID>,
    pub camera: GameCamera,
    camera_follow_speed: f32,
    max_thrust_duration: f32,
    thrust_fuel: f32,
    thrust_recharge_rate: f32,
    is_thrusting: bool,
    thrust_activation_delay: f32,
    thrust_delay_timer: f32,
    jump_held: bool,
    spawn_pos: Vector2,
    map: Option<Map>,
    map_entities: Option<MapSpawnResult>,
    pub hook: GrapplingHook,
    last_input: ActTwoInput,
    window_width: f32,
    window_height: f32,
}

impl ActTwoScene {
    /// Default camera follow smoothing speed (responsiveness factor for rubber-band effect).
    pub const DEFAULT_CAMERA_FOLLOW_SPEED: f32 = 5.0;

    /// Viewport floor collision rectangle dimensions and center position.
    /// Spans the entire virtual viewport width (320 px) along the bottom (y = 160.0..180.0).
    pub const FLOOR_WIDTH: f32 = VIRTUAL_WIDTH as f32; // 320.0
    pub const FLOOR_HEIGHT: f32 = 20.0;
    pub const FLOOR_X: f32 = VIRTUAL_WIDTH as f32 * 0.5; // 160.0
    pub const FLOOR_Y: f32 = VIRTUAL_HEIGHT as f32 - Self::FLOOR_HEIGHT * 0.5; // 170.0

    /// Mid-air green platform collision rectangle dimensions and center position.
    pub const PLATFORM_WIDTH: f32 = 60.0;
    pub const PLATFORM_HEIGHT: f32 = 12.0;
    pub const PLATFORM_X: f32 = 90.0;
    pub const PLATFORM_Y: f32 = 120.0;

    /// Expected resting Y coordinate for the player center once landed on top of the mid-air platform:
    /// platform top edge (114.0) - player half height (10.0) = 104.0.
    pub const PLAYER_PLATFORM_RESTING_Y: f32 = (Self::PLATFORM_Y - Self::PLATFORM_HEIGHT * 0.5) - Self::PLAYER_HEIGHT * 0.5;

    /// Player rectangle dimensions and initial center position in the middle of the camera viewport.
    pub const PLAYER_WIDTH: f32 = 20.0;
    pub const PLAYER_HEIGHT: f32 = 20.0;
    pub const PLAYER_SPAWN_X: f32 = VIRTUAL_WIDTH as f32 * 0.5; // 160.0
    pub const PLAYER_SPAWN_Y: f32 = VIRTUAL_HEIGHT as f32 * 0.5; // 90.0

    /// Gobbler enemy rectangle dimensions and horizontal move speed.
    pub const GOBBLER_WIDTH: f32 = GobblerData::WIDTH;
    pub const GOBBLER_HEIGHT: f32 = GobblerData::HEIGHT;
    pub const GOBBLER_MOVE_SPEED: f32 = GobblerData::DEFAULT_SPEED;

    /// Expected resting Y coordinate for the player center once landed on top of the floor collider:
    /// floor top edge (160.0) - player half height (10.0) = 150.0.
    pub const PLAYER_RESTING_Y: f32 = (Self::FLOOR_Y - Self::FLOOR_HEIGHT * 0.5) - Self::PLAYER_HEIGHT * 0.5;

    /// Horizontal walking speed in virtual px/s.
    pub const PLAYER_MOVE_SPEED: f32 = 80.0;

    /// Upward vertical impulse applied when jumping.
    pub const PLAYER_JUMP_IMPULSE: f32 = 300.0;

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

    /// Default thruster recharge rate in fuel units (seconds) per second while grounded (e.g. 0.5 = 3.0s for full recharge from 0).
    pub const DEFAULT_THRUST_RECHARGE_RATE: f32 = 0.5;

    pub fn new() -> Self {
        Self::from_map_file("resources/maps/act_two/map1.json")
            .unwrap_or_else(|_| Self::test_sandbox())
    }

    /// Hardcoded test sandbox scene with a single floor and mid-air platform.
    pub fn test_sandbox() -> Self {
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

        // 2. Mid-air solid platform collision rectangle
        let platform_id = world
            .spawn(
                EntityType::Ground,
                Vector2::new(Self::PLATFORM_X, Self::PLATFORM_Y),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(Self::PLATFORM_WIDTH, Self::PLATFORM_HEIGHT))
            .with_physics(PhysicsData::static_body())
            .build();

        // 3. Player entity positioned in the middle of the view
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
            platform_id: Some(platform_id),
            camera: GameCamera::new(),
            camera_follow_speed: Self::DEFAULT_CAMERA_FOLLOW_SPEED,
            max_thrust_duration: Self::DEFAULT_MAX_THRUST_DURATION,
            thrust_fuel: Self::DEFAULT_MAX_THRUST_DURATION,
            thrust_recharge_rate: Self::DEFAULT_THRUST_RECHARGE_RATE,
            is_thrusting: false,
            thrust_activation_delay: Self::DEFAULT_THRUST_ACTIVATION_DELAY,
            thrust_delay_timer: 0.0,
            jump_held: false,
            spawn_pos: Vector2::new(Self::PLAYER_SPAWN_X, Self::PLAYER_SPAWN_Y),
            map: None,
            map_entities: None,
            hook: GrapplingHook::new(),
            last_input: ActTwoInput::default(),
            window_width: GAME_WIDTH as f32,
            window_height: GAME_HEIGHT as f32,
        }
    }

    /// Creates an Act Two scene loaded from a Tiled `Map`.
    ///
    /// Spawns all platform objects as solid obstacles and positions the player
    /// at the map's designated `Spawn` point (or defaults to viewport center).
    pub fn from_map(map: &Map) -> Self {
        let mut world = World::new();
        let map_entities = map.spawn_into_world(&mut world);
        let spawn_pos = map_entities
            .spawn_point
            .unwrap_or(Vector2::new(Self::PLAYER_SPAWN_X, Self::PLAYER_SPAWN_Y));

        let player_id = world
            .spawn(
                EntityType::Player,
                spawn_pos,
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_physics_preset(PhysicsPreset::Normal)
            .with_collider(Collider::new(Self::PLAYER_WIDTH, Self::PLAYER_HEIGHT))
            .build();

        let mut camera = GameCamera::new();
        let target = camera.target_for_center(spawn_pos);
        camera.set_position(Vector2::new(target.x.max(0.0), target.y));

        Self {
            world,
            player_id: Some(player_id),
            floor_id: None,
            platform_id: None,
            camera,
            camera_follow_speed: Self::DEFAULT_CAMERA_FOLLOW_SPEED,
            max_thrust_duration: Self::DEFAULT_MAX_THRUST_DURATION,
            thrust_fuel: Self::DEFAULT_MAX_THRUST_DURATION,
            thrust_recharge_rate: Self::DEFAULT_THRUST_RECHARGE_RATE,
            is_thrusting: false,
            thrust_activation_delay: Self::DEFAULT_THRUST_ACTIVATION_DELAY,
            thrust_delay_timer: 0.0,
            jump_held: false,
            spawn_pos,
            map: Some(map.clone()),
            map_entities: Some(map_entities),
            hook: GrapplingHook::new(),
            last_input: ActTwoInput::default(),
            window_width: GAME_WIDTH as f32,
            window_height: GAME_HEIGHT as f32,
        }
    }

    /// Creates an Act Two scene by reading and parsing a Tiled JSON map file from `path`.
    pub fn from_map_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, MapError> {
        let map = Map::load_from_file(path)?;
        Ok(Self::from_map(&map))
    }

    /// Returns the loaded Tiled `Map`, if constructed from a map.
    pub fn map(&self) -> Option<&Map> {
        self.map.as_ref()
    }

    /// Returns the resolved sprite sheet path for the primary map tileset, if any.
    pub fn tileset_image_path(&self) -> Option<&str> {
        self.map
            .as_ref()
            .and_then(|m| m.tilesets().first())
            .map(|ts| ts.image.as_str())
            .filter(|s| !s.is_empty())
    }

    /// Returns the entities spawned from the loaded map, if any.
    pub fn map_entities(&self) -> Option<&MapSpawnResult> {
        self.map_entities.as_ref()
    }

    /// Returns the Gobbler enemy entity IDs spawned from the loaded map, if any.
    pub fn gobbler_ids(&self) -> &[EntityID] {
        self.map_entities
            .as_ref()
            .map(|m| m.gobblers.as_slice())
            .unwrap_or(&[])
    }

    /// Spawn position for the player in world coordinates.
    pub fn spawn_pos(&self) -> Vector2 {
        self.spawn_pos
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

    pub fn platform_id(&self) -> Option<EntityID> {
        self.platform_id
    }

    pub fn platform_position(&self) -> Option<Vector2> {
        self.platform_id.map(|id| self.world.positions[id.0])
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

    /// Thruster recharge rate in fuel units (seconds) per second when resting on ground or platform.
    pub fn thrust_recharge_rate(&self) -> f32 {
        self.thrust_recharge_rate
    }

    /// Sets the thruster recharge rate in fuel units (seconds) per second.
    pub fn set_thrust_recharge_rate(&mut self, rate: f32) {
        self.thrust_recharge_rate = rate.max(0.0);
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

    /// Camera follow speed factor governing the rubber-band response rate.
    pub fn camera_follow_speed(&self) -> f32 {
        self.camera_follow_speed
    }

    /// Sets the camera follow speed factor.
    pub fn set_camera_follow_speed(&mut self, speed: f32) {
        self.camera_follow_speed = speed.max(0.0);
    }

    /// Smoothly updates the camera to follow the player on the X and Y axes with a rubber-band effect.
    ///
    /// The left side of the camera is clamped to world coordinate `x = 0.0` at all times.
    /// As the player moves right or vertically (jumping, thruster climbing, falling),
    /// the camera follows smoothly with elastic lag.
    pub fn update_camera(&mut self, dt: f32) {
        let Some(player_id) = self.player_id else { return };
        if player_id.0 >= self.world.next_entity_id || !self.world.alive[player_id.0] {
            return;
        }

        let player_pos = self.world.positions[player_id.0];
        let mut target = self.camera.target_for_center(player_pos);
        // The left side of the camera is clamped to x = 0 on world coordinates
        target.x = target.x.max(0.0);

        self.camera.follow_smooth(target, self.camera_follow_speed, dt);
        self.camera.clamp_min_x(0.0);
    }

    /// Handles platformer inputs for the player:
    /// - Mouse Click: throws a grappling hook in the clicked direction (up to 30 px)
    /// - Arrow-Up / W while attached: pulls player up on the rope
    /// - Arrow-Left / A and Arrow-Right / D: moves or swings along the rope
    /// - Space pressed on rope: dissolves rope and jumps with accumulated momentum
    /// - Space pressed on ground: jump (initiates delayed thruster window)
    /// - Space held continuously: engages thrusters once the delay window elapses
    /// - Space pressed while airborne (after release): engages thrusters immediately
    pub fn handle_input(&mut self, input: &ActTwoInput) {
        let Some(player_id) = self.player_id else { return };
        if player_id.0 >= self.world.next_entity_id || !self.world.alive[player_id.0] {
            return;
        }

        self.last_input = *input;
        self.jump_held = input.jump_held;

        // Mouse click throws the grappling hook
        if input.hook_click {
            let target_world = self.camera.screen_to_virtual_world(
                input.mouse_pos,
                self.window_width,
                self.window_height,
            );
            let player_pos = self.world.positions[player_id.0];
            self.hook.throw_hook(player_pos, target_world, &self.world, player_id.0);
        }

        // While attached to the rope, the rope simulation governs motion and swinging!
        if self.hook.is_attached() {
            self.is_thrusting = false;
            self.thrust_delay_timer = 0.0;
            return;
        }

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

    /// Resets the player entity back to the center of the viewport with zero velocity and full fuel,
    /// and resets the camera to (0, 0).
    pub fn reset_player(&mut self) {
        self.hook.dissolve();
        self.is_thrusting = false;
        self.thrust_delay_timer = 0.0;
        self.jump_held = false;
        self.thrust_fuel = self.max_thrust_duration;
        let mut target = self.camera.target_for_center(self.spawn_pos);
        target.x = target.x.max(0.0);
        self.camera.set_position(target);
        if let Some(player_id) = self.player_id {
            if player_id.0 < self.world.next_entity_id {
                self.world.positions[player_id.0] = self.spawn_pos;
                self.world.velocities[player_id.0] = Vector2::new(0.0, 0.0);
                self.world.rotations[player_id.0] = 0.0;
            }
        }
    }

    /// Updates Gobbler enemy behaviours:
    /// - Activates once inside camera viewport.
    /// - Always moves horizontally towards player.
    pub fn update_enemies(&mut self, dt: f32) {
        behaviour::gobbler::update(&mut self.world, self.player_id, &self.camera, dt);
    }

    /// Updates physics integration, thruster forces, fuel consumption, and boundary clamping.
    pub fn update_physics(&mut self, dt: f32) {
        let Some(player_id) = self.player_id else {
            self.update_enemies(dt);
            physics::update(&mut self.world, dt);
            collision::update(&mut self.world, dt);
            return;
        };

        if self.hook.is_attached() {
            let res = self.hook.update(
                &mut self.world,
                player_id.0,
                self.last_input.up,
                self.last_input.left,
                self.last_input.right,
                self.last_input.jump,
                Self::PLAYER_JUMP_IMPULSE,
                dt,
            );
            if res.dissolved_and_jumped {
                self.thrust_delay_timer = self.thrust_activation_delay;
                self.is_thrusting = false;
            }
        } else {
            self.hook.update_miss(dt);

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

            // Standard physics integration
            physics::update(&mut self.world, dt);
        }

        // Update enemy AI behaviour before physics integration / collision
        self.update_enemies(dt);

        // Standard collision resolution
        collision::update(&mut self.world, dt);

        if self.hook.is_attached() {
            self.hook.post_collision_adjust(self.world.positions[player_id.0]);
        }

        if player_id.0 < self.world.next_entity_id && self.world.alive[player_id.0] {
            // Slowly recharge thruster fuel and reset delay when resting on solid ground or platform
            if collision::is_blocked_down(&self.world, player_id.0) {
                if self.thrust_fuel < self.max_thrust_duration && dt > 0.0 {
                    self.thrust_fuel = (self.thrust_fuel + self.thrust_recharge_rate * dt).min(self.max_thrust_duration);
                }
                if !self.hook.is_attached() {
                    self.is_thrusting = false;
                    self.thrust_delay_timer = 0.0;
                }
            }

            // Update camera to smoothly follow the player with a rubber-band effect
            self.update_camera(dt);

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

    /// Returns true if the player has entered the exit rectangle.
    pub fn is_player_at_exit(&self) -> bool {
        let Some(player_id) = self.player_id else { return false };
        if player_id.0 >= self.world.next_entity_id || !self.world.alive[player_id.0] {
            return false;
        }
        let Some(player_col) = self.world.colliders[player_id.0] else { return false };
        let player_pos = self.world.positions[player_id.0];
        let (p_min_x, p_min_y, p_max_x, p_max_y) = player_col.aabb(player_pos);

        if let Some(map_res) = &self.map_entities {
            for &exit_id in &map_res.exits {
                if exit_id.0 < self.world.next_entity_id && self.world.alive[exit_id.0] {
                    if let Some(exit_col) = self.world.colliders[exit_id.0] {
                        let exit_pos = self.world.positions[exit_id.0];
                        let (e_min_x, e_min_y, e_max_x, e_max_y) = exit_col.aabb(exit_pos);
                        let overlap_x = p_max_x.min(e_max_x) - p_min_x.max(e_min_x);
                        let overlap_y = p_max_y.min(e_max_y) - p_min_y.max(e_min_y);
                        if overlap_x > 0.0 && overlap_y > 0.0 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Returns a reference to the grappling hook.
    pub fn hook(&self) -> &GrapplingHook {
        &self.hook
    }

    /// Returns a mutable reference to the grappling hook.
    pub fn hook_mut(&mut self) -> &mut GrapplingHook {
        &mut self.hook
    }

    /// Returns true if the grappling hook is currently anchored to a wall/platform.
    pub fn is_hook_attached(&self) -> bool {
        self.hook.is_attached()
    }

    /// Checks if a transition should occur to Act Three upon reaching the exit rectangle.
    pub fn check_transition(&self) -> Option<SceneAction> {
        if self.is_player_at_exit() {
            Some(SceneAction::Switch(Box::new(ActThreeScene::new())))
        } else {
            None
        }
    }
}

impl Default for ActTwoScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for ActTwoScene {
    fn on_enter(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
        let tileset_path = self
            .tileset_image_path()
            .unwrap_or("resources/graphics/Platforms.png")
            .to_string();
        if let Err(e) = resources.load_texture(rl, thread, SpriteID::Platforms, &tileset_path) {
            eprintln!("Failed to load Act Two tileset texture '{}': {}", tileset_path, e);
        }
    }

    fn on_exit(&mut self) {}

    fn update(&mut self, ctx: &mut Context) -> Option<SceneAction> {
        // Press R to reset the player to center for easy repeatable fall testing
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_R) {
            self.reset_player();
        }

        // Press F4 to switch to minimal test sandbox
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_F4) {
            return Some(SceneAction::Switch(Box::new(Self::test_sandbox())));
        }

        self.window_width = ctx.rl.get_screen_width() as f32;
        self.window_height = ctx.rl.get_screen_height() as f32;

        let input = ActTwoInput::from_raylib(ctx.rl);
        let was_attached_before = self.hook.is_attached();
        self.handle_input(&input);

        if input.hook_click {
            if self.hook.is_attached() && !was_attached_before {
                ctx.audio.play_sfx(SoundID::Fireball_hit_Wall);
            } else if !self.hook.is_attached() {
                ctx.audio.play_sfx(SoundID::FireballShot);
            }
        }

        self.update_physics(ctx.dt);

        self.check_transition()
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
        canvas.clear_background(Color::BLACK);

        // 1. Draw static obstacle / platform collider rectangles
        for i in 0..self.world.next_entity_id {
            if !self.world.alive[i] || !self.world.tags[i].is_solid_obstacle() {
                continue;
            }
            let col = match self.world.colliders[i] {
                Some(c) => c,
                None => continue,
            };

            let color = if Some(EntityID(i)) == self.floor_id
                || Some(EntityID(i)) == self.platform_id
                || col.is_solid
            {
                Color::GREEN
            } else {
                Color::GOLD
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

        // 2. Draw Tiled tile layers from the loaded sprite sheet
        if let (Some(map), Some(tex)) = (&self.map, resources.get_texture(&SpriteID::Platforms)) {
            let tile_w = map.tile_width as f32;
            let tile_h = map.tile_height as f32;
            let cam_x = self.camera.x();
            let cam_y = self.camera.y();
            let cam_right = cam_x + self.camera.viewport_width;
            let cam_bottom = cam_y + self.camera.viewport_height;

            for layer in map.tile_layers() {
                if !layer.visible {
                    continue;
                }
                let alpha = (layer.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
                let tint = Color::new(255, 255, 255, alpha);

                for (col, row, gid) in layer.non_empty_tiles() {
                    let world_x = (col as f32) * tile_w;
                    let world_y = (row as f32) * tile_h;

                    // Cull tiles outside the active camera viewport
                    if world_x + tile_w < cam_x
                        || world_x > cam_right
                        || world_y + tile_h < cam_y
                        || world_y > cam_bottom
                    {
                        continue;
                    }

                    if let Some(source) = map.tile_source_rect(gid) {
                        let dest = Rectangle::new(
                            (world_x - cam_x) * VIRTUAL_SCALE,
                            (world_y - cam_y) * VIRTUAL_SCALE,
                            tile_w * VIRTUAL_SCALE,
                            tile_h * VIRTUAL_SCALE,
                        );
                        canvas.draw_texture_pro(
                            tex,
                            source,
                            dest,
                            Vector2::new(0.0, 0.0),
                            0.0,
                            tint,
                        );
                    }
                }
            }
        }

        // 3. Draw dynamic entities (Player, Gobbler enemies, etc.) on top of tiles
        for i in 0..self.world.next_entity_id {
            if !self.world.alive[i] || self.world.tags[i].is_solid_obstacle() {
                continue;
            }
            let col = match self.world.colliders[i] {
                Some(c) => c,
                None => continue,
            };

            let color = if Some(EntityID(i)) == self.player_id || self.world.tags[i] == EntityType::Player {
                Color::PURPLE
            } else if self.world.tags[i] == EntityType::Gobbler || self.world.tags[i].is_enemy() {
                Color::BLUE
            } else if !col.is_solid {
                Color::GOLD
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

        // Draw grappling hook rope, anchor point, and miss effect
        if let Some(player_id) = self.player_id {
            if player_id.0 < self.world.next_entity_id && self.world.alive[player_id.0] {
                let pos = self.world.positions[player_id.0];
                self.hook.draw(canvas, &self.camera, pos);
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

        // Controls hint overlay
        canvas.draw_text(
            "Controls: Mouse-Click = Throw Hook (30px) | Up = Pull Up | Left/Right = Swing | Space = Detach & Jump",
            12,
            GAME_HEIGHT - 24,
            14,
            Color::new(180, 180, 190, 180),
        );
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
        let scene = ActTwoScene::test_sandbox();
        assert_eq!(scene.tag(), Some(SceneTag::ActTwo));
        assert_eq!(scene.camera().x(), 0.0);
        assert_eq!(scene.camera().y(), 0.0);
        assert_eq!(scene.camera_follow_speed(), ActTwoScene::DEFAULT_CAMERA_FOLLOW_SPEED);
        assert!(!scene.is_thrusting());
        assert_eq!(scene.max_thrust_duration(), ActTwoScene::DEFAULT_MAX_THRUST_DURATION);
        assert_eq!(scene.thrust_fuel(), ActTwoScene::DEFAULT_MAX_THRUST_DURATION);
        assert_eq!(scene.thrust_recharge_rate(), ActTwoScene::DEFAULT_THRUST_RECHARGE_RATE);

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

        let platform_id = scene.platform_id().expect("Platform entity should exist");
        assert_eq!(scene.world().tags[platform_id.0], EntityType::Ground);
        assert_eq!(
            scene.world().positions[platform_id.0],
            Vector2::new(ActTwoScene::PLATFORM_X, ActTwoScene::PLATFORM_Y)
        );
        let platform_col = scene.world().colliders[platform_id.0].expect("Platform should have collider");
        assert_eq!(platform_col.width, ActTwoScene::PLATFORM_WIDTH);
        assert_eq!(platform_col.height, ActTwoScene::PLATFORM_HEIGHT);
        assert!(platform_col.is_solid);
        let platform_phys = scene.world().physics_data[platform_id.0].as_ref().expect("Platform should have physics");
        assert!(platform_phys.is_static);
    }

    #[test]
    fn test_floor_spans_entire_camera_viewport() {
        let scene = ActTwoScene::test_sandbox();
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
        let scene = ActTwoScene::test_sandbox();
        let player_pos = scene.player_position().unwrap();
        assert_eq!(player_pos.x, scene.camera().viewport_width * 0.5);
        assert_eq!(player_pos.y, scene.camera().viewport_height * 0.5);
    }

    #[test]
    fn test_player_naturally_falls_and_lands_on_ground() {
        let mut scene = ActTwoScene::test_sandbox();
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
        let mut scene = ActTwoScene::test_sandbox();
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
    fn test_thruster_recharges_slowly_on_ground_and_platform() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Settle player on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        assert!(scene.is_player_grounded());

        // Set fuel to a low amount (0.2s) to test gradual replenishment
        scene.set_thrust_fuel(0.2);
        assert_eq!(scene.thrust_fuel(), 0.2);

        // Step physics forward by 1 frame (0.016s)
        scene.update_physics(0.016);
        let expected_1_frame = 0.2 + ActTwoScene::DEFAULT_THRUST_RECHARGE_RATE * 0.016;
        assert!(
            (scene.thrust_fuel() - expected_1_frame).abs() < 1e-3,
            "Fuel should have increased gradually by rate * dt ({}), but was {}",
            expected_1_frame,
            scene.thrust_fuel()
        );
        // It must NOT instantly jump back to 100%
        assert!(
            scene.thrust_fuel() < scene.max_thrust_duration(),
            "Fuel must not instantly replenish to 100% upon touching ground"
        );

        // Step forward by another 60 frames (~0.96s) on the ground
        for _ in 0..60 {
            scene.update_physics(0.016);
        }
        let expected_after_60 = expected_1_frame + ActTwoScene::DEFAULT_THRUST_RECHARGE_RATE * (60.0 * 0.016);
        assert!(
            (scene.thrust_fuel() - expected_after_60).abs() < 1e-3,
            "Fuel should have increased gradually to {}, but was {}",
            expected_after_60,
            scene.thrust_fuel()
        );
        assert!(scene.thrust_fuel() < scene.max_thrust_duration());

        // Test that slow recharge also works identically when resting on the mid-air platform
        scene.world_mut().positions[player_id.0] =
            Vector2::new(ActTwoScene::PLATFORM_X, ActTwoScene::PLAYER_PLATFORM_RESTING_Y);
        scene.world_mut().velocities[player_id.0] = Vector2::new(0.0, 0.0);
        assert!(scene.is_player_grounded());

        scene.set_thrust_fuel(0.3);
        for _ in 0..20 {
            scene.update_physics(0.016);
        }
        let expected_platform_fuel = 0.3 + ActTwoScene::DEFAULT_THRUST_RECHARGE_RATE * (20.0 * 0.016);
        assert!(
            (scene.thrust_fuel() - expected_platform_fuel).abs() < 1e-3,
            "Fuel should slowly recharge on platform too, expected {}, got {}",
            expected_platform_fuel,
            scene.thrust_fuel()
        );
        assert!(scene.thrust_fuel() < scene.max_thrust_duration());

        // Wait long enough on the platform (3.0s) for it to fully recharge and cap at max
        for _ in 0..250 {
            scene.update_physics(0.016);
        }
        assert_eq!(scene.thrust_fuel(), scene.max_thrust_duration());
    }

    #[test]
    fn test_thrust_recharge_rate_tuning() {
        let mut scene = ActTwoScene::test_sandbox();
        assert_eq!(scene.thrust_recharge_rate(), ActTwoScene::DEFAULT_THRUST_RECHARGE_RATE);

        // Tune recharge rate to 1.0 fuel/s
        scene.set_thrust_recharge_rate(1.0);
        assert_eq!(scene.thrust_recharge_rate(), 1.0);

        // Settle on ground and set fuel to 0.0
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        scene.set_thrust_fuel(0.0);

        // 30 frames (0.48s) of recharge should add 0.48s of fuel at rate 1.0
        for _ in 0..30 {
            scene.update_physics(0.016);
        }
        let expected = 30.0 * 0.016 * 1.0;
        assert!((scene.thrust_fuel() - expected).abs() < 1e-3);
    }

    #[test]
    fn test_thruster_duration_is_upgradeable() {
        let mut scene = ActTwoScene::test_sandbox();
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
        assert_eq!(center_x, scene.spawn_pos().x);

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
        let mut scene = ActTwoScene::test_sandbox();
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
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Settle on ground
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        let half_w = ActTwoScene::PLAYER_WIDTH * 0.5;
        let min_x = scene.camera().x() + half_w;

        // Move Left continuously past boundary (camera left clamped to x = 0, so player clamped to min_x)
        for _ in 0..300 {
            scene.handle_input(&ActTwoInput::new(true, false, false));
            scene.update_physics(0.016);
        }
        let clamped_left = scene.world().positions[player_id.0].x;
        assert_eq!(clamped_left, min_x, "Player should be clamped at left boundary {}", min_x);
        assert_eq!(scene.camera().x(), 0.0, "Camera should stay clamped at x = 0");

        // Move Right continuously past the initial 320px viewport
        for _ in 0..600 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }
        let right_pos = scene.world().positions[player_id.0].x;
        assert!(right_pos > 320.0, "Player should wander to the right side past initial boundary");
        assert!(scene.camera().x() > 0.0, "Camera should follow the player to the right");
        // Player must always remain within the camera viewport
        let current_max_x = scene.camera().x() + scene.camera().viewport_width - half_w;
        assert!(right_pos <= current_max_x + 1e-3, "Player should remain within current camera viewport boundary");
    }

    #[test]
    fn test_player_falls_through_without_floor_collider() {
        let mut scene = ActTwoScene::test_sandbox();
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
        let mut scene = ActTwoScene::test_sandbox();

        // Spawn multiple test objects at different horizontal locations and heights (distinct from player x=160 and platform x=90)
        let test_positions = [
            Vector2::new(30.0, 20.0),
            Vector2::new(130.0, 50.0),
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
    fn test_object_lands_on_mid_air_platform() {
        let mut scene = ActTwoScene::test_sandbox();
        let obj_id = scene
            .world_mut()
            .spawn(
                EntityType::Obstacle,
                Vector2::new(ActTwoScene::PLATFORM_X, 20.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_physics_preset(PhysicsPreset::Normal)
            .with_collider(Collider::new(16.0, 16.0))
            .build();

        for _ in 0..600 {
            scene.update_physics(0.016);
        }

        // Expected resting Y for a 16x16 collider on platform:
        // platform top (PLATFORM_Y - PLATFORM_HEIGHT * 0.5) - 8.0
        let expected_y = (ActTwoScene::PLATFORM_Y - ActTwoScene::PLATFORM_HEIGHT * 0.5) - 8.0;
        let y = scene.world().positions[obj_id.0].y;
        assert!(
            (y - expected_y).abs() < 1e-3,
            "Object dropped above platform should land on platform at {}, but was at {}",
            expected_y,
            y
        );
    }

    #[test]
    fn test_player_can_jump_onto_and_rest_on_platform() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Let player land on the floor first
        for _ in 0..300 {
            scene.update_physics(0.016);
        }
        assert!(scene.is_player_grounded());
        assert!((scene.world().positions[player_id.0].y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3);

        // Jump while moving left towards the platform with Space held
        scene.handle_input(&ActTwoInput::new(true, false, true).with_jump_held(true));
        for f in 0..50 {
            if f > 0 {
                scene.handle_input(&ActTwoInput::new(true, false, false).with_jump_held(true));
            }
            scene.update_physics(0.016);
        }

        // Release input and let player settle on platform
        for _ in 0..200 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(false));
            scene.update_physics(0.016);
        }

        let landed_y = scene.world().positions[player_id.0].y;
        assert!(
            (landed_y - ActTwoScene::PLAYER_PLATFORM_RESTING_Y).abs() < 1e-3,
            "Player should have landed on platform, expected {}, got {}",
            ActTwoScene::PLAYER_PLATFORM_RESTING_Y,
            landed_y
        );
        assert!(scene.is_player_grounded(), "Player should be grounded on platform");
        assert_eq!(scene.thrust_fuel(), scene.max_thrust_duration(), "Fuel should recharge on platform");
    }

    #[test]
    fn test_player_can_jump_off_platform_and_land_on_floor() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Place player on platform
        scene.world_mut().positions[player_id.0] =
            Vector2::new(ActTwoScene::PLATFORM_X, ActTwoScene::PLAYER_PLATFORM_RESTING_Y);
        scene.world_mut().velocities[player_id.0] = Vector2::new(0.0, 0.0);
        assert!(scene.is_player_grounded());

        // Jump and move right off the platform
        scene.handle_input(&ActTwoInput::new(false, true, true).with_jump_held(false));
        for _ in 0..50 {
            scene.handle_input(&ActTwoInput::new(false, true, false).with_jump_held(false));
            scene.update_physics(0.016);
        }

        // Release input and let player land on floor
        for _ in 0..250 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(false));
            scene.update_physics(0.016);
        }

        let landed_y = scene.world().positions[player_id.0].y;
        assert!(
            (landed_y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3,
            "Player should have landed on floor, expected {}, got {}",
            ActTwoScene::PLAYER_RESTING_Y,
            landed_y
        );
        assert!(scene.is_player_grounded());
    }

    #[test]
    fn test_player_walks_under_platform_without_obstruction() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Settle on floor
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        // Walk left past the platform (platform is at x = 90, width = 60, so 60..120)
        // From x = 160 to x < 40
        for _ in 0..120 {
            scene.handle_input(&ActTwoInput::new(true, false, false));
            scene.update_physics(0.016);
        }

        let x = scene.world().positions[player_id.0].x;
        let y = scene.world().positions[player_id.0].y;
        assert!(x < 40.0, "Player should have walked completely past platform to the left");
        assert!(
            (y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3,
            "Player should still be smoothly on the floor at {}, got {}",
            ActTwoScene::PLAYER_RESTING_Y,
            y
        );
    }

    #[test]
    fn test_reset_player() {
        let mut scene = ActTwoScene::test_sandbox();
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
        assert_eq!(scene.camera().x(), 0.0);
        assert_eq!(scene.camera().y(), 0.0);
    }

    #[test]
    fn test_camera_clamped_to_x_zero_on_left() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Move player left from spawn towards x = 0
        for _ in 0..200 {
            scene.handle_input(&ActTwoInput::new(true, false, false));
            scene.update_physics(0.016);
            assert_eq!(scene.camera().x(), 0.0, "Camera left side must remain clamped to x = 0");
        }

        let half_w = ActTwoScene::PLAYER_WIDTH * 0.5;
        assert_eq!(scene.world().positions[player_id.0].x, half_w);
        assert_eq!(scene.camera().x(), 0.0);
    }

    #[test]
    fn test_camera_follows_player_moving_right() {
        let mut scene = ActTwoScene::new();

        // Move player right
        for _ in 0..100 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }

        assert!(
            scene.camera().x() > 0.0,
            "Camera should follow player to the right when moving right, camera.x = {}",
            scene.camera().x()
        );
    }

    #[test]
    fn test_camera_rubber_band_lag_and_catchup() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Place player at spawn and ensure camera at (0, 0)
        scene.world_mut().positions[player_id.0] = Vector2::new(ActTwoScene::PLAYER_SPAWN_X, 90.0);
        scene.camera_mut().set_position(Vector2::new(0.0, 0.0));

        // Move right for 60 frames (~1.0s)
        for _ in 0..60 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }

        let player_x = scene.world().positions[player_id.0].x;
        let target_x = player_x - scene.camera().viewport_width * 0.5;
        let moving_cam_x = scene.camera().x();

        // During motion, camera lags behind target (rubber-band lag)
        assert!(
            moving_cam_x < target_x,
            "Camera should lag behind target during rightward motion (rubber-band effect): cam_x={}, target_x={}",
            moving_cam_x,
            target_x
        );
        let lag_while_moving = target_x - moving_cam_x;
        assert!(lag_while_moving > 5.0, "Noticeable rubber-band lag should exist while moving");

        // Stop moving (no input): camera should smoothly catch up towards target_x
        for _ in 0..60 {
            scene.handle_input(&ActTwoInput::new(false, false, false));
            scene.update_physics(0.016);
        }

        let settled_cam_x = scene.camera().x();
        let current_target_x = scene.world().positions[player_id.0].x - scene.camera().viewport_width * 0.5;
        let lag_after_stop = (current_target_x - settled_cam_x).abs();

        assert!(
            lag_after_stop < lag_while_moving,
            "Camera should catch up after player stops: initial lag {}, settled lag {}",
            lag_while_moving,
            lag_after_stop
        );
        assert!(
            lag_after_stop < 1.0,
            "Camera should almost completely close the lag distance once player is stationary, got {}",
            lag_after_stop
        );
    }

    #[test]
    fn test_camera_follows_player_on_y_axis() {
        let mut scene = ActTwoScene::test_sandbox();
        let _player_id = scene.player_id().unwrap();

        // Initially at spawn: player y = 90.0, camera y = 0.0
        assert_eq!(scene.camera().y(), 0.0);

        // Player falls under gravity down to ground (y = 150.0)
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        // Camera should follow downwards (y increases towards 150 - 90 = 60.0)
        assert!(
            scene.camera().y() > 40.0,
            "Camera should have followed player downward on Y axis, camera.y = {}",
            scene.camera().y()
        );

        let ground_cam_y = scene.camera().y();

        // Jump and engage thrusters upward
        scene.handle_input(&ActTwoInput::new(false, false, true).with_jump_held(true));
        for _ in 0..60 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_jump_held(true));
            scene.update_physics(0.016);
        }

        // Player is climbing upward, camera y must decrease (following upward)
        assert!(
            scene.camera().y() < ground_cam_y,
            "Camera should follow player upward on Y axis when climbing with thrusters: ground_y={}, climb_y={}",
            ground_cam_y,
            scene.camera().y()
        );
    }

    #[test]
    fn test_camera_returns_and_clamps_when_player_moves_back_left() {
        let mut scene = ActTwoScene::test_sandbox();

        // Move right so camera moves to the right
        for _ in 0..120 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }
        assert!(scene.camera().x() > 50.0);

        // Move left until player returns to starting area
        for _ in 0..300 {
            scene.handle_input(&ActTwoInput::new(true, false, false));
            scene.update_physics(0.016);
        }

        // Camera must smoothly return to 0.0 and clamp to x = 0
        assert_eq!(
            scene.camera().x(),
            0.0,
            "Camera must return and clamp to x = 0 when player moves back to left side"
        );
    }

    #[test]
    fn test_camera_follow_speed_tuning() {
        let mut scene = ActTwoScene::new();
        assert_eq!(scene.camera_follow_speed(), ActTwoScene::DEFAULT_CAMERA_FOLLOW_SPEED);

        scene.set_camera_follow_speed(10.0);
        assert_eq!(scene.camera_follow_speed(), 10.0);

        // Higher speed should catch up faster when player is past viewport center
        let mut fast_scene = ActTwoScene::new();
        fast_scene.set_camera_follow_speed(15.0);
        let fast_player = fast_scene.player_id().unwrap();
        fast_scene.world_mut().positions[fast_player.0] = Vector2::new(300.0, 912.0);

        let mut slow_scene = ActTwoScene::new();
        slow_scene.set_camera_follow_speed(2.0);
        let slow_player = slow_scene.player_id().unwrap();
        slow_scene.world_mut().positions[slow_player.0] = Vector2::new(300.0, 912.0);

        for _ in 0..30 {
            fast_scene.update_physics(0.016);
            slow_scene.update_physics(0.016);
        }

        assert!(
            fast_scene.camera().x() > slow_scene.camera().x(),
            "Fast camera follow speed should track player closer than slow camera follow speed: fast={}, slow={}",
            fast_scene.camera().x(),
            slow_scene.camera().x()
        );
    }

    #[test]
    fn test_act_two_scene_from_map_file() {
        let scene = ActTwoScene::from_map_file("resources/maps/act_two/map1.json")
            .expect("Should construct ActTwoScene from map1");

        assert!(scene.player_id().is_some());
        let spawn_pos = scene.spawn_pos();
        assert_eq!(spawn_pos, Vector2::new(96.0, 912.0));
        assert_eq!(scene.player_position().unwrap(), spawn_pos);

        let map_res = scene.map_entities().expect("MapSpawnResult should be present");
        assert_eq!(map_res.platforms.len(), 18);
        assert_eq!(map_res.exits.len(), 1);
        assert_eq!(map_res.gobblers.len(), 3);
    }

    #[test]
    fn test_act_two_scene_from_map_player_navigates_and_lands() {
        let mut scene = ActTwoScene::from_map_file("resources/maps/act_two/map1.json")
            .expect("Should construct ActTwoScene from map1");

        // Simulate 2 seconds of physics - player falls from (96, 912) onto platform 22 (top y = 1024)
        // Resting y is 1024 - 10 = 1014
        for _ in 0..120 {
            scene.update_physics(1.0 / 60.0);
        }

        let landed_y = scene.player_position().unwrap().y;
        assert!(
            (landed_y - 1014.0).abs() < 1e-2,
            "Player should land on first platform at y = 1014.0, got {}",
            landed_y
        );
        assert!(scene.is_player_grounded());
    }

    #[test]
    fn test_act_two_scene_new_loads_map1() {
        let scene = ActTwoScene::new();
        assert_eq!(scene.tag(), Some(SceneTag::ActTwo));
        assert!(scene.player_id().is_some());
        assert_eq!(scene.spawn_pos(), Vector2::new(96.0, 912.0));
        assert_eq!(scene.player_position().unwrap(), Vector2::new(96.0, 912.0));
        assert_eq!(scene.camera().y(), 822.0);

        let map_res = scene.map_entities().expect("MapSpawnResult should be present on ActTwoScene::new()");
        assert_eq!(map_res.platforms.len(), 18);
        assert_eq!(map_res.exits.len(), 1);
        assert_eq!(map_res.gobblers.len(), 3);
    }

    #[test]
    fn test_player_reaches_exit_triggers_act_three() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        // Initially at spawn (96, 912), player is not at exit
        assert!(!scene.is_player_at_exit());
        assert!(scene.check_transition().is_none());

        // Move player to the center of the exit rectangle
        let map_res = scene.map_entities().expect("MapSpawnResult expected");
        assert!(!map_res.exits.is_empty(), "Map should contain at least one Exit object");
        let exit_id = map_res.exits[0];
        let exit_pos = scene.world().positions[exit_id.0];
        scene.world_mut().positions[player_id.0] = exit_pos;
        assert!(scene.is_player_at_exit(), "Player should be detected inside exit rectangle");

        let action = scene.check_transition();
        assert!(action.is_some(), "check_transition should return Some(SceneAction::Switch)");
        match action.unwrap() {
            SceneAction::Switch(next_scene) => {
                assert_eq!(next_scene.tag(), Some(SceneTag::ActThree));
            }
        }
    }

    #[test]
    fn test_act_two_exit_transition_through_scene_manager() {
        let scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        let mut scene_manager = SceneManager::new(Box::new(scene));
        assert_eq!(scene_manager.current_tag(), Some(SceneTag::ActTwo));

        // Teleport player into the exit zone
        {
            let current = scene_manager.current_scene_mut();
            let act_two = current.as_any_mut().downcast_mut::<ActTwoScene>().unwrap();
            let exit_id = act_two.map_entities().unwrap().exits[0];
            let exit_pos = act_two.world().positions[exit_id.0];
            act_two.world_mut().positions[player_id.0] = exit_pos;
            assert!(act_two.is_player_at_exit());
        }

        // Trigger transition via SceneAction
        let action = {
            let current = scene_manager.current_scene_mut();
            let act_two = current.as_any_mut().downcast_mut::<ActTwoScene>().unwrap();
            act_two.check_transition()
        };

        if let Some(SceneAction::Switch(next)) = action {
            scene_manager.switch_scene(next);
        }

        assert_eq!(scene_manager.current_tag(), Some(SceneTag::ActThree));
    }

    #[test]
    fn test_gobblers_spawned_from_map1() {
        let scene = ActTwoScene::new();
        let gobbler_ids = scene.gobbler_ids();
        assert_eq!(gobbler_ids.len(), 3, "Expected 3 Gobbler enemies from map1.json");

        for &id in gobbler_ids {
            let idx = id.0;
            assert_eq!(scene.world().tags[idx], EntityType::Gobbler);
            assert!(scene.world().tags[idx].is_enemy());
            assert!(scene.world().alive[idx]);

            // Gobbler dimensions match player rectangle (20.0 x 20.0)
            let col = scene.world().colliders[idx].expect("Gobbler should have collider");
            assert_eq!(col.width, ActTwoScene::PLAYER_WIDTH);
            assert_eq!(col.height, ActTwoScene::PLAYER_HEIGHT);
            assert!(col.is_solid);

            // Dynamic physics preset
            let phys = scene.world().physics_data[idx].as_ref().expect("Gobbler should have physics");
            assert!(!phys.is_static);
        }
    }

    #[test]
    fn test_gobbler_starts_moving_once_inside_camera_view() {
        let mut scene = ActTwoScene::new();
        let gobbler_ids = scene.gobbler_ids();
        assert_eq!(gobbler_ids.len(), 3);

        let first_gobbler = gobbler_ids[0]; // Spawns at x = 512, y = 800

        // Initially at spawn (player x = 96, camera x = 0..320):
        // Gobbler 1 at x = 512 is outside camera viewport (0..320)
        assert_eq!(scene.camera().x(), 0.0);
        assert!(!scene.world().gobbler_data[first_gobbler.0].unwrap().is_active);

        // Step physics while outside view - Gobbler should not move horizontally towards player
        scene.update_physics(0.016);
        assert_eq!(scene.world().velocities[first_gobbler.0].x, 0.0);

        // Position camera around the Gobbler so it is inside the viewport (x in [400..720], y in [750..930])
        scene.camera_mut().set_position(Vector2::new(400.0, 750.0));
        scene.update_enemies(0.016);

        // Gobbler is now inside camera view -> must activate and move towards player (left)
        assert!(scene.world().gobbler_data[first_gobbler.0].unwrap().is_active);
        assert_eq!(scene.world().velocities[first_gobbler.0].x, -ActTwoScene::GOBBLER_MOVE_SPEED);
    }

    #[test]
    fn test_gobbler_always_moves_x_wise_to_direction_of_player() {
        let mut scene = ActTwoScene::new();
        let gobbler_ids = scene.gobbler_ids();
        let gobbler_id = gobbler_ids[0];

        // Put camera around gobbler so it is active
        scene.camera_mut().set_position(Vector2::new(400.0, 750.0));
        scene.update_enemies(0.016);
        assert!(scene.world().gobbler_data[gobbler_id.0].unwrap().is_active);

        // Gobbler position
        let gobbler_x = scene.world().positions[gobbler_id.0].x;

        // Player is to the left of Gobbler
        let player_id = scene.player_id().unwrap();
        scene.world_mut().positions[player_id.0].x = gobbler_x - 100.0;
        scene.update_enemies(0.016);
        assert_eq!(scene.world().velocities[gobbler_id.0].x, -ActTwoScene::GOBBLER_MOVE_SPEED);

        // Player moves to the right of Gobbler
        scene.world_mut().positions[player_id.0].x = gobbler_x + 100.0;
        scene.update_enemies(0.016);
        assert_eq!(scene.world().velocities[gobbler_id.0].x, ActTwoScene::GOBBLER_MOVE_SPEED);
    }

    #[test]
    fn test_gobbler_falls_down_from_platform_under_gravity() {
        let mut world = World::new();
        let camera = GameCamera::at(0.0, 0.0);

        let player = world
            .spawn(
                EntityType::Player,
                Vector2::new(50.0, 50.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .build();

        // Platform from x = 100 to x = 200 at y = 100
        world
            .spawn(
                EntityType::Ground,
                Vector2::new(150.0, 100.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(100.0, 20.0))
            .with_physics(PhysicsData::static_body())
            .build();

        // Gobbler on top of platform at x = 150, resting y = 80
        let gobbler = world
            .spawn(
                EntityType::Gobbler,
                Vector2::new(150.0, 80.0),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Gobbler,
            )
            .with_collider(Collider::new(20.0, 20.0))
            .with_physics_preset(crate::components::PhysicsPreset::Normal)
            .with_gobbler(40.0)
            .build();

        // Simulate walking left towards player (x = 50) off the platform (x < 100)
        for _ in 0..120 {
            behaviour::gobbler::update(&mut world, Some(player), &camera, 0.016);
            physics::update(&mut world, 0.016);
            collision::update(&mut world, 0.016);
        }

        // Gobbler walked left off the platform edge (x < 100) and fell down (y > 80)
        assert!(world.positions[gobbler.0].x < 100.0, "Gobbler should have walked off the left edge of platform");
        assert!(world.positions[gobbler.0].y > 80.0, "Gobbler should have fallen under gravity after walking off platform");
    }

    #[test]
    fn test_act_two_mouse_click_throws_hook_and_attaches_to_platform() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Place player directly under platform at x = 90.0, resting on floor (y = 150.0)
        // Platform is at x = 90.0, y = 120.0, height = 12.0 (bottom edge at 126.0)
        // Distance is 150 - 126 = 24 px (well within the 30 px hook reach)
        scene.world_mut().positions[player_id.0] = Vector2::new(90.0, 150.0);
        assert!(!scene.is_hook_attached());

        // Mouse click on the platform:
        // Convert virtual (90, 120) to screen coords: (90 - camera.x) * 4 = 360, (120 - camera.y) * 4 = 480
        let screen_mouse = Vector2::new(90.0 * VIRTUAL_SCALE, 120.0 * VIRTUAL_SCALE);
        let input = ActTwoInput::new(false, false, false).with_hook_click(true, screen_mouse);
        scene.handle_input(&input);

        assert!(scene.is_hook_attached(), "Hook should attach to platform within 30 px reach");
        let anchor = scene.hook().anchor().unwrap();
        assert_eq!(anchor.x, 90.0);
        assert_eq!(anchor.y, 126.0); // Platform bottom edge
        assert!((scene.hook().rope_length().unwrap() - 24.0).abs() < 1e-3);
    }

    #[test]
    fn test_act_two_pull_up_lifts_player_off_ground() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Place player under platform and attach hook
        scene.world_mut().positions[player_id.0] = Vector2::new(90.0, 150.0);
        let screen_mouse = Vector2::new(90.0 * VIRTUAL_SCALE, 120.0 * VIRTUAL_SCALE);
        scene.handle_input(&ActTwoInput::new(false, false, false).with_hook_click(true, screen_mouse));
        assert!(scene.is_hook_attached());
        assert!(scene.is_player_grounded());

        // Hold Arrow Up for 0.15s to pull up on the rope
        for _ in 0..10 {
            scene.handle_input(&ActTwoInput::new(false, false, false).with_up(true));
            scene.update_physics(0.016);
        }

        let player_y = scene.world().positions[player_id.0].y;
        assert!(
            player_y < 150.0,
            "Player should have been lifted upward towards anchor, got y = {}",
            player_y
        );
        let new_len = scene.hook().rope_length().unwrap();
        assert!(new_len < 24.0, "Rope length should have shortened from 24 px, got {}", new_len);
    }

    #[test]
    fn test_act_two_swing_left_and_right_builds_momentum() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Attach hook to platform and pull up slightly so player is suspended in mid-air
        scene.world_mut().positions[player_id.0] = Vector2::new(90.0, 142.0);
        let screen_mouse = Vector2::new(90.0 * VIRTUAL_SCALE, 120.0 * VIRTUAL_SCALE);
        scene.handle_input(&ActTwoInput::new(false, false, false).with_hook_click(true, screen_mouse));
        assert!(scene.is_hook_attached());

        // Hold Arrow Right to swing right
        for _ in 0..15 {
            scene.handle_input(&ActTwoInput::new(false, true, false));
            scene.update_physics(0.016);
        }

        let vel = scene.world().velocities[player_id.0];
        assert!(
            vel.x > 20.0,
            "Player should build up rightward swing momentum, got vx = {}",
            vel.x
        );
        let pos = scene.world().positions[player_id.0];
        assert!(pos.x > 90.0, "Player should have swung right of anchor, got x = {}", pos.x);

        // Now hold Arrow Left to swing back left
        for _ in 0..25 {
            scene.handle_input(&ActTwoInput::new(true, false, false));
            scene.update_physics(0.016);
        }

        let vel_left = scene.world().velocities[player_id.0];
        assert!(
            vel_left.x < 0.0,
            "Player should swing left with momentum when holding Arrow Left, got vx = {}",
            vel_left.x
        );
    }

    #[test]
    fn test_act_two_space_dissolves_rope_and_jumps() {
        let mut scene = ActTwoScene::test_sandbox();
        let player_id = scene.player_id().unwrap();

        // Suspended on rope, swinging right with speed
        scene.world_mut().positions[player_id.0] = Vector2::new(100.0, 138.0);
        scene.world_mut().velocities[player_id.0] = Vector2::new(150.0, -40.0);
        let screen_mouse = Vector2::new(90.0 * VIRTUAL_SCALE, 120.0 * VIRTUAL_SCALE);
        scene.handle_input(&ActTwoInput::new(false, false, false).with_hook_click(true, screen_mouse));
        assert!(scene.is_hook_attached());

        // Press Space to release rope and jump
        let jump_input = ActTwoInput::new(false, false, true);
        scene.handle_input(&jump_input);
        scene.update_physics(0.016);

        // Rope dissolved!
        assert!(!scene.is_hook_attached(), "Rope must dissolve when Space is pressed");
        // Player launched upward!
        let vel = scene.world().velocities[player_id.0];
        assert!(
            vel.y <= -ActTwoScene::PLAYER_JUMP_IMPULSE * 0.75,
            "Player should jump upward upon rope dissolving, got vy = {}",
            vel.y
        );
        assert!(
            vel.x > 50.0,
            "Horizontal swing momentum must be preserved on release, got vx = {}",
            vel.x
        );
    }

    #[test]
    fn test_act_two_missed_hook_does_not_attach() {
        let mut scene = ActTwoScene::test_sandbox();
        let _player_id = scene.player_id().unwrap();

        // Player at spawn (160, 90). Clicking into empty air to the right (x=200, y=90)
        // No platform or wall within 30 px
        let screen_mouse = Vector2::new(200.0 * VIRTUAL_SCALE, 90.0 * VIRTUAL_SCALE);
        let input = ActTwoInput::new(false, false, false).with_hook_click(true, screen_mouse);
        scene.handle_input(&input);

        assert!(!scene.is_hook_attached(), "Hook should not attach when nothing is in range");
        assert!(matches!(scene.hook().state, HookState::Missed { .. }));
    }

    #[test]
    fn test_act_two_loads_tiles_layer_and_tileset_path() {
        let scene = ActTwoScene::new();
        assert_eq!(
            scene.tileset_image_path(),
            Some("resources/graphics/Platforms.png")
        );
        let map = scene.map().expect("ActTwoScene::new() should retain loaded Map");
        assert_eq!(map.tile_layers().count(), 1);
        let tiles_layer = map.tile_layers().next().unwrap();
        assert_eq!(tiles_layer.name, "Tiles");
        assert!(tiles_layer.non_empty_tiles().count() > 0);
    }
}

