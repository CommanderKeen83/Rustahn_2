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
}

impl Default for GameCamera {
    fn default() -> Self {
        Self::new()
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
}
