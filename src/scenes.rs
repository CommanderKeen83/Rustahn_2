pub mod act_one;

use crate::prelude::*;
use sola_raylib::prelude::*;

pub use act_one::{ActOnePhase, ActOneScene, IntroPan};
pub type LevelOneScene = ActOneScene;
pub type ActOne = ActOneScene;

/// Actions a scene can request from the scene manager.
pub enum SceneAction {
    Switch(Box<dyn Scene>),
}

/// Trait defining the lifecycle and rendering of a game level or scene.
pub trait Scene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _resources: &mut ResourceManager) {}
    fn on_exit(&mut self) {}
    fn update(&mut self, _ctx: &mut Context) -> Option<SceneAction> {
        None
    }
    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager);
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
