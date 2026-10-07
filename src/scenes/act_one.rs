use crate::prelude::*;
use crate::scenes::{Scene, SceneAction};
use sola_raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct Asteroid {
    pub base_x: f32,
    pub base_y: f32,
    pub width: i32,
    pub height: i32,
    pub phase: f32,
    pub amplitude: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActOnePhase {
    /// 0..6s: Camera holds steady at the left edge of the skyline.
    InitialHold,
    /// 6s..18.8s: Camera glides smoothly rightward across the panorama.
    Panning,
    /// 18.8s+: Camera reached the right boundary of the 640px background and stays steady.
    Stationary,
}

/// Pure, deterministic intro camera controller: hold, then pan, then stop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntroPan {
    pub hold: f32,
    pub speed: f32,
    pub max_x: f32,
}

impl IntroPan {
    pub const fn new(hold: f32, speed: f32, max_x: f32) -> Self {
        Self { hold, speed, max_x }
    }

    pub fn camera_x(&self, t: f32) -> f32 {
        ((t - self.hold).max(0.0) * self.speed).min(self.max_x)
    }

    pub fn phase(&self, t: f32) -> ActOnePhase {
        if t < self.hold {
            ActOnePhase::InitialHold
        } else if self.camera_x(t) < self.max_x {
            ActOnePhase::Panning
        } else {
            ActOnePhase::Stationary
        }
    }

    pub fn finish_time(&self) -> f32 {
        self.hold + self.max_x / self.speed
    }
}

/// Act One / Level 1 scene featuring the calm night skyline and gently undulating asteroids.
pub struct ActOneScene {
    world: World,
    player_id: Option<EntityID>,
    asteroids: Vec<Asteroid>,
    time: f32,
    pan: IntroPan,
    pub camera: GameCamera,
}

impl ActOneScene {
    pub const HOLD_DURATION: f32 = 6.0;
    pub const PAN_SPEED: f32 = 25.0;
    pub const MAX_PAN: f32 = 320.0; // 640 foreground width - 320 camera viewport width

    // Player spaceship dimensions in 320x180 virtual pixels.
    // 36x16 scales down the 1882x836 original texture proportionally (1882 / 836 ≈ 2.25).
    pub const PLAYER_WIDTH: f32 = 78.0;
    pub const PLAYER_HEIGHT: f32 = 34.0;

