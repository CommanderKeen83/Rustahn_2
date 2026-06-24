use sola_raylib::prelude::*;
use crate::prelude::*;

pub enum StateTag{
    Menu,
    GameIntro,
    Game,
    Pause,
    Options,
}

pub enum State{
    Menu(screens::Menu),
    GameIntro(screens::GameIntro),
    Game(Box<screens::Game>), // because it could be massive
    Pause(screens::Pause),
    Options(screens::Options),
}

impl State{
    // builder functions
    fn menu() -> Self{ Self::Menu(screens::Menu{} )}
    fn gameintro() -> Self{ Self::GameIntro(screens::GameIntro{} )}
    fn game() -> Self{ Self::Game(Box::new(screens::Game{} ))}
    fn pause() -> Self{ Self::Pause(screens::Pause{} )}
    fn options() -> Self{ Self::Options(screens::Options{} )}

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

    pub fn fromTag(tag: StateTag)->Self{
        match tag{
            StateTag::Menu => Self::menu(),
            StateTag::GameIntro => Self::gameintro(),
            StateTag::Game => Self::game(),
            StateTag::Pause => Self::pause(),
            StateTag::Options => Self::options(),
        }
    }

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
    stack: Vec<State>,
}
impl StateManager{
    pub fn new(tag: StateTag) -> Self {
        let mut state = State::fromTag(tag);
        state.on_enter();
        state.on_activate();
        Self{
            stack: vec![state],
        }
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