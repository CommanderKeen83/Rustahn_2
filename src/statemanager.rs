use crate::prelude::*;
use sola_raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTag {
    Menu,
    GameIntro,
    Game,
    Pause,
    Options,
}

pub enum State {
    Menu(screens::Menu),
    GameIntro(screens::GameIntro),
    Game(Box<screens::Game>), // because it could be massive
    Pause(screens::Pause),
    Options(screens::Options),
}

impl State {
    // builder functions
    fn menu() -> Self {
        Self::Menu(screens::Menu::new())
    }
    fn gameintro() -> Self {
        Self::GameIntro(screens::GameIntro::new())
    }
    fn game() -> Self {
        Self::Game(Box::new(screens::Game {}))
    }
    fn pause() -> Self {
        Self::Pause(screens::Pause {})
    }
    fn options() -> Self {
        Self::Options(screens::Options {})
    }

    // all screens must implement these functions
    pub fn on_enter(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        resources: &mut ResourceManager,
    ) {
        match self {
            State::Menu(screen) => screen.on_enter(rl, thread, resources),
            State::GameIntro(screen) => screen.on_enter(rl, thread, resources),
            State::Game(screen) => screen.on_enter(rl, thread, resources),
            State::Pause(screen) => screen.on_enter(rl, thread, resources),
            State::Options(screen) => screen.on_enter(rl, thread, resources),
        }
    }
    pub fn on_activate(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        resources: &mut ResourceManager,
    ) {
        match self {
            State::Menu(screen) => screen.on_activate(rl, thread, resources),
            State::GameIntro(screen) => screen.on_activate(rl, thread, resources),
            State::Game(screen) => screen.on_activate(rl, thread, resources),
            State::Pause(screen) => screen.on_activate(rl, thread, resources),
            State::Options(screen) => screen.on_activate(rl, thread, resources),
        }
    }
    pub fn on_deactivate(&mut self) {
        match self {
            State::Menu(screen) => screen.on_deactivate(),
            State::GameIntro(screen) => screen.on_deactivate(),
            State::Game(screen) => screen.on_deactivate(),
            State::Pause(screen) => screen.on_deactivate(),
            State::Options(screen) => screen.on_deactivate(),
        }
    }
    pub fn on_exit(&mut self) {
        match self {
            State::Menu(screen) => screen.on_exit(),
            State::GameIntro(screen) => screen.on_exit(),
            State::Game(screen) => screen.on_exit(),
            State::Pause(screen) => screen.on_exit(),
            State::Options(screen) => screen.on_exit(),
        }
    }

    pub fn from_tag(tag: StateTag) -> Self {
        match tag {
            StateTag::Menu => Self::menu(),
            StateTag::GameIntro => Self::gameintro(),
            StateTag::Game => Self::game(),
            StateTag::Pause => Self::pause(),
            StateTag::Options => Self::options(),
        }
    }

    pub fn tag(&self) -> StateTag {
        match self {
            State::Menu(_) => StateTag::Menu,
            State::GameIntro(_) => StateTag::GameIntro,
            State::Game(_) => StateTag::Game,
            State::Pause(_) => StateTag::Pause,
            State::Options(_) => StateTag::Options,
        }
    }

    pub fn update(
        &mut self,
        rl: &RaylibHandle,
        dt: f32,
        has_save: bool,
        audio: &mut AudioManager<'_, '_>,
    ) -> Option<Transition> {
        match self {
            State::Menu(screen) => screen.update(rl, dt, has_save, audio),
            State::GameIntro(screen) => screen.update(rl, dt, audio),
            State::Game(screen) => screen.update(rl, dt, audio),
            State::Pause(screen) => screen.update(rl, dt, audio),
            State::Options(screen) => screen.update(rl, dt, audio),
        }
    }
    pub fn draw(
        &mut self,
        canvas: &mut RaylibTextureMode<RaylibHandle>,
        resources: &ResourceManager,
    ) {
        match self {
            State::Menu(screen) => screen.draw(canvas, resources),
            State::GameIntro(screen) => screen.draw(canvas, resources),
            State::Game(screen) => screen.draw(canvas, resources),
            State::Pause(screen) => screen.draw(canvas, resources),
            State::Options(screen) => screen.draw(canvas, resources),
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            State::Menu(screen) => screen.is_transparent(),
            State::GameIntro(screen) => screen.is_transparent(),
            State::Game(screen) => screen.is_transparent(),
            State::Pause(screen) => screen.is_transparent(),
            State::Options(screen) => screen.is_transparent(),
        }
    }

    pub fn is_transcendent(&self) -> bool {
        match self {
            State::Menu(screen) => screen.is_transcendent(),
            State::GameIntro(screen) => screen.is_transcendent(),
            State::Game(screen) => screen.is_transcendent(),
            State::Pause(screen) => screen.is_transcendent(),
            State::Options(screen) => screen.is_transcendent(),
        }
    }

