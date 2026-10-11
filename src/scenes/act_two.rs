use crate::prelude::*;
use crate::scenes::{Scene, SceneAction, SceneTag};
use crate::systems::collision;
use sola_raylib::prelude::*;

/// Act Two scene: testing ground for gameplay mechanics.
///
/// Features a solid green collision rectangle spanning the entire floor of the camera viewport
/// and a purple player rectangle in the center of the viewport that naturally falls under gravity
/// and rests on the ground.
pub struct ActTwoScene {
    world: World,
    player_id: Option<EntityID>,
    floor_id: Option<EntityID>,
    pub camera: GameCamera,
}

impl ActTwoScene {
    /// Viewport floor collision rectangle dimensions and center position.
    /// Spans the entire virtual viewport width (320 px) along the bottom (y = 160.0..180.0).
    pub const FLOOR_WIDTH: f32 = VIRTUAL_WIDTH as f32; // 320.0
    pub const FLOOR_HEIGHT: f32 = 20.0;
    pub const FLOOR_X: f32 = VIRTUAL_WIDTH as f32 * 0.5; // 160.0
    pub const FLOOR_Y: f32 = VIRTUAL_HEIGHT as f32 - Self::FLOOR_HEIGHT * 0.5; // 170.0

    /// Player rectangle dimensions and initial center position in the middle of the camera viewport.
    pub const PLAYER_WIDTH: f32 = 20.0;
    pub const PLAYER_HEIGHT: f32 = 20.0;
    pub const PLAYER_SPAWN_X: f32 = VIRTUAL_WIDTH as f32 * 0.5; // 160.0
    pub const PLAYER_SPAWN_Y: f32 = VIRTUAL_HEIGHT as f32 * 0.5; // 90.0

    /// Expected resting Y coordinate for the player center once landed on top of the floor collider:
    /// floor top edge (160.0) - player half height (10.0) = 150.0.
    pub const PLAYER_RESTING_Y: f32 = (Self::FLOOR_Y - Self::FLOOR_HEIGHT * 0.5) - Self::PLAYER_HEIGHT * 0.5;

