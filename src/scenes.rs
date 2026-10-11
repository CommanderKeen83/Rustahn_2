pub mod act_one;
pub mod act_two;
pub mod asteroid_field;

use crate::prelude::*;
use sola_raylib::prelude::*;

pub use crate::camera::{ActOnePhase, IntroPan, IntroPhase};
pub use act_one::ActOneScene;
pub use act_two::{ActTwoInput, ActTwoScene};
pub use asteroid_field::{Asteroid, AsteroidField};
pub type LevelOneScene = ActOneScene;
pub type ActOne = ActOneScene;
pub type LevelTwoScene = ActTwoScene;
pub type ActTwo = ActTwoScene;

/// Identifiers for game scenes / acts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneTag {
    ActOne,
    ActTwo,
}

/// Actions a scene can request from the scene manager.
pub enum SceneAction {
    Switch(Box<dyn Scene>),
}

impl std::fmt::Debug for SceneAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneAction::Switch(scene) => write!(f, "SceneAction::Switch({:?})", scene.tag()),
        }
    }
}

/// Trait defining the lifecycle and rendering of a game level or scene.
pub trait Scene: 'static {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _resources: &mut ResourceManager) {}
    fn on_exit(&mut self) {}
    fn update(&mut self, _ctx: &mut Context) -> Option<SceneAction> {
        None
    }
    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager);

    fn tag(&self) -> Option<SceneTag> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Manager responsible for hosting and transitioning between game scenes.
pub struct SceneManager {
    current_scene: Box<dyn Scene>,
}

impl SceneManager {
    pub fn new(initial_scene: Box<dyn Scene>) -> Self {
        Self {
            current_scene: initial_scene,
        }
    }

    pub fn current_scene(&self) -> &dyn Scene {
        &*self.current_scene
    }

    pub fn current_scene_mut(&mut self) -> &mut dyn Scene {
        &mut *self.current_scene
    }

    pub fn current_tag(&self) -> Option<SceneTag> {
        self.current_scene.tag()
    }

    pub fn switch_scene(&mut self, new_scene: Box<dyn Scene>) {
        self.current_scene.on_exit();
        self.current_scene = new_scene;
    }

    pub fn on_enter(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
        self.current_scene.on_enter(rl, thread, resources);
    }

    pub fn update(&mut self, ctx: &mut Context) {
        if let Some(action) = self.current_scene.update(ctx) {
            match action {
                SceneAction::Switch(next) => {
                    self.switch_scene(next);
                }
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
        self.current_scene.draw(canvas, resources);
    }
}

