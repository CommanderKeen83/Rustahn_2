use crate::window::{GAME_HEIGHT, GAME_WIDTH, VIRTUAL_HEIGHT, VIRTUAL_SCALE, VIRTUAL_WIDTH};
use sola_raylib::prelude::*;

/// A 2D game camera representing a virtual viewport into the game world.
///
/// Named `GameCamera` to prevent collisions with `sola_raylib`'s 3D `Camera` type alias.
/// By default, the camera viewport is configured to the virtual retro resolution
/// (320x180 px), which scales 4x to the 1280x720 canvas (`VIRTUAL_SCALE = 4.0`).
#[derive(Debug, Clone, Copy)]
pub struct GameCamera {
    pub position: Vector2,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl GameCamera {
    /// Creates a new camera centered at (0, 0) with default virtual viewport dimensions (320x180).
    pub fn new() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            viewport_width: VIRTUAL_WIDTH as f32,
            viewport_height: VIRTUAL_HEIGHT as f32,
        }
    }

    /// Creates a camera positioned at (x, y) with default virtual viewport dimensions.
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            position: Vector2::new(x, y),
            viewport_width: VIRTUAL_WIDTH as f32,
            viewport_height: VIRTUAL_HEIGHT as f32,
        }
    }

    /// Horizontal camera position in virtual pixels.
    pub fn x(&self) -> f32 {
        self.position.x
    }

    /// Vertical camera position in virtual pixels.
    pub fn y(&self) -> f32 {
        self.position.y
    }

    /// Set horizontal camera position.
    pub fn set_x(&mut self, x: f32) {
        self.position.x = x;
    }

    /// Set vertical camera position.
    pub fn set_y(&mut self, y: f32) {
        self.position.y = y;
    }

    /// Set camera position.
    pub fn set_position(&mut self, pos: Vector2) {
        self.position = pos;
    }

    /// Move camera position by a given delta in virtual pixels.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.position.x += dx;
        self.position.y += dy;
    }

    /// Clamp camera x coordinate between `min` and `max`.
    pub fn clamp_x(&mut self, min: f32, max: f32) {
        self.position.x = self.position.x.clamp(min, max);
    }

    /// Maximum camera horizontal pan offset for a given world / texture width.
    pub fn max_pan_x(&self, total_width: f32) -> f32 {
        (total_width - self.viewport_width).max(0.0)
    }

    /// Generates a source rectangle of viewport size (320x180) from a texture at the camera's
    /// current position, clamped so it never samples outside `[0.0, texture_width]`.
    pub fn source_rect_clamped(&self, texture_width: f32, texture_height: f32) -> Rectangle {
        let max_x = (texture_width - self.viewport_width).max(0.0);
        let max_y = (texture_height - self.viewport_height).max(0.0);
        let x = self.position.x.clamp(0.0, max_x);
        let y = self.position.y.clamp(0.0, max_y);
        Rectangle::new(x, y, self.viewport_width, self.viewport_height)
    }

    /// The destination rectangle on the render canvas (1280x720) for full-screen projection.
    pub fn canvas_dest_rect(&self) -> Rectangle {
        Rectangle::new(0.0, 0.0, GAME_WIDTH as f32, GAME_HEIGHT as f32)
    }

    /// Computes the canvas screen X coordinate for a world element with optional parallax factor.
    ///
    /// * `world_x`: position in world canvas units
    /// * `parallax`: parallax factor (0.0 = static backdrop, 1.0 = moves with foreground)
    pub fn to_screen_x(&self, world_x: f32, parallax: f32) -> f32 {
        let cam_offset = self.position.x * VIRTUAL_SCALE * parallax;
        world_x - cam_offset
    }

    /// Converts this camera to sola_raylib's [`Camera2D`].
    pub fn to_raylib_camera2d(&self) -> Camera2D {
        Camera2D {
            offset: Vector2::new(0.0, 0.0),
            target: self.position,
            rotation: 0.0,
            zoom: VIRTUAL_SCALE,
        }
    }

    /// Converts a screen pixel coordinate (e.g. from mouse input) into virtual world coordinates,
    /// accounting for window letterboxing/scaling and camera viewport offset.
    pub fn screen_to_virtual_world(&self, screen_pos: Vector2, window_width: f32, window_height: f32) -> Vector2 {
        let game_w = GAME_WIDTH as f32;
        let game_h = GAME_HEIGHT as f32;
        let scale = (window_width / game_w).min(window_height / game_h).max(0.001);
        let offset_x = (window_width - (game_w * scale)) * 0.5;
        let offset_y = (window_height - (game_h * scale)) * 0.5;

        let canvas_x = (screen_pos.x - offset_x) / scale;
        let canvas_y = (screen_pos.y - offset_y) / scale;

        let world_x = self.x() + canvas_x / VIRTUAL_SCALE;
        let world_y = self.y() + canvas_y / VIRTUAL_SCALE;

        Vector2::new(world_x, world_y)
    }

    /// Computes the target camera top-left position needed to center an entity at `center`.
    pub fn target_for_center(&self, center: Vector2) -> Vector2 {
        Vector2::new(
            center.x - self.viewport_width * 0.5,
            center.y - self.viewport_height * 0.5,
        )
    }

    /// Smoothly moves the camera position towards `target` using frame-rate independent
    /// exponential damping (rubber-band easing).
    ///
    /// `speed` controls the tracking stiffness/responsiveness (e.g. 5.0 for a slight rubber-band lag).
    /// `dt` is the elapsed frame time in seconds.
    pub fn follow_smooth(&mut self, target: Vector2, speed: f32, dt: f32) {
        if dt <= 0.0 || speed <= 0.0 {
            return;
        }
        let factor = 1.0 - (-speed * dt).exp();
        self.position.x += (target.x - self.position.x) * factor;
        self.position.y += (target.y - self.position.y) * factor;

        if (target.x - self.position.x).abs() < 1e-4 {
            self.position.x = target.x;
        }
        if (target.y - self.position.y).abs() < 1e-4 {
            self.position.y = target.y;
        }
    }

    /// Clamps the horizontal camera position so that `self.position.x >= min_x`.
    pub fn clamp_min_x(&mut self, min_x: f32) {
        self.position.x = self.position.x.max(min_x);
    }
}

