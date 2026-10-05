use crate::prelude::*;
use sola_raylib::prelude::*;

pub struct Game {}

impl Game {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for Game {
    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager) {
        canvas.draw_triangle(
            Vector2::new(200.0, 100.0),
            Vector2::new(100.0, 100.0),
            Vector2::new(150.0, 200.0),
            Color::RED,
        );
    }
}
