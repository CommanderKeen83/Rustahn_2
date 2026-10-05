use crate::prelude::*;
use sola_raylib::prelude::*;

pub struct Options {}

impl Options {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for Options {
    fn draw(&mut self, _canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager) {}

    fn is_transparent(&self) -> bool {
        true
    }
}
