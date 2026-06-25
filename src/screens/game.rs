use sola_raylib::prelude::*;
pub struct Game{}
impl Game{
    pub fn on_enter(&mut self){}
    pub fn on_activate(&mut self){}
    pub fn on_deactivate(&mut self){}
    pub fn on_exit(&mut self){}

    pub fn update(&mut self, dt: f32){}
    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>){}

    pub fn is_transparent(&self) -> bool {
        false
    }

    pub fn is_transcendent(&self) -> bool {
        false
    }

}