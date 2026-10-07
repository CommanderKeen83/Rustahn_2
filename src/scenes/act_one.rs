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

/// Act One / Level 1 scene featuring the calm night skyline and gently undulating asteroids.
pub struct ActOneScene {
    asteroids: Vec<Asteroid>,
    time: f32,
    pub camera: GameCamera,
}

impl ActOneScene {
    pub const HOLD_DURATION: f32 = 6.0;
    pub const PAN_SPEED: f32 = 25.0;
    pub const MAX_PAN: f32 = 320.0; // 640 foreground width - 320 camera viewport width

    pub fn new() -> Self {
        Self {
            asteroids: Self::init_asteroids(),
            time: 0.0,
            camera: GameCamera::new(),
        }
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
        if self.time < Self::HOLD_DURATION {
            ActOnePhase::InitialHold
        } else if self.camera.x() < Self::MAX_PAN {
            ActOnePhase::Panning
        } else {
            ActOnePhase::Stationary
        }
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
        let mut list = Vec::with_capacity(250);

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
    }

    fn update(&mut self, ctx: &mut Context) -> Option<SceneAction> {
        ctx.audio.play_music(MusicID::Level1_Theme);

        self.time += ctx.dt;

        // Story camera sequence:
        // 1. Holds steady at x = 0 for 6 seconds (initial overview of the left vista).
        // 2. Pans right at PAN_SPEED (25 px/s).
        // 3. Stops when the right edge of the camera view reaches the end of the 640px background (camera.x = 320).
        let cam_x = if self.time <= Self::HOLD_DURATION {
            0.0
        } else {
            let elapsed_pan = self.time - Self::HOLD_DURATION;
            (elapsed_pan * Self::PAN_SPEED).min(Self::MAX_PAN)
        };
        self.camera.set_x(cam_x);

        // Camera parallax offset in asteroid coordinate space
        let parallax_cam_x = self.camera.x() * VIRTUAL_SCALE * 0.25;

        // Dynamic wrap window: always anchored relative to the camera viewport so asteroids
        // continuously span the skyline from edge to edge without ever leaving empty sky.
        let wrap_bound = parallax_cam_x - 120.0;
        let wrap_max = wrap_bound + Self::ASTEROID_WRAP_SPAN;

        for a in &mut self.asteroids {
            a.base_x -= Self::ASTEROID_DRIFT_SPEED * ctx.dt;
            while a.base_x < wrap_bound {
                a.base_x += Self::ASTEROID_WRAP_SPAN;
            }
            while a.base_x > wrap_max {
                a.base_x -= Self::ASTEROID_WRAP_SPAN;
            }
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

            canvas.draw_rectangle(
                current_x as i32,
                current_y as i32,
                a.width,
                a.height,
                a.color,
            );
        }

        // 3. Draw foreground (hills / skyline) over the asteroids
        // The texture is 640x180. The camera samples a 320x180 viewport window from the texture,
        // and projects it to the 1280x720 canvas (exact 4x integer scaling, no squeezing!).
        if let Some(tex) = resources.get_texture(&SpriteID::ActOneForeground) {
            let source = self.camera.source_rect_clamped(tex.width() as f32, tex.height() as f32);
            let dest = self.camera.canvas_dest_rect();
            canvas.draw_texture_pro(tex, source, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_one_asteroids_initialization() {
        let scene = ActOneScene::new();
        assert!(!scene.asteroids.is_empty(), "Asteroid string should not be empty");
        assert!(scene.asteroids.len() >= 65, "Expected string of pearls to have at least 65 asteroids/specks");

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

        assert!(has_left_coverage, "Asteroids must cover beyond the left edge at max camera pan");
        assert!(has_right_coverage, "Asteroids must cover beyond the right edge at max camera pan");
    }

    #[test]
    fn test_act_one_story_camera_sequence() {
        let mut scene = ActOneScene::new();
        assert_eq!(scene.phase(), ActOnePhase::InitialHold);
        assert!(!scene.is_pan_finished());

        // During initial hold (e.g. at 3.0s)
        scene.set_time(3.0);
        let cam_x = if scene.time() <= ActOneScene::HOLD_DURATION {
            0.0
        } else {
            let elapsed_pan = scene.time() - ActOneScene::HOLD_DURATION;
            (elapsed_pan * ActOneScene::PAN_SPEED).min(ActOneScene::MAX_PAN)
        };
        scene.camera_mut().set_x(cam_x);
        assert_eq!(scene.camera().x(), 0.0);
        assert_eq!(scene.phase(), ActOnePhase::InitialHold);

        // Mid-pan (e.g. at 10.0s -> 4s into pan -> 4 * 25 = 100px)
        scene.set_time(10.0);
        let elapsed_pan = scene.time() - ActOneScene::HOLD_DURATION;
        let cam_x = (elapsed_pan * ActOneScene::PAN_SPEED).min(ActOneScene::MAX_PAN);
        scene.camera_mut().set_x(cam_x);
        assert_eq!(scene.camera().x(), 100.0);
        assert_eq!(scene.phase(), ActOnePhase::Panning);
        assert!(!scene.is_pan_finished());

        // Reached end of background (e.g. at 25.0s -> capped at 320.0px)
        scene.set_time(25.0);
        let elapsed_pan = scene.time() - ActOneScene::HOLD_DURATION;
        let cam_x = (elapsed_pan * ActOneScene::PAN_SPEED).min(ActOneScene::MAX_PAN);
        scene.camera_mut().set_x(cam_x);
        assert_eq!(scene.camera().x(), 320.0);
        assert_eq!(scene.phase(), ActOnePhase::Stationary);
        assert!(scene.is_pan_finished());
    }
}
