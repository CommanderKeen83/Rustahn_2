use sola_raylib::prelude::*;
use crate::prelude::*;


pub enum State{
    Menu(MenuScreen),
    GameIntro(GameIntroScreen),
    Game(GameScreen),
    Pause(PauseScreen),
}
pub struct StateManager{

}
impl StateManager{
    pub fn new() -> Self{
        Self{}
    }


    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>) {
        // test: just draw a red triangle to prove the pipeline works
        canvas.draw_triangle(
            Vector2::new(640.0, 200.0),
            Vector2::new(400.0, 500.0),
            Vector2::new(880.0, 500.0),
            Color::RED,
        );
    }
}