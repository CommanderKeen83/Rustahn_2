use crate::prelude::*;
use crate::scenes::{Scene, SceneAction, SceneTag};
use sola_raylib::prelude::*;

/// Act Three scene: the next stage after completing Act Two.
pub struct ActThreeScene {
    world: World,
    pub camera: GameCamera,
    time: f32,
}

impl ActThreeScene {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            camera: GameCamera::new(),
            time: 0.0,
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

    pub fn time(&self) -> f32 {
        self.time
    }
}

impl Default for ActThreeScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for ActThreeScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _resources: &mut ResourceManager) {}

    fn on_exit(&mut self) {}

    fn update(&mut self, ctx: &mut Context) -> Option<SceneAction> {
        self.time += ctx.dt;
        None
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager) {
        canvas.clear_background(Color::new(12, 14, 24, 255));

        let title = "ACT THREE";
        let title_font_size = 36;
        let screen_center_x = (GAME_WIDTH / 2) as i32;
        let screen_center_y = (GAME_HEIGHT / 2) as i32;

        let title_offset = title_font_size * 5 / 2;
        canvas.draw_text(
            title,
            screen_center_x - title_offset,
            screen_center_y - 30,
            title_font_size,
            Color::GOLD,
        );

        let subtitle = "Exit reached! Act Two cleared.";
        let sub_font_size = 18;
        let sub_offset = sub_font_size * 14 / 2;
        canvas.draw_text(
            subtitle,
            screen_center_x - sub_offset,
            screen_center_y + 20,
            sub_font_size,
            Color::LIGHTGRAY,
        );
    }

    fn tag(&self) -> Option<SceneTag> {
        Some(SceneTag::ActThree)
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
    fn test_act_three_scene_initialization() {
        let scene = ActThreeScene::new();
        assert_eq!(scene.tag(), Some(SceneTag::ActThree));
        assert_eq!(scene.time(), 0.0);
    }
}
