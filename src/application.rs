use crate::window::*;
use crate::gamestate::*;

pub struct Application{
    window: Window,
    gamestate: Gamestate,
}

impl Application{
    pub fn new() -> Self{
        Self{
            window: Window::init(),
            gamestate: Gamestate::new(),
        }
    }
    pub fn update(&mut self, delta_time: f32){

    }
    pub fn draw(&mut self){
        self.window.draw(|canvas| {
            self.gamestate.draw(canvas);
        });

    }
    // main game loop
    pub fn run(&mut self){
        while !self.window.should_close() {
            self.update(self.window.get_delta_time());
            self.draw();
        }
    }
}