impl Default for GameCamera {
    fn default() -> Self {
        Self::new()
    }
}

/// Phase of an introductory panning sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntroPhase {
    /// Initial hold steady at the beginning of the viewport path.
    InitialHold,
    /// Smooth panning across the virtual panorama.
    Panning,
    /// Reached target boundary and stays steady.
    Stationary,
}

/// Backwards-compatible alias for the phase in Act One.
pub type ActOnePhase = IntroPhase;

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

    pub fn phase(&self, t: f32) -> IntroPhase {
        if t < self.hold {
            IntroPhase::InitialHold
        } else if self.camera_x(t) < self.max_x {
            IntroPhase::Panning
        } else {
            IntroPhase::Stationary
        }
    }

    pub fn finish_time(&self) -> f32 {
        self.hold + self.max_x / self.speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_initialization() {
        let cam = GameCamera::new();
        assert_eq!(cam.x(), 0.0);
        assert_eq!(cam.y(), 0.0);
        assert_eq!(cam.viewport_width, 320.0);
        assert_eq!(cam.viewport_height, 180.0);
    }

    #[test]
    fn test_camera_panning_and_clamping() {
        let mut cam = GameCamera::at(50.0, 0.0);
        cam.pan(25.0, 10.0);
        assert_eq!(cam.x(), 75.0);
        assert_eq!(cam.y(), 10.0);

        cam.clamp_x(0.0, 60.0);
        assert_eq!(cam.x(), 60.0);
    }

    #[test]
    fn test_camera_source_rect_clamping() {
        let cam = GameCamera::at(400.0, 0.0);
        // For a 640x180 texture, max x pan is 640 - 320 = 320
        let rect = cam.source_rect_clamped(640.0, 180.0);
        assert_eq!(rect.x, 320.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 320.0);
        assert_eq!(rect.height, 180.0);
    }

    #[test]
    fn test_camera_parallax_to_screen_x() {
        let cam = GameCamera::at(100.0, 0.0);
        // At parallax 0.0 (backdrop): screen x is unchanged
        assert_eq!(cam.to_screen_x(500.0, 0.0), 500.0);
        // At parallax 0.25: cam offset is 100 * 4.0 * 0.25 = 100.0
        assert_eq!(cam.to_screen_x(500.0, 0.25), 400.0);
        // At parallax 1.0: cam offset is 100 * 4.0 * 1.0 = 400.0
        assert_eq!(cam.to_screen_x(500.0, 1.0), 100.0);
    }

    #[test]
    fn test_intro_pan_sequence() {
        let pan = IntroPan::new(6.0, 25.0, 320.0);
        assert_eq!(pan.camera_x(3.0), 0.0);
        assert_eq!(pan.phase(3.0), IntroPhase::InitialHold);
        assert_eq!(pan.camera_x(10.0), 100.0);
        assert_eq!(pan.phase(10.0), IntroPhase::Panning);
        assert_eq!(pan.camera_x(25.0), 320.0);
        assert_eq!(pan.phase(25.0), IntroPhase::Stationary);
        assert!((pan.finish_time() - 18.8).abs() < 1e-4);
    }

    #[test]
    fn test_camera_target_for_center_and_follow_smooth() {
        let mut cam = GameCamera::new();
        assert_eq!(cam.x(), 0.0);
        assert_eq!(cam.y(), 0.0);

        let target = cam.target_for_center(Vector2::new(320.0, 180.0));
        assert_eq!(target.x, 160.0);
        assert_eq!(target.y, 90.0);

        // Smooth follow moves towards target with rubber-band easing
        cam.follow_smooth(target, 5.0, 0.1);
        assert!(cam.x() > 0.0 && cam.x() < 160.0);
        assert!(cam.y() > 0.0 && cam.y() < 90.0);

        // Clamp min x
        cam.set_x(-50.0);
        cam.clamp_min_x(0.0);
        assert_eq!(cam.x(), 0.0);
    }

    #[test]
    fn test_camera_screen_to_virtual_world() {
        let cam = GameCamera::at(100.0, 50.0);
        // Window size matches game width/height (1280x720)
        let screen_pos = Vector2::new(640.0, 360.0);
        let world = cam.screen_to_virtual_world(screen_pos, 1280.0, 720.0);
        // Canvas center: 640/4 = 160, 360/4 = 90.
        // World: 100 + 160 = 260, 50 + 90 = 140.
        assert_eq!(world.x, 260.0);
        assert_eq!(world.y, 140.0);
    }
}
