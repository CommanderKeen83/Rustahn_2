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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StateSet(u8);

impl StateSet {
    pub fn contains(&self, tag: StateTag) -> bool {
        (self.0 & (1 << (tag as u8))) != 0
    }

    pub fn insert(&mut self, tag: StateTag) {
        self.0 |= 1 << (tag as u8);
    }
}

pub struct UpdateContext<'a> {
    pub rl: &'a RaylibHandle,
    pub dt: f32,
    pub audio: AudioManager<'a, 'a>,
    active_states: StateSet,
}

impl<'a> UpdateContext<'a> {
    pub fn new(rl: &'a RaylibHandle, dt: f32, audio: AudioManager<'a, 'a>, active_states: StateSet) -> Self {
        Self {
            rl,
            dt,
            audio,
            active_states,
        }
    }

    pub fn has_state(&self, tag: StateTag) -> bool {
        self.active_states.contains(tag)
    }
}

pub type Context<'a> = UpdateContext<'a>;

pub trait Screen {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _resources: &mut ResourceManager) {}
    fn on_activate(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _resources: &mut ResourceManager) {}
    fn on_deactivate(&mut self) {}
    fn on_exit(&mut self) {}

    fn update(&mut self, _ctx: &mut Context) -> Option<Transition> {
        None
    }
    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager);

    fn is_transparent(&self) -> bool {
        false
    }

    fn is_transcendent(&self) -> bool {
        false
    }

    fn wants_4_3(&self) -> bool {
        false
    }

    fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        None
    }
}

macro_rules! delegate_screen {
    ($self:expr, $screen:ident => $expr:expr) => {
        match $self {
            State::Menu($screen) => $expr,
            State::GameIntro($screen) => $expr,
            State::Game($screen) => $expr,
            State::Pause($screen) => $expr,
            State::Options($screen) => $expr,
        }
    };
}

impl Screen for State {
    fn on_enter(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
        delegate_screen!(self, s => s.on_enter(rl, thread, resources));
    }

    fn on_activate(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
        delegate_screen!(self, s => s.on_activate(rl, thread, resources));
    }

    fn on_deactivate(&mut self) {
        delegate_screen!(self, s => s.on_deactivate());
    }

    fn on_exit(&mut self) {
        delegate_screen!(self, s => s.on_exit());
    }

    fn update(&mut self, ctx: &mut Context) -> Option<Transition> {
        delegate_screen!(self, s => s.update(ctx))
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
        delegate_screen!(self, s => s.draw(canvas, resources));
    }

    fn is_transparent(&self) -> bool {
        delegate_screen!(self, s => s.is_transparent())
    }

    fn is_transcendent(&self) -> bool {
        delegate_screen!(self, s => s.is_transcendent())
    }

    fn wants_4_3(&self) -> bool {
        delegate_screen!(self, s => s.wants_4_3())
    }

    fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        delegate_screen!(self, s => s.get_post_process_shader())
    }
}

pub enum Transition {
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

impl StackEntry {
    pub fn deactivate(&mut self) {
        if self.has_activated {
            self.state.on_deactivate();
            self.has_activated = false;
        }
    }

    pub fn exit(mut self) {
        self.deactivate();
        if self.has_entered {
            self.state.on_exit();
        }
    }
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

    pub fn active_states(&self) -> StateSet {
        let mut set = StateSet::default();
        for entry in &self.stack {
            set.insert(entry.state.tag());
        }
        set
    }

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32, audio: &mut AudioManager<'_, '_>) {
        if self.stack.is_empty() {
            return;
        }

        let mut start_idx = self.stack.len() - 1;
        while start_idx > 0 {
            if !self.stack[start_idx].state.is_transcendent() {
                break;
            }
            start_idx -= 1;
        }

        let mut ctx = UpdateContext::new(rl, dt, audio.reborrow(), self.active_states());

        let mut transitions = Vec::new();
        for entry in &mut self.stack[start_idx..] {
            if let Some(t) = entry.state.update(&mut ctx) {
                transitions.push(t);
            }
        }
        for t in transitions {
            self.add_transition(t);
        }
    }

    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    pub fn push(&mut self, state: State) {
        if let Some(top) = self.stack.last_mut() {
            top.deactivate();
        }
        self.stack.push(StackEntry {
            state,
            has_entered: false,
            has_activated: false,
        });
    }

    pub fn pop(&mut self) {
        if let Some(top) = self.stack.pop() {
            top.exit();
        }
    }

    pub fn swap(&mut self, state: State) {
        if let Some(top) = self.stack.pop() {
            top.exit();
        }
        self.stack.push(StackEntry {
            state,
            has_entered: false,
            has_activated: false,
        });
    }

    pub fn clear_and_push(&mut self, state: State) {
        self.clear();
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
                    top.deactivate();
                }
                let existing_state = self.stack.remove(pos);
                self.stack.push(existing_state);
            }
        } else {
            if let Some(top) = self.stack.last_mut() {
                top.deactivate();
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
            let entry = self.stack.remove(pos);
            entry.exit();
        }
    }

    pub fn clear(&mut self) {
        while let Some(top) = self.stack.pop() {
            top.exit();
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
        self.stack.last().map_or(false, |top| top.state.wants_4_3())
    }

    pub fn late_update(&mut self) {
        let pending: Vec<Transition> = self.transitions.drain(..).collect();
        for transition in pending {
            self.apply_transition(transition);
        }
    }

    fn apply_transition(&mut self, transition: Transition) {
        match transition {
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

    pub fn apply_lifecycles(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
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

    pub fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
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

    #[test]
    fn test_screen_aspect_ratio() {
        let menu = State::from_tag(StateTag::Menu);
        let game_intro = State::from_tag(StateTag::GameIntro);
        let game = State::from_tag(StateTag::Game);
        let pause = State::from_tag(StateTag::Pause);
        let options = State::from_tag(StateTag::Options);

        assert!(menu.wants_4_3());
        assert!(game_intro.wants_4_3());
        assert!(!game.wants_4_3());
        assert!(!pause.wants_4_3());
        assert!(!options.wants_4_3());
    }

    #[test]
    fn test_lifecycle_guards() {
        let mut manager = StateManager::new(StateTag::Menu);
        assert_eq!(manager.stack.len(), 1);
        assert!(!manager.stack[0].has_entered);
        assert!(!manager.stack[0].has_activated);

        // Pushing without applying lifecycles must not deactivate the unactivated top
        manager.push(State::from_tag(StateTag::Game));
        assert_eq!(manager.stack.len(), 2);
        assert!(!manager.stack[0].has_activated);
        assert!(!manager.stack[1].has_entered);
        assert!(!manager.stack[1].has_activated);

        // Popping a state that never entered or activated must safely discard without panic
        manager.pop();
        assert_eq!(manager.stack.len(), 1);

        // Clear on unentered states
        manager.clear();
        assert!(manager.is_empty());
    }
}
