use crate::prelude::*;
use crate::scenes::asteroid_field::{Asteroid, AsteroidField};
use crate::scenes::{Scene, SceneAction};
use crate::systems::collision;
use sola_raylib::prelude::*;

// Re-export camera pan types and asteroid types for backwards compatibility
pub use crate::camera::{ActOnePhase, IntroPan, IntroPhase};
pub use crate::scenes::asteroid_field::{Asteroid as ActOneAsteroid, AsteroidField as ActOneAsteroidField};

/// Act One / Level 1 scene featuring the calm night skyline, drifting asteroids, and parked player ship.
pub struct ActOneScene {
    world: World,
    player_id: Option<EntityID>,
    ground_id: Option<EntityID>,
    asteroid_field: AsteroidField,
    time: f32,
    pan: IntroPan,
    pub camera: GameCamera,
}

impl ActOneScene {
    pub const HOLD_DURATION: f32 = 6.0;
    pub const PAN_SPEED: f32 = 25.0;
    pub const MAX_PAN: f32 = 320.0; // 640 foreground width - 320 camera viewport width

    // Player spaceship dimensions in 320x180 virtual pixels.
    // Scales down the 1882x836 original texture proportionally (1882 / 836 ≈ 2.25).
    pub const PLAYER_WIDTH: f32 = 78.0;
    pub const PLAYER_HEIGHT: f32 = 34.0;
    /// Parked position on the ground on the right side of the 640px background panorama.
    pub const PLAYER_SPAWN_X: f32 = 560.0;
    pub const PLAYER_SPAWN_Y: f32 = 166.0;

    /// Ground collider dimensions and position underneath the parked spaceship.
    /// Top of ground rectangle = GROUND_Y - GROUND_HEIGHT * 0.5 = 203.0 - 20.0 = 183.0.
    /// Bottom of spaceship = PLAYER_SPAWN_Y + PLAYER_HEIGHT * 0.5 = 166.0 + 17.0 = 183.0.
    pub const GROUND_X: f32 = 560.0;
    pub const GROUND_Y: f32 = 203.0;
    pub const GROUND_WIDTH: f32 = 200.0;
    pub const GROUND_HEIGHT: f32 = 40.0;

    pub const SKY_TOP: f32 = AsteroidField::SKY_TOP;
    pub const SKY_BOTTOM: f32 = AsteroidField::SKY_BOTTOM;

