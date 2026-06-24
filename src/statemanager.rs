use sola_raylib::prelude::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn on_activate(&mut self){
        match self{
            State::Menu(screen) => screen.on_activate(),
            State::GameIntro(screen) => screen.on_activate(),
            State::Game(screen) => screen.on_activate(),
            State::Pause(screen) => screen.on_activate(),
            State::Options(screen) => screen.on_activate()
        }
    }
    pub fn on_deactivate(&mut self){
        match self{
            State::Menu(screen) => screen.on_deactivate(),
            State::GameIntro(screen) => screen.on_deactivate(),
            State::Game(screen) => screen.on_deactivate(),
            State::Pause(screen) => screen.on_deactivate(),
            State::Options(screen) => screen.on_deactivate()
        }
    }
    pub fn on_exit(&mut self){
        match self{
            State::Menu(screen) => screen.on_exit(),
            State::GameIntro(screen) => screen.on_exit(),
            State::Game(screen) => screen.on_exit(),
            State::Pause(screen) => screen.on_exit(),
            State::Options(screen) => screen.on_exit()
        }
    }

    pub fn fromTag(tag: StateTag)->Self{
        match tag{
            StateTag::Menu => Self::menu(),
            StateTag::GameIntro => Self::gameintro(),
            StateTag::Game => Self::game(),
            StateTag::Pause => Self::pause(),
            StateTag::Options => Self::options(),
        }
    }

    pub fn tag(&self) -> StateTag{
        match self{
            State::Menu(_) => StateTag::Menu,
            State::GameIntro(_) => StateTag::GameIntro,
            State::Game(_) => StateTag::Game,
            State::Pause(_) => StateTag::Pause,
            State::Options(_) => StateTag::Options,
        }
    }

    pub fn update(&mut self, dt: f32){
        match self{
            State::Menu(screen) => screen.update(dt),
            State::GameIntro(screen) => screen.update(dt),
            State::Game(screen) => screen.update(dt),
            State::Pause(screen) => screen.update(dt),
            State::Options(screen) => screen.update(dt)
        }
    }
    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>){
        match self{
            State::Menu(screen) => screen.draw(canvas),
            State::GameIntro(screen) => screen.draw(canvas),
            State::Game(screen) => screen.draw(canvas),
            State::Pause(screen) => screen.draw(canvas),
            State::Options(screen) => screen.draw(canvas)
        }
    }


}

pub enum Transition{
    None, 
    Push(State), // Push a State on Top of the Stack
    Switch(StateTag), // Reuse existing or push fresh
    Pop, // remove state on Top of The Stack
    Swap(State), // pop current, push new
    ClearAndPush(State), // pop all, push new
}
pub struct StateManager{
    stack: Vec<State>,
    transitions: Vec<Transition>,
}
impl StateManager{
    pub fn new(tag: StateTag) -> Self {
        let mut state = State::fromTag(tag);
        state.on_enter();
        state.on_activate();
        Self{
            stack: vec![state],
            transitions: vec![],
        }
    }

    pub fn update(&mut self, dt: f32) {
        for state in &mut self.stack {
            state.update(dt);
        }
    }

    pub fn add_transition(&mut self, transition: Transition) {
        match transition {
            Transition::None => {}
            _ => self.transitions.push(transition),
        }
    }

    pub fn push(&mut self, state: State) {
        if let Some(top) = self.stack.last_mut() {
            top.on_deactivate();
        }
        let mut state = state;
        state.on_enter();
        state.on_activate();
        self.stack.push(state);
    }

    pub fn pop(&mut self) {
        if let Some(mut top) = self.stack.pop() {
            top.on_deactivate();
            top.on_exit();
        }
        if let Some(new_top) = self.stack.last_mut() {
            new_top.on_activate();
        }
    }

    pub fn swap(&mut self, state: State) {
        if let Some(mut top) = self.stack.pop() {
            top.on_deactivate();
            top.on_exit();
        }
        let mut state = state;
        state.on_enter();
        state.on_activate();
        self.stack.push(state);
    }

    pub fn clear_and_push(&mut self, state: State) {
        while let Some(mut top) = self.stack.pop() {
            top.on_deactivate();
            top.on_exit();
        }
        let mut state = state;
        state.on_enter();
        state.on_activate();
        self.stack.push(state);
    }

    pub fn switch(&mut self, tag: StateTag) {
        let position = self.stack.iter().position(|s| s.tag() == tag);
        if let Some(pos) = position {
            if pos < self.stack.len() - 1 {
                if let Some(top) = self.stack.last_mut() {
                    top.on_deactivate();
                }
                let mut existing_state = self.stack.remove(pos);
                existing_state.on_activate();
                self.stack.push(existing_state);
            }
        } else {
            if let Some(top) = self.stack.last_mut() {
                top.on_deactivate();
            }
            let mut state = State::fromTag(tag);
            state.on_enter();
            state.on_activate();
            self.stack.push(state);
        }
    }

    pub fn late_update(&mut self) {
        let pending: Vec<Transition> = self.transitions.drain(..).collect();
        for transition in pending {
            match transition {
                Transition::None => {}
                Transition::Push(state) => self.push(state),
                Transition::Switch(tag) => self.switch(tag),
                Transition::Pop => self.pop(),
                Transition::Swap(state) => self.swap(state),
                Transition::ClearAndPush(state) => self.clear_and_push(state),
            }
        }
    }


    pub fn draw(&self, canvas: &mut RaylibTextureMode<RaylibHandle>) {
        for state in &self.stack{
            state.draw(canvas);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_new_state() {
        let mut manager = StateManager::new(StateTag::Menu);
        assert_eq!(manager.stack.len(), 1);
        assert_eq!(manager.stack[0].tag(), StateTag::Menu);

        // Switch to a new state: Game
        manager.switch(StateTag::Game);
        assert_eq!(manager.stack.len(), 2);
        assert_eq!(manager.stack[0].tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].tag(), StateTag::Game);
    }

    #[test]
    fn test_switch_existing_state_middle() {
        let mut manager = StateManager::new(StateTag::Menu);
        manager.switch(StateTag::Game);
        manager.switch(StateTag::Pause);
        
        // Stack should be [Menu, Game, Pause]
        assert_eq!(manager.stack.len(), 3);
        assert_eq!(manager.stack[0].tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].tag(), StateTag::Game);
        assert_eq!(manager.stack[2].tag(), StateTag::Pause);

        // Switch to Game (which already exists at index 1)
        manager.switch(StateTag::Game);
        
        // Stack should now be [Menu, Pause, Game]
        assert_eq!(manager.stack.len(), 3);
        assert_eq!(manager.stack[0].tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].tag(), StateTag::Pause);
        assert_eq!(manager.stack[2].tag(), StateTag::Game);
    }

    #[test]
    fn test_switch_existing_state_top() {
        let mut manager = StateManager::new(StateTag::Menu);
        manager.switch(StateTag::Game);
        
        // Stack should be [Menu, Game]
        assert_eq!(manager.stack.len(), 2);
        assert_eq!(manager.stack[0].tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].tag(), StateTag::Game);

        // Switch to Game (which is already top)
        manager.switch(StateTag::Game);
        
        // Stack should remain [Menu, Game]
        assert_eq!(manager.stack.len(), 2);
        assert_eq!(manager.stack[0].tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].tag(), StateTag::Game);
    }
}