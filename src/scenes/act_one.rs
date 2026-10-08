use crate::prelude::*;
use crate::scenes::asteroid_field::{Asteroid, AsteroidField};
use crate::scenes::{Scene, SceneAction};
use sola_raylib::prelude::*;

// Re-export camera pan types and asteroid types for backwards compatibility
pub use crate::camera::{ActOnePhase, IntroPan, IntroPhase};
pub use crate::scenes::asteroid_field::{Asteroid as ActOneAsteroid, AsteroidField as ActOneAsteroidField};

/// Act One / Level 1 scene featuring the calm night skyline, drifting asteroids, and parked player ship.
pub struct ActOneScene {
    world: World,
    player_id: Option<EntityID>,
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
                player_stats::THRUST_POWER,
                player_stats::MAX_SPEED,
                player_stats::ROTATION_SPEED,
            ))
            .build();

        Self {
            world,
            player_id: Some(player_id),
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
}
