use sola_raylib::prelude::*;
use crate::prelude::ResourceManager;
pub struct Options{}
impl Options{
    pub fn on_enter(&mut self, _rl: Option<&mut RaylibHandle>, _thread: Option<&RaylibThread>, _resources: &mut ResourceManager){}
    pub fn on_activate(&mut self, _rl: Option<&mut RaylibHandle>, _thread: Option<&RaylibThread>, _resources: &mut ResourceManager){}
    pub fn on_deactivate(&mut self){}
    pub fn on_exit(&mut self){}

    pub fn update(&mut self, _rl: &RaylibHandle, _dt: f32) -> Option<crate::prelude::Transition> {
        None
    }
    pub fn draw(&mut self, _canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager){}

    pub fn is_transparent(&self) -> bool {
        true
    }

    pub fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        None
    }

    pub fn is_transcendent(&self) -> bool {
        false
    }

}