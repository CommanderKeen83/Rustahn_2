use crate::ecs;
use sola_raylib::prelude::*;

pub struct Gamestate {
    #[allow(dead_code)]
    world: ecs::World,
}
impl Gamestate {
    pub fn new() -> Self {
        Self {
            world: ecs::World::new(),
        }
    }

    pub fn update(&mut self, _dt: f32) {}
    pub fn draw(&self, _canvas: &mut RaylibTextureMode<RaylibHandle>) {
        // test: just draw a red triangle to prove the pipeline works
    }
}
