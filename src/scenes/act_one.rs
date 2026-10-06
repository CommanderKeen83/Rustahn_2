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

/// Act One / Level 1 scene featuring the calm night skyline and gently undulating asteroids.
pub struct ActOneScene {
    asteroids: Vec<Asteroid>,
    time: f32,
}

impl ActOneScene {
    pub fn new() -> Self {
        Self {
            asteroids: Self::init_asteroids(),
            time: 0.0,
        }
    }

    pub fn asteroids(&self) -> &[Asteroid] {
        &self.asteroids
    }

    fn init_asteroids() -> Vec<Asteroid> {
        let mut list = Vec::with_capacity(120);

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

        const COUNT: usize = 65;
        const SPACING: f32 = 22.0;
        const START_X: f32 = -60.0;

        for i in 0..COUNT {
            let x = START_X + i as f32 * SPACING;
            // Baseline follows a gentle arch across the upper starry sky (y: ~180 to ~230)
            let arch = (i as f32 * std::f32::consts::TAU / COUNT as f32).sin() * 25.0;
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

        // Gently drift the string of pearls horizontally across the night sky
        const DRIFT_SPEED: f32 = 25.0;
        const WRAP_BOUND: f32 = -70.0;
        const WRAP_SPAN: f32 = 65.0 * 22.0; // 1430.0

        for a in &mut self.asteroids {
            a.base_x -= DRIFT_SPEED * ctx.dt;
            if a.base_x < WRAP_BOUND {
                a.base_x += WRAP_SPAN;
            }
        }

        None
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
        // 1. Draw the dark night sky background
        if let Some(tex) = resources.get_texture(&SpriteID::LevelOneBackground) {
            let source = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);
            let dest = Rectangle::new(0.0, 0.0, GAME_WIDTH as f32, GAME_HEIGHT as f32);
            canvas.draw_texture_pro(tex, source, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
        }

        // 2. Render gently moving string of pearls asteroids
        // Wave number matches wrap span so sinusoidal wrapping is seamless: k = 2pi / 1430.0 ≈ 0.004394
        const K: f32 = std::f32::consts::TAU / (65.0 * 22.0);
        const OMEGA: f32 = 1.4;

        for a in &self.asteroids {
            let current_x = a.base_x.round();
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
        if let Some(tex) = resources.get_texture(&SpriteID::ActOneForeground) {
            let source = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);
            let dest = Rectangle::new(0.0, 0.0, GAME_WIDTH as f32, GAME_HEIGHT as f32);
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
}
