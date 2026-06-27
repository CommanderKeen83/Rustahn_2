use sola_raylib::prelude::*;
use crate::prelude::ResourceManager;
pub struct GameIntro{}
impl GameIntro{
    pub fn on_enter(&mut self){}
    pub fn on_activate(&mut self){}
    pub fn on_deactivate(&mut self){}
    pub fn on_exit(&mut self){}

    pub fn update(&mut self, _rl: &RaylibHandle, _dt: f32) -> Option<crate::prelude::Transition> {
        None
    }
    pub fn draw(&self, _canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager){}

    pub fn is_transparent(&self) -> bool {
        false
    }

    pub fn is_transcendent(&self) -> bool {
        false
    }

}