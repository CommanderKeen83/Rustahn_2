use sola_raylib::prelude::*;
pub struct Menu{}
impl Menu{
    pub fn on_enter(&mut self){}
    pub fn on_activate(&mut self){}
    pub fn on_deactivate(&mut self){}
    pub fn on_exit(&mut self){}

    pub fn update(&mut self, dt: f32){}
    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>){}

}