    pub fn new() -> Self {
        let mut world = World::new();
        let player_id = world
            .spawn(
                EntityType::Player,
                Vector2::new(400.0, 90.0),
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
            asteroids: Self::init_asteroids(),
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

    pub fn asteroids(&self) -> &[Asteroid] {
        &self.asteroids
    }

    const ASTEROID_COUNT: usize = 150;
    const ASTEROID_SPACING: f32 = 22.0;
    const ASTEROID_WRAP_SPAN: f32 = Self::ASTEROID_COUNT as f32 * Self::ASTEROID_SPACING; // 3300.0
    const ASTEROID_START_X: f32 = -100.0;
    const ASTEROID_DRIFT_SPEED: f32 = 25.0;

    fn init_asteroids() -> Vec<Asteroid> {
        // 150 core pearls + 75 companion micro-specks (one for every even index) = exactly 225
        const TOTAL_CAPACITY: usize = ActOneScene::ASTEROID_COUNT + ActOneScene::ASTEROID_COUNT / 2;
        let mut list = Vec::with_capacity(TOTAL_CAPACITY);

        // Warm retro palette: yellow, orange, red, and muted ochre shades
        let colors = [
            Color::new(255, 245, 175, 255), // light cream / pearl gold
            Color::new(255, 215, 60, 255),  // warm yellow
            Color::new(250, 185, 45, 255),  // golden amber
            Color::new(245, 135, 40, 255),  // vibrant orange
            Color::new(225, 75, 35, 255),   // deep orange-red
            Color::new(200, 125, 50, 255),  // muted ochre
            Color::new(175, 55, 35, 255),   // dark terracotta / ruby
        ];

        let sizes = [(4, 4), (6, 6), (4, 6), (6, 4), (4, 4), (2, 2), (4, 4)];

        for i in 0..Self::ASTEROID_COUNT {
            let x = Self::ASTEROID_START_X + i as f32 * Self::ASTEROID_SPACING;
            // Baseline follows a gentle arch across the starry sky (y: ~180 to ~230)
            let arch = (i as f32 * std::f32::consts::TAU / Self::ASTEROID_COUNT as f32).sin() * 25.0;
            let base_y = 205.0 + arch;
            let (w, h) = sizes[i % sizes.len()];
            let color = colors[i % colors.len()];
            let phase = i as f32 * 0.12;
            let amplitude = 22.0 + (i as f32 * 0.25).sin() * 5.0;

            // Core pearl bead
            list.push(Asteroid {
                base_x: x,
                base_y,
                width: w,
                height: h,
                phase,
                amplitude,
                color,
            });

            // Companion micro-cluster specks around select pearls
            if i % 2 == 0 {
                let offset_x = if i % 4 == 0 { 4.0 } else { -4.0 };
                let offset_y = if i % 4 == 0 { 3.0 } else { -3.0 };
                let speck_color = colors[(i + 3) % colors.len()];
                let (sw, sh) = if i % 3 == 0 { (2, 2) } else { (3, 2) };

                list.push(Asteroid {
                    base_x: x + offset_x,
                    base_y: base_y + offset_y,
                    width: sw,
                    height: sh,
                    phase: phase + 0.08,
                    amplitude,
                    color: speck_color,
                });
            }
        }

        list
    }

    /// Updates asteroid positions, gently drifting and wrapping seamlessly around the camera.
    pub fn update_asteroids(&mut self, dt: f32) {
        let parallax_cam_x = self.camera.x() * VIRTUAL_SCALE * 0.25;
        let wrap_bound = parallax_cam_x - 120.0;
        let wrap_max = wrap_bound + Self::ASTEROID_WRAP_SPAN;

        for a in &mut self.asteroids {
            a.base_x -= Self::ASTEROID_DRIFT_SPEED * dt;
            while a.base_x < wrap_bound {
                a.base_x += Self::ASTEROID_WRAP_SPAN;
            }
            while a.base_x > wrap_max {
                a.base_x -= Self::ASTEROID_WRAP_SPAN;
            }
        }
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

        if let Err(e) = resources.load_texture(rl, thread, SpriteID::Player, "resources/graphics/spaceship.png") {
            eprintln!("Failed to load spaceship player texture: {}", e);
        }
    }

    fn update(&mut self, ctx: &mut Context) -> Option<SceneAction> {
        ctx.audio.play_music(MusicID::Level1_Theme);

        self.time += ctx.dt;
        self.camera.set_x(self.pan.camera_x(self.time));
        self.update_asteroids(ctx.dt);

        // Keep player ship cruising along with the camera pan in virtual coordinates
        if let Some(player_id) = self.player_id {
            // Screen x = 50.0, with a gentle hovering undulation
            let hover = (self.time * 2.5).sin() * 1.5;
            self.world.positions[player_id.0].x = self.camera.x() + 50.0;
            self.world.positions[player_id.0].y = 90.0 + hover;
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

        // 2. Render gently moving string of pearls asteroids
        // Wave number matches wrap span so sinusoidal wrapping is seamless:
        // k = (2 * 2pi) / ASTEROID_WRAP_SPAN
        const K: f32 = (2.0 * std::f32::consts::TAU) / ActOneScene::ASTEROID_WRAP_SPAN;
        const OMEGA: f32 = 1.4;

        for a in &self.asteroids {
            let current_x = self.camera.to_screen_x(a.base_x, 0.25).round();
            let wave = (self.time * OMEGA + a.base_x * K + a.phase).sin();
            let current_y = (a.base_y + wave * a.amplitude).round();

            canvas.draw_rectangle(current_x as i32, current_y as i32, a.width, a.height, a.color);
        }

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
            scene.asteroids.len(),
            225,
            "Expected exactly 225 asteroids (150 pearls + 75 companion specks)"
        );

        for a in &scene.asteroids {
            assert!(a.width >= 1 && a.width <= 6, "Asteroid width must be 1..=6");
            assert!(a.height >= 1 && a.height <= 6, "Asteroid height must be 1..=6");
            assert!(a.base_y >= 150.0 && a.base_y <= 300.0, "Asteroid base_y in starry sky range");
            assert!(a.amplitude > 0.0, "Asteroid must have positive wave amplitude");
        }
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

        let has_left_coverage = scene.asteroids.iter().any(|a| {
            let sx = a.base_x - parallax_cam_x;
            sx <= screen_min && sx >= -150.0
        });
        let has_right_coverage = scene.asteroids.iter().any(|a| {
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
        assert_eq!(pos.x, 400.0);
        assert_eq!(pos.y, 90.0);
    }

    #[test]
    fn test_act_one_player_dimensions() {
        assert_eq!(ActOneScene::PLAYER_WIDTH, 50.0);
        assert_eq!(ActOneScene::PLAYER_HEIGHT, 22.0);
        let ratio = ActOneScene::PLAYER_WIDTH / ActOneScene::PLAYER_HEIGHT;
        let expected_ratio = 1882.0 / 836.0;
        assert!((ratio - expected_ratio).abs() < 0.05);
    }
}