    pub fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        match self {
            State::Menu(screen) => screen.get_post_process_shader(),
            State::GameIntro(screen) => screen.get_post_process_shader(),
            State::Game(screen) => screen.get_post_process_shader(),
            State::Pause(screen) => screen.get_post_process_shader(),
            State::Options(screen) => screen.get_post_process_shader(),
        }
    }
}

pub enum Transition {
    None,
    Push(State),               // Push a State on Top of the Stack
    Switch(StateTag),          // Reuse existing or push fresh
    Pop,                       // remove state on Top of The Stack
    Swap(State),               // pop current, push new
    ClearAndPush(State),       // pop all, push new
    Remove(StateTag),          // remove a state from anywhere in the stack
    Clear,                     // pop all states
    Multiple(Vec<Transition>), // apply multiple transitions in sequence
}

pub struct StackEntry {
    pub state: State,
    pub has_entered: bool,
    pub has_activated: bool,
}

pub struct StateManager {
    stack: Vec<StackEntry>,
    transitions: Vec<Transition>,
}

impl StateManager {
    pub fn new(tag: StateTag) -> Self {
        let state = State::from_tag(tag);
        Self {
            stack: vec![StackEntry {
                state,
                has_entered: false,
                has_activated: false,
            }],
            transitions: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn has_state(&self, tag: StateTag) -> bool {
        self.stack.iter().any(|s| s.state.tag() == tag)
    }

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32, audio: &mut AudioManager<'_, '_>) {
        if self.stack.is_empty() {
            return;
        }

        let has_save = self.has_state(StateTag::Game);

        let mut start_idx = self.stack.len() - 1;
        while start_idx > 0 {
            if !self.stack[start_idx].state.is_transcendent() {
                break;
            }
            start_idx -= 1;
        }

        let mut transitions = Vec::new();
        for entry in &mut self.stack[start_idx..] {
            if let Some(t) = entry.state.update(rl, dt, has_save, audio) {
                transitions.push(t);
            }
        }
        for t in transitions {
            self.add_transition(t);
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
            top.state.on_deactivate();
            top.has_activated = false;
        }
        self.stack.push(StackEntry {
            state,
            has_entered: false,
            has_activated: false,
        });
    }

    pub fn pop(&mut self) {
        if let Some(mut top) = self.stack.pop() {
            top.state.on_deactivate();
            top.state.on_exit();
        }
        if let Some(new_top) = self.stack.last_mut() {
            new_top.has_activated = false;
        }
    }

    pub fn swap(&mut self, state: State) {
        if let Some(mut top) = self.stack.pop() {
            top.state.on_deactivate();
            top.state.on_exit();
        }
        self.stack.push(StackEntry {
            state,
            has_entered: false,
            has_activated: false,
        });
    }

    pub fn clear_and_push(&mut self, state: State) {
        while let Some(mut top) = self.stack.pop() {
            top.state.on_deactivate();
            top.state.on_exit();
        }
        self.stack.push(StackEntry {
            state,
            has_entered: false,
            has_activated: false,
        });
    }

    pub fn switch(&mut self, tag: StateTag) {
        let position = self.stack.iter().position(|s| s.state.tag() == tag);
        if let Some(pos) = position {
            if pos < self.stack.len() - 1 {
                if let Some(top) = self.stack.last_mut() {
                    top.state.on_deactivate();
                    top.has_activated = false;
                }
                let mut existing_state = self.stack.remove(pos);
                existing_state.has_activated = false;
                self.stack.push(existing_state);
            }
        } else {
            if let Some(top) = self.stack.last_mut() {
                top.state.on_deactivate();
                top.has_activated = false;
            }
            let state = State::from_tag(tag);
            self.stack.push(StackEntry {
                state,
                has_entered: false,
                has_activated: false,
            });
        }
    }

    pub fn remove(&mut self, tag: StateTag) {
        let position = self.stack.iter().position(|s| s.state.tag() == tag);
        if let Some(pos) = position {
            let mut entry = self.stack.remove(pos);
            entry.state.on_deactivate();
            entry.state.on_exit();
            if pos == self.stack.len() {
                if let Some(new_top) = self.stack.last_mut() {
                    new_top.has_activated = false;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        while let Some(mut top) = self.stack.pop() {
            top.state.on_deactivate();
            top.state.on_exit();
        }
    }

    pub fn init_resources(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        for entry in &mut self.stack {
            if let State::Menu(menu) = &mut entry.state {
                menu.init_shader(rl, thread);
            }
        }
    }

    pub fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        if let Some(top) = self.stack.last_mut() {
            top.state.get_post_process_shader()
        } else {
            None
        }
    }

    pub fn current_state_uses_4_3(&self) -> bool {
        if let Some(top) = self.stack.last() {
            match top.state.tag() {
                StateTag::Menu | StateTag::GameIntro => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn late_update(&mut self) {
        let pending: Vec<Transition> = self.transitions.drain(..).collect();
        for transition in pending {
            self.apply_transition(transition);
        }
    }

    fn apply_transition(&mut self, transition: Transition) {
        match transition {
            Transition::None => {}
            Transition::Push(state) => self.push(state),
            Transition::Switch(tag) => self.switch(tag),
            Transition::Pop => self.pop(),
            Transition::Swap(state) => self.swap(state),
            Transition::ClearAndPush(state) => self.clear_and_push(state),
            Transition::Remove(tag) => self.remove(tag),
            Transition::Clear => self.clear(),
            Transition::Multiple(list) => {
                for t in list {
                    self.apply_transition(t);
                }
            }
        }
    }

    pub fn apply_lifecycles(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        resources: &mut ResourceManager,
    ) {
        // 1. Enter any unentered states
        for entry in &mut self.stack {
            if !entry.has_entered {
                entry.state.on_enter(rl, thread, resources);
                entry.has_entered = true;
            }
        }

        // 2. Activate the top state if not already active
        if let Some(top) = self.stack.last_mut() {
            if !top.has_activated {
                top.state.on_activate(rl, thread, resources);
                top.has_activated = true;
            }
        }
    }

    pub fn draw(
        &mut self,
        canvas: &mut RaylibTextureMode<RaylibHandle>,
        resources: &ResourceManager,
    ) {
        if self.stack.is_empty() {
            return;
        }

        let mut start_idx = self.stack.len() - 1;
        while start_idx > 0 {
            if !self.stack[start_idx].state.is_transparent() {
                break;
            }
            start_idx -= 1;
        }

        for i in start_idx..self.stack.len() {
            self.stack[i].state.draw(canvas, resources);
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
        assert_eq!(manager.stack[0].state.tag(), StateTag::Menu);

        // Switch to a new state: Game
        manager.switch(StateTag::Game);
        assert_eq!(manager.stack.len(), 2);
        assert_eq!(manager.stack[0].state.tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].state.tag(), StateTag::Game);
    }

    #[test]
    fn test_switch_existing_state_middle() {
        let mut manager = StateManager::new(StateTag::Menu);
        manager.switch(StateTag::Game);
        manager.switch(StateTag::Pause);

        // Stack should be [Menu, Game, Pause]
        assert_eq!(manager.stack.len(), 3);
        assert_eq!(manager.stack[0].state.tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].state.tag(), StateTag::Game);
        assert_eq!(manager.stack[2].state.tag(), StateTag::Pause);

        // Switch to Game (which already exists at index 1)
        manager.switch(StateTag::Game);

        // Stack should now be [Menu, Pause, Game]
        assert_eq!(manager.stack.len(), 3);
        assert_eq!(manager.stack[0].state.tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].state.tag(), StateTag::Pause);
        assert_eq!(manager.stack[2].state.tag(), StateTag::Game);
    }

    #[test]
    fn test_switch_existing_state_top() {
        let mut manager = StateManager::new(StateTag::Menu);
        manager.switch(StateTag::Game);

        // Stack should be [Menu, Game]
        assert_eq!(manager.stack.len(), 2);
        assert_eq!(manager.stack[0].state.tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].state.tag(), StateTag::Game);

        // Switch to Game (which is already top)
        manager.switch(StateTag::Game);

        // Stack should remain [Menu, Game]
        assert_eq!(manager.stack.len(), 2);
        assert_eq!(manager.stack[0].state.tag(), StateTag::Menu);
        assert_eq!(manager.stack[1].state.tag(), StateTag::Game);
    }

    #[test]
    fn test_screen_transparency() {
        let menu = State::from_tag(StateTag::Menu);
        let game_intro = State::from_tag(StateTag::GameIntro);
        let game = State::from_tag(StateTag::Game);
        let pause = State::from_tag(StateTag::Pause);
        let options = State::from_tag(StateTag::Options);

        assert!(!menu.is_transparent());
        assert!(!game_intro.is_transparent());
        assert!(!game.is_transparent());
        assert!(pause.is_transparent());
        assert!(options.is_transparent());
    }

    #[test]
    fn test_screen_transcendence() {
        let menu = State::from_tag(StateTag::Menu);
        let game_intro = State::from_tag(StateTag::GameIntro);
        let game = State::from_tag(StateTag::Game);
        let pause = State::from_tag(StateTag::Pause);
        let options = State::from_tag(StateTag::Options);

        assert!(!menu.is_transcendent());
        assert!(!game_intro.is_transcendent());
        assert!(!game.is_transcendent());
        assert!(!pause.is_transcendent());
        assert!(!options.is_transcendent());
    }
}