    pub fn new() -> Self {
        let mut world = World::new();
        let player_id = world
            .spawn(
                EntityType::Player,
                Vector2::new(Self::PLAYER_SPAWN_X, Self::PLAYER_SPAWN_Y),
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
            .with_physics_preset(PhysicsPreset::Light)
            .with_collider(Collider::new(Self::PLAYER_WIDTH, Self::PLAYER_HEIGHT))
            .build();

        let ground_id = world
            .spawn(
                EntityType::Ground,
                Vector2::new(Self::GROUND_X, Self::GROUND_Y),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(Self::GROUND_WIDTH, Self::GROUND_HEIGHT))
            .with_physics(PhysicsData::static_body())
            .build();

        Self {
            world,
            player_id: Some(player_id),
            ground_id: Some(ground_id),
            asteroid_field: AsteroidField::new(),
            time: 0.0,
            pan: IntroPan::new(Self::HOLD_DURATION, Self::PAN_SPEED, Self::MAX_PAN),
            camera: GameCamera::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn player_id(&self) -> Option<EntityID> {
        self.player_id
    }

    pub fn player_position(&self) -> Option<Vector2> {
        self.player_id.map(|id| self.world.positions[id.0])
    }

    pub fn ground_id(&self) -> Option<EntityID> {
        self.ground_id
    }

    pub fn ground_position(&self) -> Option<Vector2> {
        self.ground_id.map(|id| self.world.positions[id.0])
    }

    pub fn pan(&self) -> &IntroPan {
        &self.pan
    }

    pub fn camera(&self) -> &GameCamera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut GameCamera {
        &mut self.camera
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn phase(&self) -> ActOnePhase {
        self.pan.phase(self.time)
    }

    pub fn is_pan_finished(&self) -> bool {
        self.phase() == ActOnePhase::Stationary
    }

    pub fn asteroid_field(&self) -> &AsteroidField {
        &self.asteroid_field
    }

    pub fn asteroid_field_mut(&mut self) -> &mut AsteroidField {
        &mut self.asteroid_field
    }

    pub fn asteroids(&self) -> &[Asteroid] {
        self.asteroid_field.asteroids()
    }

    pub fn update_asteroids(&mut self, dt: f32) {
        let parallax_cam_x = self.camera.x() * VIRTUAL_SCALE * 0.25;
        self.asteroid_field.update(dt, parallax_cam_x);
    }

    pub fn update_physics(&mut self, dt: f32) {
        physics::update(&mut self.world, dt);
        collision::update(&mut self.world, dt);
    }

    pub fn update_input(&mut self, input: &PlayerInput, dt: f32) {
        crate::systems::input::update(&mut self.world, input, dt);
    }
}

impl Default for ActOneScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for ActOneScene {
    fn on_enter(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
        if let Err(e) = resources.load_texture(
            rl,
            thread,
            SpriteID::LevelOneBackground,
            "resources/graphics/act_one_blackbackground.png",
        ) {
            eprintln!("Failed to load act one background: {}", e);
        }

        if let Err(e) = resources.load_texture(
            rl,
            thread,
            SpriteID::ActOneForeground,
            "resources/graphics/act_one_foreground.png",
        ) {
            eprintln!("Failed to load act one foreground: {}", e);
        }

        if let Err(e) = resources.load_texture(rl, thread, SpriteID::Player, "resources/graphics/spaceship2.png") {
            eprintln!("Failed to load spaceship player texture: {}", e);
        }
    }

    fn update(&mut self, ctx: &mut Context) -> Option<SceneAction> {
        ctx.audio.play_music(MusicID::Level1_Theme);

        self.time += ctx.dt;
        self.camera.set_x(self.pan.camera_x(self.time));

        let parallax_cam_x = self.camera.x() * VIRTUAL_SCALE * 0.25;
        self.asteroid_field.update(ctx.dt, parallax_cam_x);

        let input = PlayerInput::from_raylib(ctx.rl);
        self.update_input(&input, ctx.dt);

        self.update_physics(ctx.dt);

        // Ceiling clamp so spaceship cannot fly off top of screen
        if let Some(player_id) = self.player_id {
            if self.world.alive[player_id.0] {
                let min_y = Self::PLAYER_HEIGHT * 0.5;
                if self.world.positions[player_id.0].y < min_y {
                    self.world.positions[player_id.0].y = min_y;
                    if self.world.velocities[player_id.0].y < 0.0 {
                        self.world.velocities[player_id.0].y = 0.0;
                    }
                }
            }
        }

        if self.is_pan_finished() {
            // Gameplay hooks will start here
        }

        None
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
        // 1. Draw the dark night sky background
        // act_one_blackbackground is 320x180, filling the 320x180 camera viewport exactly
        if let Some(tex) = resources.get_texture(&SpriteID::LevelOneBackground) {
            let source = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);
            let dest = self.camera.canvas_dest_rect();
            canvas.draw_texture_pro(tex, source, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
        }

        // 2. Render scattered asteroids traveling along low-trajectory diagonal paths
        self.asteroid_field.draw(canvas, &self.camera);

        // 3. Draw foreground (hills / skyline) over the asteroids
        // The texture is 640x180. The camera samples a 320x180 viewport window from the texture,
        // and projects it to the 1280x720 canvas (exact 4x integer scaling, no squeezing!).
        if let Some(tex) = resources.get_texture(&SpriteID::ActOneForeground) {
            let source = self.camera.source_rect_clamped(tex.width() as f32, tex.height() as f32);
            let dest = self.camera.canvas_dest_rect();
            canvas.draw_texture_pro(tex, source, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
        }

        // 4. Draw player spaceship entity
        // The spaceship is scaled down from 1882x836 to PLAYER_WIDTH x PLAYER_HEIGHT (36x16 virtual px)
        if let Some(player_id) = self.player_id {
            if self.world.alive[player_id.0] {
                if let Some(tex) = resources.get_texture(&SpriteID::Player) {
                    let pos = self.world.positions[player_id.0];
                    let rotation = self.world.rotations[player_id.0];

                    let source = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);

                    let screen_x = self.camera.to_screen_x(pos.x * VIRTUAL_SCALE, 1.0);
                    let screen_y = pos.y * VIRTUAL_SCALE;

                    let dest = Rectangle::new(
                        screen_x,
                        screen_y,
                        Self::PLAYER_WIDTH * VIRTUAL_SCALE,
                        Self::PLAYER_HEIGHT * VIRTUAL_SCALE,
                    );
                    let origin = Vector2::new(dest.width * 0.5, dest.height * 0.5);

                    canvas.draw_texture_pro(tex, source, dest, origin, rotation, Color::WHITE);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intro_pan_sequence() {
        let pan = IntroPan::new(6.0, 25.0, 320.0);
        assert_eq!(pan.camera_x(3.0), 0.0);
        assert_eq!(pan.phase(3.0), ActOnePhase::InitialHold);
        assert_eq!(pan.camera_x(10.0), 100.0);
        assert_eq!(pan.phase(10.0), ActOnePhase::Panning);
        assert_eq!(pan.camera_x(25.0), 320.0);
        assert_eq!(pan.phase(25.0), ActOnePhase::Stationary);
        assert!((pan.finish_time() - 18.8).abs() < 1e-4);
    }

    #[test]
    fn test_act_one_scene_pan_delegation() {
        let mut scene = ActOneScene::new();
        assert_eq!(scene.phase(), ActOnePhase::InitialHold);
        assert!(!scene.is_pan_finished());

        scene.set_time(3.0);
        let x = scene.pan().camera_x(scene.time());
        scene.camera_mut().set_x(x);
        assert_eq!(scene.camera().x(), 0.0);
        assert_eq!(scene.phase(), ActOnePhase::InitialHold);

        scene.set_time(10.0);
        let x = scene.pan().camera_x(scene.time());
        scene.camera_mut().set_x(x);
        assert_eq!(scene.camera().x(), 100.0);
        assert_eq!(scene.phase(), ActOnePhase::Panning);

        scene.set_time(25.0);
        let x = scene.pan().camera_x(scene.time());
        scene.camera_mut().set_x(x);
        assert_eq!(scene.camera().x(), 320.0);
        assert_eq!(scene.phase(), ActOnePhase::Stationary);
        assert!(scene.is_pan_finished());
    }

    #[test]
    fn test_act_one_asteroids_initialization() {
        let scene = ActOneScene::new();
        assert_eq!(
            scene.asteroids().len(),
            225,
            "Expected exactly 225 scattered asteroids"
        );

        for a in scene.asteroids() {
            assert!(a.width >= 1 && a.width <= 6, "Asteroid width must be 1..=6");
            assert!(a.height >= 1 && a.height <= 6, "Asteroid height must be 1..=6");
            assert!(
                a.base_y >= ActOneScene::SKY_TOP && a.base_y <= ActOneScene::SKY_BOTTOM,
                "Asteroid base_y in starry sky range"
            );
            assert!(a.speed_x > 0.0, "Asteroid must travel leftward");
            assert!(a.speed_y > 0.0, "Asteroid must travel downward");
            let slope = a.speed_y / a.speed_x;
            assert!(
                slope >= 0.15 && slope <= 0.35,
                "Asteroid must follow low diagonal trajectory"
            );
        }
    }

    #[test]
    fn test_act_one_asteroids_diagonal_motion() {
        let mut scene = ActOneScene::new();
        let initial_positions: Vec<(f32, f32)> = scene
            .asteroids()
            .iter()
            .map(|a| (a.base_x, a.base_y))
            .collect();

        // Advance 1 second
        scene.update_asteroids(1.0);

        let mut checked_count = 0;
        for (i, a) in scene.asteroids().iter().enumerate() {
            let (init_x, init_y) = initial_positions[i];
            // Asteroids that did not wrap across wrap boundaries moved along straight diagonal vector
            if a.base_x < init_x && a.base_y > init_y {
                let dx = init_x - a.base_x;
                let dy = a.base_y - init_y;
                assert!(dx > 0.0, "Must travel leftward");
                assert!(dy > 0.0, "Must travel downward");
                let slope = dy / dx;
                assert!(
                    slope >= 0.15 && slope <= 0.35,
                    "Must keep low downward trajectory"
                );
                checked_count += 1;
            }
        }
        assert!(checked_count > 200, "Most asteroids should advance without wrapping in 1s");
    }

    #[test]
    fn test_act_one_camera() {
        let mut scene = ActOneScene::new();
        assert_eq!(scene.camera().x(), 0.0);
        scene.camera_mut().set_x(120.0);
        assert_eq!(scene.camera().x(), 120.0);
    }

    #[test]
    fn test_act_one_asteroids_cover_camera_panning() {
        let mut scene = ActOneScene::new();
        // At camera x = 320.0 (maximum foreground pan)
        scene.camera_mut().set_x(320.0);
        let parallax_cam_x = scene.camera().x() * VIRTUAL_SCALE * 0.25;

        let screen_min = 0.0;
        let screen_max = GAME_WIDTH as f32;

        let has_left_coverage = scene.asteroids().iter().any(|a| {
            let sx = a.base_x - parallax_cam_x;
            sx <= screen_min && sx >= -150.0
        });
        let has_right_coverage = scene.asteroids().iter().any(|a| {
            let sx = a.base_x - parallax_cam_x;
            sx >= screen_max && sx <= screen_max + 2000.0
        });

        assert!(
            has_left_coverage,
            "Asteroids must cover beyond the left edge at max camera pan"
        );
        assert!(
            has_right_coverage,
            "Asteroids must cover beyond the right edge at max camera pan"
        );
    }

    #[test]
    fn test_act_one_player_initialization() {
        let scene = ActOneScene::new();
        assert!(scene.player_id().is_some());
        let id = scene.player_id().unwrap();
        assert_eq!(scene.world().tags[id.0], EntityType::Player);
        assert!(scene.world().alive[id.0]);
        assert_eq!(scene.world().sprite_ids[id.0], SpriteID::Player);
        assert!(scene.world().movement_data[id.0].is_some());
        let pos = scene.player_position().unwrap();
        assert_eq!(pos.x, ActOneScene::PLAYER_SPAWN_X);
        assert_eq!(pos.y, ActOneScene::PLAYER_SPAWN_Y);
    }

    #[test]
    fn test_act_one_player_dimensions() {
        assert_eq!(ActOneScene::PLAYER_WIDTH, 78.0);
        assert_eq!(ActOneScene::PLAYER_HEIGHT, 34.0);
        let ratio = ActOneScene::PLAYER_WIDTH / ActOneScene::PLAYER_HEIGHT;
        let expected_ratio = 1882.0 / 836.0;
        assert!((ratio - expected_ratio).abs() < 0.05);
    }

    #[test]
    fn test_act_one_player_stays_fixed_and_hidden_at_start() {
        let scene = ActOneScene::new();
        let initial_pos = scene.player_position().unwrap();

        // Player must start offscreen to the right of the initial camera viewport (320px)
        let player_left_edge = initial_pos.x - ActOneScene::PLAYER_WIDTH * 0.5;
        assert!(
            player_left_edge > scene.camera().viewport_width,
            "Player must not be visible within the initial 320px viewport"
        );
    }

    #[test]
    fn test_act_one_ground_initialization() {
        let scene = ActOneScene::new();
        assert!(scene.ground_id().is_some());
        let id = scene.ground_id().unwrap();
        assert_eq!(scene.world().tags[id.0], EntityType::Ground);
        assert!(scene.world().alive[id.0]);
        assert_eq!(scene.world().sprite_ids[id.0], SpriteID::None);
        let pos = scene.ground_position().unwrap();
        assert_eq!(pos.x, ActOneScene::GROUND_X);
        assert_eq!(pos.y, ActOneScene::GROUND_Y);

        let ground_col = scene.world().colliders[id.0].unwrap();
        assert_eq!(ground_col.width, ActOneScene::GROUND_WIDTH);
        assert_eq!(ground_col.height, ActOneScene::GROUND_HEIGHT);
        assert!(ground_col.is_solid);

        let ground_physics = scene.world().physics_data[id.0].as_ref().unwrap();
        assert!(ground_physics.is_static);
    }

    #[test]
    fn test_act_one_player_has_physics_and_collider() {
        let scene = ActOneScene::new();
        let id = scene.player_id().unwrap();
        let player_physics = scene.world().physics_data[id.0].as_ref().unwrap();
        assert!(!player_physics.is_static);
        assert_eq!(player_physics.gravity_scale, PhysicsData::LIGHT_GRAVITY_SCALE);
        assert_eq!(player_physics.max_fall_speed, Some(PhysicsData::LIGHT_MAX_FALL_SPEED));

        let player_col = scene.world().colliders[id.0].unwrap();
        assert_eq!(player_col.width, ActOneScene::PLAYER_WIDTH);
        assert_eq!(player_col.height, ActOneScene::PLAYER_HEIGHT);
        assert!(player_col.is_solid);
    }

    #[test]
    fn test_act_one_player_rests_on_ground_without_falling() {
        let mut scene = ActOneScene::new();
        let player_id = scene.player_id().unwrap();

        // Simulate 5 seconds of physics (300 frames of 60fps)
        for _ in 0..300 {
            scene.update_physics(1.0 / 60.0);
        }

        let pos = scene.world().positions[player_id.0];
        assert_eq!(pos.x, ActOneScene::PLAYER_SPAWN_X);
        // Player should still be parked at 166.0 (not fallen through the ground)
        assert!(
            (pos.y - ActOneScene::PLAYER_SPAWN_Y).abs() < 1e-3,
            "Player y position was {}, expected {}",
            pos.y,
            ActOneScene::PLAYER_SPAWN_Y
        );
        assert_eq!(scene.world().velocities[player_id.0].y, 0.0);
    }

    #[test]
    fn test_player_falls_through_without_ground_collider() {
        let mut scene = ActOneScene::new();
        let player_id = scene.player_id().unwrap();

        // Remove the ground collider to simulate absence of ground rectangle
        let ground_id = scene.ground_id().unwrap();
        scene.world_mut().alive[ground_id.0] = false;

        // Simulate 10 seconds of physics with light floaty gravity
        for _ in 0..600 {
            scene.update_physics(1.0 / 60.0);
        }

        let pos = scene.world().positions[player_id.0];
        // Without ground, gravity pulls player down well past spawn y
        assert!(
            pos.y > ActOneScene::PLAYER_SPAWN_Y + 40.0,
            "Player should have fallen through the screen without ground collider, got {}",
            pos.y
        );
    }

    #[test]
    fn test_player_dropped_from_above_lands_on_ground() {
        let mut scene = ActOneScene::new();
        let player_id = scene.player_id().unwrap();

        // Start player above the ground in mid-air
        scene.world_mut().positions[player_id.0].y = 150.0;

        // Simulate 6 seconds of physics until it lands and settles
        for _ in 0..360 {
            scene.update_physics(1.0 / 60.0);
        }

        let pos = scene.world().positions[player_id.0];
        assert!(
            (pos.y - ActOneScene::PLAYER_SPAWN_Y).abs() < 1e-3,
            "Dropped player should land and settle at PLAYER_SPAWN_Y ({}), but was at {}",
            ActOneScene::PLAYER_SPAWN_Y,
            pos.y
        );
        assert_eq!(scene.world().velocities[player_id.0].y, 0.0);
    }

    #[test]
    fn test_act_one_arrow_key_up_lifts_player() {
        let mut scene = ActOneScene::new();
        let player_id = scene.player_id().unwrap();

        let up_input = PlayerInput {
            up: true,
            ..Default::default()
        };

        // Simulate 1 second of holding Arrow-Key Up
        for _ in 0..60 {
            scene.update_input(&up_input, 1.0 / 60.0);
            scene.update_physics(1.0 / 60.0);
        }

        let pos = scene.world().positions[player_id.0];
        // Player should have lifted above ground (pos.y < 166.0)
        assert!(
            pos.y < ActOneScene::PLAYER_SPAWN_Y - 10.0,
            "Player should have lifted off ground, expected y < {}, got {}",
            ActOneScene::PLAYER_SPAWN_Y - 10.0,
            pos.y
        );
        assert!(scene.world().velocities[player_id.0].y < 0.0);
    }

    #[test]
    fn test_act_one_release_thrust_gravity_pulls_down() {
        let mut scene = ActOneScene::new();
        let player_id = scene.player_id().unwrap();

        let up_input = PlayerInput {
            up: true,
            ..Default::default()
        };

        // 1. Lift player up for 1 second
        for _ in 0..60 {
            scene.update_input(&up_input, 1.0 / 60.0);
            scene.update_physics(1.0 / 60.0);
        }

        let lifted_y = scene.world().positions[player_id.0].y;
        assert!(lifted_y < ActOneScene::PLAYER_SPAWN_Y);

        // 2. Release thrust: simulate 10 seconds of gentle floaty gravity pulling down until landed
        let no_input = PlayerInput::default();
        for _ in 0..600 {
            scene.update_input(&no_input, 1.0 / 60.0);
            scene.update_physics(1.0 / 60.0);
        }

        let final_pos = scene.world().positions[player_id.0];
        // Should have gently landed back on ground at PLAYER_SPAWN_Y
        assert!(
            (final_pos.y - ActOneScene::PLAYER_SPAWN_Y).abs() < 1e-3,
            "Player should have settled back onto ground, expected {}, got {}",
            ActOneScene::PLAYER_SPAWN_Y,
            final_pos.y
        );
        assert_eq!(scene.world().velocities[player_id.0].y, 0.0);
    }

    #[test]
    fn test_act_one_arrow_key_down_descends_faster() {
        let mut scene_gravity = ActOneScene::new();
        let mut scene_down = ActOneScene::new();

        let p_grav = scene_gravity.player_id().unwrap();
        let p_down = scene_down.player_id().unwrap();

        // Put both players in the air at y = 80.0
        scene_gravity.world_mut().positions[p_grav.0].y = 80.0;
        scene_down.world_mut().positions[p_down.0].y = 80.0;

        let no_input = PlayerInput::default();
        let down_input = PlayerInput {
            down: true,
            ..Default::default()
        };

        // Simulate 0.5s
        for _ in 0..30 {
            scene_gravity.update_input(&no_input, 1.0 / 60.0);
            scene_gravity.update_physics(1.0 / 60.0);

            scene_down.update_input(&down_input, 1.0 / 60.0);
            scene_down.update_physics(1.0 / 60.0);
        }

        let vel_grav = scene_gravity.world().velocities[p_grav.0].y;
        let vel_down = scene_down.world().velocities[p_down.0].y;

        assert!(
            vel_down > vel_grav * 1.8,
            "Down input descent velocity ({}) should be significantly faster than gravity alone ({})",
            vel_down,
            vel_grav
        );
        assert!(scene_down.world().positions[p_down.0].y > scene_gravity.world().positions[p_grav.0].y);
    }
}
