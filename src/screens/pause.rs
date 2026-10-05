use crate::prelude::*;
use sola_raylib::prelude::*;

pub struct Pause {}

impl Screen for Pause {
    fn draw(&mut self, _canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager) {}

    fn is_transparent(&self) -> bool {
        true
    }
}
