use sola_raylib::prelude::*;
use crate::prelude::*;


pub enum State{
    Menu(Menu),
    GameIntro(GameIntro),
    Game(Box<Game>), // because it could be massive
    Pause(Pause),
    Options(Options),
}

impl State{
    // builder functions
    pub fn menu() -> Self{ Self::Menu(Menu{} )}
    pub fn gameintro() -> Self{ Self::GameIntro(GameIntro{} )}
    pub fn game() -> Self{ Self::Game(Box::new(Game{} ))}
    pub fn pause() -> Self{ Self::Pause(Pause{} )}

    // all screens must implement these functions
    pub fn on_enter(&mut self){
        match self{
            State::Menu(screen) => screen.on_enter(),
            State::GameIntro(screen) => screen.on_enter(),
            State::Game(screen) => screen.on_enter(),
            State::Pause(screen) => screen.on_enter(),
            State::Options(screen) => screen.on_enter()
        }
    }
    pub fn on_activate(&mut self){}
    pub fn on_deactivate(&mut self){}
    pub fn on_exit(&mut self){}

    pub fn update(&mut self, dt: f32){}
    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>){}


}

pub enum Transition{
    None, 
    Push(State), // Push a State on Top of the Stack
    Switch(State), // Reuse existing or push fresh
    Pop, // remove state on Top of The Stack
    Swap(State), // pop current, push new
    ClearAndPush(State), // pop all, push new
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

/*




*/