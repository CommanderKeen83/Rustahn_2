use crate::camera::GameCamera;
use sola_raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct Asteroid {
    pub base_x: f32,
    pub base_y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: i32,
    pub height: i32,
    pub color: Color,
}

/// Particle field of drifting asteroids traveling along a low-trajectory diagonal path.
#[derive(Debug, Clone)]
pub struct AsteroidField {
    asteroids: Vec<Asteroid>,
}

impl AsteroidField {
    pub const COUNT: usize = 225;
    pub const WRAP_SPAN: f32 = 3300.0;
    pub const START_X: f32 = -150.0;
    pub const SKY_TOP: f32 = -30.0;
    pub const SKY_BOTTOM: f32 = 330.0;
    pub const SKY_SPAN: f32 = Self::SKY_BOTTOM - Self::SKY_TOP;

    /// Fast, deterministic 32-bit integer pseudo-random number generator in [0.0, 1.0).
    fn pseudo_random(i: usize, seed: u32) -> f32 {
        let mut n = (i as u32).wrapping_mul(0x85ebca6b).wrapping_add(seed);
        n ^= n >> 13;
        n = n.wrapping_mul(0xc2b2ae35);
        n ^= n >> 16;
        (n as f32) / (u32::MAX as f32)
    }

    /// Creates and initializes a new pseudo-randomly scattered asteroid field.
    pub fn new() -> Self {
        let mut asteroids = Vec::with_capacity(Self::COUNT);

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

        let sizes = [(2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (4, 6), (6, 4), (4, 4), (2, 2)];

        for i in 0..Self::COUNT {
            let frac = i as f32 / Self::COUNT as f32;
            let jitter = (Self::pseudo_random(i, 1) - 0.5)
                * (Self::WRAP_SPAN / Self::COUNT as f32 * 1.6);
            let base_x = Self::START_X + frac * Self::WRAP_SPAN + jitter;

            // Scatter vertically across the open starry night sky
            let base_y = Self::SKY_TOP + Self::pseudo_random(i, 2) * Self::SKY_SPAN;

            // Low trajectory diagonal path from top-right to bottom-left:
            // speed_x moves leftward (30..42 px/s), speed_y moves downward with a shallow slope (~12..17 degrees)
            let speed_x = 30.0 + Self::pseudo_random(i, 3) * 12.0;
            let slope = 0.22 + Self::pseudo_random(i, 4) * 0.08;
            let speed_y = speed_x * slope;

            let size_idx = (Self::pseudo_random(i, 5) * sizes.len() as f32) as usize % sizes.len();
            let (w, h) = sizes[size_idx];

            let color_idx = (Self::pseudo_random(i, 6) * colors.len() as f32) as usize % colors.len();
            let color = colors[color_idx];

            asteroids.push(Asteroid {
                base_x,
                base_y,
                speed_x,
                speed_y,
                width: w,
                height: h,
                color,
            });
        }

        Self { asteroids }
    }

    /// Read-only slice of all asteroids in the field.
    pub fn asteroids(&self) -> &[Asteroid] {
        &self.asteroids
    }

    /// Mutable slice of all asteroids in the field.
    pub fn asteroids_mut(&mut self) -> &mut [Asteroid] {
        &mut self.asteroids
    }

    /// Updates asteroid positions along their low diagonal trajectory and wraps around camera boundaries.
    pub fn update(&mut self, dt: f32, parallax_cam_x: f32) {
        let wrap_bound = parallax_cam_x - 150.0;
        let wrap_max = wrap_bound + Self::WRAP_SPAN;

        for a in &mut self.asteroids {
            a.base_x -= a.speed_x * dt;
            a.base_y += a.speed_y * dt;

            while a.base_x < wrap_bound {
                a.base_x += Self::WRAP_SPAN;
            }
            while a.base_x > wrap_max {
                a.base_x -= Self::WRAP_SPAN;
            }

            while a.base_y > Self::SKY_BOTTOM {
                a.base_y -= Self::SKY_SPAN;
            }
            while a.base_y < Self::SKY_TOP {
                a.base_y += Self::SKY_SPAN;
            }
        }
    }

    /// Renders all asteroids to the canvas using camera parallax projection.
    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>, camera: &GameCamera) {
        for a in &self.asteroids {
            let current_x = camera.to_screen_x(a.base_x, 0.25).round();
            let current_y = a.base_y.round();

            canvas.draw_rectangle(current_x as i32, current_y as i32, a.width, a.height, a.color);
        }
    }
}

impl Default for AsteroidField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asteroid_field_initialization() {
        let field = AsteroidField::new();
        assert_eq!(field.asteroids().len(), AsteroidField::COUNT);

        for a in field.asteroids() {
            assert!(a.width >= 1 && a.width <= 6);
            assert!(a.height >= 1 && a.height <= 6);
            assert!(a.base_y >= AsteroidField::SKY_TOP && a.base_y <= AsteroidField::SKY_BOTTOM);
            assert!(a.speed_x > 0.0);
            assert!(a.speed_y > 0.0);
            let slope = a.speed_y / a.speed_x;
            assert!(slope >= 0.15 && slope <= 0.35);
        }
    }

    #[test]
    fn test_asteroid_field_update_motion() {
        let mut field = AsteroidField::new();
        let initial_positions: Vec<(f32, f32)> = field
            .asteroids()
            .iter()
            .map(|a| (a.base_x, a.base_y))
            .collect();

        field.update(1.0, 0.0);

        let mut moved_count = 0;
        for (i, a) in field.asteroids().iter().enumerate() {
            let (init_x, init_y) = initial_positions[i];
            if a.base_x < init_x && a.base_y > init_y {
                let dx = init_x - a.base_x;
                let dy = a.base_y - init_y;
                assert!(dx > 0.0);
                assert!(dy > 0.0);
                let slope = dy / dx;
                assert!(slope >= 0.15 && slope <= 0.35);
                moved_count += 1;
            }
        }
        assert!(moved_count > 200);
    }
}
