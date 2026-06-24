use crate::window::*;
use crate::gamestate::*;
use crate::statemanager::*;

pub struct Application{
    window: Window,
    gamestate: Gamestate,
    statemanager: StateManager,
}

impl Application{
    pub fn new() -> Self{
        Self{
            window: Window::init(),
            gamestate: Gamestate::new(),
            statemanager: StateManager::new(StateTag::Menu),
        }
    }
    pub fn update(&mut self, delta_time: f32){
        self.statemanager.update(delta_time);
        self.statemanager.late_update();
    }
    pub fn draw(&mut self){
            self.window.draw(|canvas|{
                self.statemanager.draw(canvas);
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