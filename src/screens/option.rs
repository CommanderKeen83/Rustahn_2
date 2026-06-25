use sola_raylib::prelude::*;
pub struct Options{}
impl Options{
    pub fn on_enter(&mut self){}
    pub fn on_activate(&mut self){}
    pub fn on_deactivate(&mut self){}
    pub fn on_exit(&mut self){}

    pub fn update(&mut self, _dt: f32){}
    pub fn draw(&self, _canvas: &mut RaylibTextureMode<RaylibHandle>){}

    pub fn is_transparent(&self) -> bool {
        true
    }

    pub fn is_transcendent(&self) -> bool {
        false
    }

}