    pub fn new() -> Self {
        let mut world = World::new();

        // 1. Solid floor collision rectangle spanning the whole camera viewport from left to right
        let floor_id = world
            .spawn(
                EntityType::Ground,
                Vector2::new(Self::FLOOR_X, Self::FLOOR_Y),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::None,
            )
            .with_collider(Collider::new(Self::FLOOR_WIDTH, Self::FLOOR_HEIGHT))
            .with_physics(PhysicsData::static_body())
            .build();

        // 2. Player entity positioned in the middle of the view
        let player_id = world
            .spawn(
                EntityType::Player,
                Vector2::new(Self::PLAYER_SPAWN_X, Self::PLAYER_SPAWN_Y),
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_physics_preset(PhysicsPreset::Normal)
            .with_collider(Collider::new(Self::PLAYER_WIDTH, Self::PLAYER_HEIGHT))
            .build();

        Self {
            world,
            player_id: Some(player_id),
            floor_id: Some(floor_id),
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

    pub fn player_id(&self) -> Option<EntityID> {
        self.player_id
    }

    pub fn player_position(&self) -> Option<Vector2> {
        self.player_id.map(|id| self.world.positions[id.0])
    }

    pub fn floor_id(&self) -> Option<EntityID> {
        self.floor_id
    }

    pub fn floor_position(&self) -> Option<Vector2> {
        self.floor_id.map(|id| self.world.positions[id.0])
    }

    pub fn ground_id(&self) -> Option<EntityID> {
        self.floor_id
    }

    pub fn ground_position(&self) -> Option<Vector2> {
        self.floor_position()
    }

    /// Resets the player entity back to the center of the viewport with zero velocity.
    pub fn reset_player(&mut self) {
        if let Some(player_id) = self.player_id {
            if player_id.0 < self.world.next_entity_id {
                self.world.positions[player_id.0] = Vector2::new(Self::PLAYER_SPAWN_X, Self::PLAYER_SPAWN_Y);
                self.world.velocities[player_id.0] = Vector2::new(0.0, 0.0);
                self.world.rotations[player_id.0] = 0.0;
            }
        }
    }

    /// Updates physics integration and collision resolution.
    pub fn update_physics(&mut self, dt: f32) {
        physics::update(&mut self.world, dt);
        collision::update(&mut self.world, dt);
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

    fn update(&mut self, ctx: &mut Context) -> Option<SceneAction> {
        // Press R to reset the player to center for easy repeatable fall testing
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_R) {
            self.reset_player();
        }

        self.update_physics(ctx.dt);
        None
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, _resources: &ResourceManager) {
        canvas.clear_background(Color::BLACK);

        for i in 0..self.world.next_entity_id {
            if !self.world.alive[i] {
                continue;
            }
            let col = match self.world.colliders[i] {
                Some(c) => c,
                None => continue,
            };

            let color = if Some(EntityID(i)) == self.floor_id || self.world.tags[i].is_solid_obstacle() {
                Color::GREEN
            } else if Some(EntityID(i)) == self.player_id || self.world.tags[i] == EntityType::Player {
                Color::PURPLE
            } else {
                Color::GRAY
            };

            let pos = self.world.positions[i];
            let rotation = self.world.rotations[i];
            let screen_w = col.width * VIRTUAL_SCALE;
            let screen_h = col.height * VIRTUAL_SCALE;

            if rotation == 0.0 {
                let screen_x = (pos.x + col.offset.x - col.width * 0.5 - self.camera.x()) * VIRTUAL_SCALE;
                let screen_y = (pos.y + col.offset.y - col.height * 0.5 - self.camera.y()) * VIRTUAL_SCALE;

                canvas.draw_rectangle(
                    screen_x.round() as i32,
                    screen_y.round() as i32,
                    screen_w.round() as i32,
                    screen_h.round() as i32,
                    color,
                );
            } else {
                let dest = Rectangle::new(
                    (pos.x + col.offset.x - self.camera.x()) * VIRTUAL_SCALE,
                    (pos.y + col.offset.y - self.camera.y()) * VIRTUAL_SCALE,
                    screen_w,
                    screen_h,
                );
                let origin = Vector2::new(dest.width * 0.5, dest.height * 0.5);
                canvas.draw_rectangle_pro(dest, origin, rotation, color);
            }
        }
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

        let floor_id = scene.floor_id().expect("Floor entity should exist");
        assert_eq!(scene.world().tags[floor_id.0], EntityType::Ground);
        assert_eq!(
            scene.world().positions[floor_id.0],
            Vector2::new(ActTwoScene::FLOOR_X, ActTwoScene::FLOOR_Y)
        );
        let floor_col = scene.world().colliders[floor_id.0].expect("Floor should have collider");
        assert_eq!(floor_col.width, ActTwoScene::FLOOR_WIDTH);
        assert_eq!(floor_col.height, ActTwoScene::FLOOR_HEIGHT);
        assert!(floor_col.is_solid);
        let floor_phys = scene.world().physics_data[floor_id.0].as_ref().expect("Floor should have physics");
        assert!(floor_phys.is_static);

        let player_id = scene.player_id().expect("Player entity should exist");
        assert_eq!(scene.world().tags[player_id.0], EntityType::Player);
        assert_eq!(
            scene.world().positions[player_id.0],
            Vector2::new(ActTwoScene::PLAYER_SPAWN_X, ActTwoScene::PLAYER_SPAWN_Y)
        );
        let player_col = scene.world().colliders[player_id.0].expect("Player should have collider");
        assert_eq!(player_col.width, ActTwoScene::PLAYER_WIDTH);
        assert_eq!(player_col.height, ActTwoScene::PLAYER_HEIGHT);
        assert!(player_col.is_solid);
        let player_phys = scene.world().physics_data[player_id.0].as_ref().expect("Player should have physics");
        assert!(!player_phys.is_static);
    }

    #[test]
    fn test_floor_spans_entire_camera_viewport() {
        let scene = ActTwoScene::new();
        let floor_id = scene.floor_id().unwrap();
        let floor_col = scene.world().colliders[floor_id.0].unwrap();
        let floor_pos = scene.world().positions[floor_id.0];

        let (min_x, min_y, max_x, max_y) = floor_col.aabb(floor_pos);
        // Spans from left side (0.0) to right side (320.0) of the viewport
        assert_eq!(min_x, scene.camera().x());
        assert_eq!(max_x, scene.camera().x() + scene.camera().viewport_width);
        // Floor bottom aligns with viewport bottom (180.0)
        assert_eq!(max_y, scene.camera().y() + scene.camera().viewport_height);
        assert_eq!(min_y, scene.camera().y() + scene.camera().viewport_height - ActTwoScene::FLOOR_HEIGHT);
    }

    #[test]
    fn test_player_spawn_in_middle_of_view() {
        let scene = ActTwoScene::new();
        let player_pos = scene.player_position().unwrap();
        assert_eq!(player_pos.x, scene.camera().viewport_width * 0.5);
        assert_eq!(player_pos.y, scene.camera().viewport_height * 0.5);
    }

    #[test]
    fn test_player_naturally_falls_and_lands_on_ground() {
        let mut scene = ActTwoScene::new();
        let player_id = scene.player_id().unwrap();

        let initial_y = scene.world().positions[player_id.0].y;
        assert_eq!(initial_y, ActTwoScene::PLAYER_SPAWN_Y);

        // Step physics forward a bit; player should be moving downwards
        scene.update_physics(0.1);
        let falling_y = scene.world().positions[player_id.0].y;
        assert!(falling_y > initial_y, "Player should fall under gravity");
        assert!(scene.world().velocities[player_id.0].y > 0.0, "Downward velocity should be positive");

        // Simulate falling until the player lands on the floor
        for _ in 0..300 {
            scene.update_physics(0.016);
        }

        let settled_y = scene.world().positions[player_id.0].y;
        assert!(
            (settled_y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3,
            "Player should settle at resting Y ({}), but was at {}",
            ActTwoScene::PLAYER_RESTING_Y,
            settled_y
        );
        assert_eq!(
            scene.world().velocities[player_id.0].y,
            0.0,
            "Velocity should be 0 once settled on ground"
        );

        // Continue running physics; player must stay resting on the ground and not fall through
        for _ in 0..100 {
            scene.update_physics(0.016);
        }

        let still_settled_y = scene.world().positions[player_id.0].y;
        assert!(
            (still_settled_y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3,
            "Player should remain at resting Y ({}), but was at {}",
            ActTwoScene::PLAYER_RESTING_Y,
            still_settled_y
        );
    }

    #[test]
    fn test_player_falls_through_without_floor_collider() {
        let mut scene = ActTwoScene::new();
        let floor_id = scene.floor_id().unwrap();
        // Remove floor collider
        scene.world_mut().colliders[floor_id.0] = None;

        for _ in 0..600 {
            scene.update_physics(0.016);
        }

        let player_id = scene.player_id().unwrap();
        let fallen_y = scene.world().positions[player_id.0].y;
        assert!(
            fallen_y > ActTwoScene::FLOOR_Y + 20.0,
            "Player should fall through the floor when no floor collider exists, got {}",
            fallen_y
        );
    }

    #[test]
    fn test_no_objects_with_physics_and_collider_fall_through() {
        let mut scene = ActTwoScene::new();

        // Spawn multiple test objects at different horizontal locations and heights (distinct from player x=160)
        let test_positions = [
            Vector2::new(30.0, 20.0),
            Vector2::new(100.0, 50.0),
            Vector2::new(180.0, 70.0),
            Vector2::new(230.0, 30.0),
            Vector2::new(290.0, 60.0),
        ];

        let mut spawned_ids = Vec::new();
        for &pos in &test_positions {
            let id = scene
                .world_mut()
                .spawn(
                    EntityType::Obstacle,
                    pos,
                    Vector2::new(0.0, 0.0),
                    0.0,
                    true,
                    SpriteID::None,
                )
                .with_physics_preset(PhysicsPreset::Normal)
                .with_collider(Collider::new(16.0, 16.0))
                .build();
            spawned_ids.push(id);
        }

        // Run physics simulation until all objects have landed
        for _ in 0..600 {
            scene.update_physics(0.016);
        }

        // Expected resting Y for a 16x16 collider:
        // floor top (160.0) - half height (8.0) = 152.0
        let expected_resting_y = (ActTwoScene::FLOOR_Y - ActTwoScene::FLOOR_HEIGHT * 0.5) - 8.0;

        for id in spawned_ids {
            let y = scene.world().positions[id.0].y;
            assert!(
                (y - expected_resting_y).abs() < 1e-3,
                "Object {} should be resting on ground at {}, but was at {}",
                id.0,
                expected_resting_y,
                y
            );
        }
    }



    #[test]
    fn test_reset_player() {
        let mut scene = ActTwoScene::new();
        // Fall to ground
        for _ in 0..200 {
            scene.update_physics(0.016);
        }
        let player_id = scene.player_id().unwrap();
        assert!((scene.world().positions[player_id.0].y - ActTwoScene::PLAYER_RESTING_Y).abs() < 1e-3);

        // Reset
        scene.reset_player();
        assert_eq!(
            scene.world().positions[player_id.0],
            Vector2::new(ActTwoScene::PLAYER_SPAWN_X, ActTwoScene::PLAYER_SPAWN_Y)
        );
        assert_eq!(scene.world().velocities[player_id.0], Vector2::new(0.0, 0.0));
    }
}
