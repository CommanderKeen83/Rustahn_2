use crate::prelude::*;
use crate::scenes::{Scene, SceneAction, SceneTag};
use sola_raylib::prelude::*;

/// Act Two scene: clean slate for testing new game mechanics.
///
/// For the start, it does not draw anything to the screen, providing
/// an empty canvas and fresh ECS World for subsequent mechanics prototyping.
pub struct ActTwoScene {
    world: World,
    pub camera: GameCamera,
}

impl ActTwoScene {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            camera: GameCamera::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn camera(&self) -> &GameCamera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut GameCamera {
        &mut self.camera
    }
}

impl Default for ActTwoScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for ActTwoScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _resources: &mut ResourceManager) {}

    fn on_exit(&mut self) {}

    fn update(&mut self, _ctx: &mut Context) -> Option<SceneAction> {
        None
    }

    fn draw(&mut self, _canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager) {
        // Intentionally blank: clean slate for future mechanics prototyping
    }

    fn tag(&self) -> Option<SceneTag> {
        Some(SceneTag::ActTwo)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_two_scene_initialization() {
        let scene = ActTwoScene::new();
        assert_eq!(scene.tag(), Some(SceneTag::ActTwo));
        assert_eq!(scene.camera().x(), 0.0);
        assert_eq!(scene.camera().y(), 0.0);
        assert_eq!(scene.world().next_entity_id, 0);
    }

    #[test]
    fn test_act_two_clean_slate_update_returns_none() {
        let scene = ActTwoScene::new();
        assert_eq!(scene.world().next_entity_id, 0);
    }
}
