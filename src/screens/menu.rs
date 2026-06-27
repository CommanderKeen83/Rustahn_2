use sola_raylib::prelude::*;
use crate::prelude::*;

pub struct Menu{}
impl Menu{
    pub fn on_enter(&mut self){}
    pub fn on_activate(&mut self){}
    pub fn on_deactivate(&mut self){}
    pub fn on_exit(&mut self){}

    pub fn update(&mut self, _dt: f32){}
    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager){
        if let Some(tex) = resources.get_texture(&SpriteID::MenuBackground) {
            let source = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);
            let dest = Rectangle::new(0.0, 0.0, GAME_WIDTH as f32, GAME_HEIGHT as f32);
            canvas.draw_texture_pro(
                tex,
                source,
                dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
        }
    }

    pub fn is_transparent(&self) -> bool {
        false
    }

    pub fn is_transcendent(&self) -> bool {
        false
    }

}