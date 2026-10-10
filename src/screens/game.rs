use crate::prelude::*;
use crate::scenes::{ActOneScene, SceneManager};
use sola_raylib::prelude::*;

pub struct Game {
    scene_manager: SceneManager,
}

impl Game {
    pub fn new() -> Self {
        Self {
            scene_manager: SceneManager::new(Box::new(ActOneScene::new())),
        }
    }

    pub fn scene_manager(&self) -> &SceneManager {
        &self.scene_manager
    }

    pub fn scene_manager_mut(&mut self) -> &mut SceneManager {
        &mut self.scene_manager
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for Game {
    fn on_enter(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
        self.scene_manager.on_enter(rl, thread, resources);
    }

    fn update(&mut self, ctx: &mut Context) -> Option<Transition> {
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) || ctx.rl.is_key_pressed(KeyboardKey::KEY_P) {
            return Some(Transition::Push(State::from_tag(StateTag::Pause)));
        }

        self.scene_manager.update(ctx);

        None
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
        self.scene_manager.draw(canvas, resources);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_scene_initialization() {
        let game = Game::new();
        assert_eq!(game.scene_manager().current_tag(), Some(SceneTag::ActOne));
    }

    #[test]
    fn test_game_scene_switch_to_act_two() {
        let mut game = Game::new();
        game.scene_manager_mut()
            .switch_scene(Box::new(ActTwoScene::new()));
        assert_eq!(game.scene_manager().current_tag(), Some(SceneTag::ActTwo));
    }
}
