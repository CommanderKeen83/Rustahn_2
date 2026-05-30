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
    // main game loop
    pub fn run(&mut self){
        while !self.window.should_close() {

            self.window.draw();
        }
    }
}