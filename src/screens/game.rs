use crate::prelude::*;
use sola_raylib::prelude::*;
pub struct Game {}
impl Game {
    pub fn on_enter(
        &mut self,
        _rl: &mut RaylibHandle,
        _thread: &RaylibThread,
        _resources: &mut ResourceManager,
    ) {
    }
    pub fn on_activate(
        &mut self,
        _rl: &mut RaylibHandle,
        _thread: &RaylibThread,
        _resources: &mut ResourceManager,
    ) {
    }
    pub fn on_deactivate(&mut self) {}
    pub fn on_exit(&mut self) {}

    pub fn update(
        &mut self,
        _rl: &RaylibHandle,
        _dt: f32,
        _audio: &mut AudioManager<'_, '_>,
    ) -> Option<crate::prelude::Transition> {
        None
    }
    pub fn draw(
        &mut self,
        _canvas: &mut RaylibTextureMode<RaylibHandle>,
        _resources: &ResourceManager,
    ) {
    }

    pub fn is_transparent(&self) -> bool {
        false
    }

    pub fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        None
    }

    pub fn is_transcendent(&self) -> bool {
        false
    }
}
