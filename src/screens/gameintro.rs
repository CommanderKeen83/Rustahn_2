use crate::prelude::*;
use sola_raylib::prelude::*;

const INTRO_TEXT: &str = "\n\
Long ago, a contract was made and the Realm of Enemata \n\
was sealed away.        \n\
For centuries, the Kingdom of Rabahn prospered, peace held. \n\
The old stories became songs. The songs became whispers.                              \n\
But something changed in the Kingdom of Rabahn.                              \n\
The harvests grew thin. The nights became restless.            \n\
The contract expired. On a calm evening, a storm brewed up -          \n\
and the Seal cracked.";

pub struct GameIntro {
    passed_time: f32,
    reveal_speed: f32,
    stormcloud_position: Vector2,
    darkness: f32,
    fade_speed: f32,
    flicker_shader: Option<Shader>,
    crt_shader: Option<Shader>,
    time_loc_flicker: i32,
    time_loc_crt: i32,
}

impl GameIntro {
    pub fn new() -> Self {
        Self {
            passed_time: 0.0,
            reveal_speed: 12.0,
            stormcloud_position: Vector2::new(-1650.0, -700.0),
            darkness: 1.0,
            fade_speed: 0.02,
            flicker_shader: None,
            crt_shader: None,
            time_loc_flicker: 0,
            time_loc_crt: 0,
        }
    }

    pub fn on_enter(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        resources: &mut ResourceManager,
    ) {
        // self.passed_time = 0.0;
        // self.darkness = 1.0;
        // self.stormcloud_position = Vector2::new(-1650.0, -500.0);

        if let Err(e) = resources.load_texture(
            rl,
            thread,
            SpriteID::GameIntroBackground,
            "resources/graphics/gameintrobackground.png",
        ) {
            eprintln!("Failed to load intro background: {}", e);
        }

        if let Err(e) = resources.load_texture(
            rl,
            thread,
            SpriteID::GameIntroStormCloud,
            "resources/graphics/stormcloud4.png",
        ) {
            eprintln!("Failed to load storm cloud texture: {}", e);
        }

        if std::path::Path::new("resources/graphics/windowlamp.png").exists() {
            let _ = resources.load_texture(
                rl,
                thread,
                SpriteID::GameIntroWindowLamp,
                "resources/graphics/windowlamp.png",
            );
        }

        let flicker = rl.load_shader(thread, None, Some("src/shaders/flicker.fs"));
        self.time_loc_flicker = flicker.get_shader_location("time");
        self.flicker_shader = Some(flicker);

        let crt = rl.load_shader(thread, None, Some("src/shaders/game_intro_crt_shader.fs"));
        self.time_loc_crt = crt.get_shader_location("time");
        self.crt_shader = Some(crt);
    }

    pub fn on_activate(
        &mut self,
        _rl: &mut RaylibHandle,
        _thread: &RaylibThread,
        _resources: &mut ResourceManager,
    ) {
    }

    pub fn on_deactivate(&mut self) {}

    pub fn on_exit(&mut self) {}

    pub fn update(
        &mut self,
        rl: &RaylibHandle,
        dt: f32,
        audio: &mut AudioManager<'_, '_>,
    ) -> Option<Transition> {
        audio.play_music(MusicID::GameIntroScreen);
        self.passed_time += dt;

        if let Some(ref mut flicker) = self.flicker_shader {
            flicker.set_shader_value(self.time_loc_flicker, self.passed_time);
        }
        if let Some(ref mut crt) = self.crt_shader {
            crt.set_shader_value(self.time_loc_crt, self.passed_time);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
            || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
            || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
        {
            return Some(Transition::Multiple(vec![
                Transition::Remove(StateTag::GameIntro),
                Transition::Switch(StateTag::Game),
            ]));
        }

        if self.passed_time > 20.0 {
            self.darkness = f32::max(0.353, self.darkness - (self.fade_speed * dt));
        }
        if self.stormcloud_position.x < -180.0 {
            self.stormcloud_position.x += dt * 25.0;
        }

        None
    }

    pub fn draw(
        &mut self,
        canvas: &mut RaylibTextureMode<RaylibHandle>,
        resources: &ResourceManager,
    ) {
        let intensity = (self.darkness * 255.0) as u8;
        let tint = Color::new(intensity, intensity, intensity, 255);

        if let Some(texture_background) = resources.get_texture(&SpriteID::GameIntroBackground) {
            canvas.draw_texture_pro(
                texture_background,
                Rectangle::new(0.0, 0.0, 320.0, 180.0),
                Rectangle::new(0.0, 0.0, 1280.0, 720.0),
                Vector2::new(0.0, 0.0),
                0.0,
                tint,
            );
        }

        if let Some(texture_stormcloud) = resources.get_texture(&SpriteID::GameIntroStormCloud) {
            let scaled_cloud_x = (self.stormcloud_position.x / 800.0) * 1280.0;
            let scaled_cloud_y = (self.stormcloud_position.y / 600.0) * 720.0;

            // The texture is 1024x1024. In the Zig version, the screen was 800 wide,
            // so a 1024 wide texture at x = -180 covered the screen (right edge at 844).
            // For the 1280x720 Rust version, we scale the cloud's size by 1.6 (1280/800)
            // to maintain the same screen coverage, drawing it at 1638x1638 size.
            canvas.draw_texture_pro(
                texture_stormcloud,
                Rectangle::new(0.0, 0.0, 1024.0, 1024.0),
                Rectangle::new(scaled_cloud_x, scaled_cloud_y, 1638.0, 1638.0),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
        }

        let shader_ready = self.flicker_shader.is_some();
        if shader_ready {
            if let Some(ref mut flicker_shader) = self.flicker_shader {
                let mut shader_mode = canvas.begin_shader_mode(flicker_shader);
                if let Some(texture_windowlamp) =
                    resources.get_texture(&SpriteID::GameIntroWindowLamp)
                {
                    shader_mode.draw_texture_pro(
                        texture_windowlamp,
                        Rectangle::new(0.0, 0.0, 15.0, 10.0),
                        Rectangle::new(1110.0, 662.0, 59.0, 30.0),
                        Vector2::new(0.0, 0.0),
                        0.0,
                        Color::WHITE,
                    );
                }
            }
        } else {
            if let Some(texture_windowlamp) = resources.get_texture(&SpriteID::GameIntroWindowLamp)
            {
                canvas.draw_texture_pro(
                    texture_windowlamp,
                    Rectangle::new(0.0, 0.0, 15.0, 10.0),
                    Rectangle::new(1110.0, 662.0, 59.0, 30.0),
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }
        }

        let chars_to_show = (self.passed_time * self.reveal_speed) as usize;
        let visible_text: String = INTRO_TEXT.chars().take(chars_to_show).collect();

        let text_x = (50.0 / 800.0 * 1280.0) as i32;
        let text_y = (200.0 / 600.0 * 720.0) as i32;
        let text_size = (20.0 / 600.0 * 720.0) as i32;
        canvas.draw_text(&visible_text, text_x, text_y, text_size, Color::WHITE);

        if self.passed_time > 42.0 {
            let sin_val = f32::sin(self.passed_time);
            let flicker_alpha = (((sin_val + 1.0) / 2.0) * 255.0) as u8;
            let skip_color = Color::new(255, 210, 80, flicker_alpha);
            let skip_x = (50.0 / 800.0 * 1280.0) as i32;
            let skip_y = (500.0 / 600.0 * 720.0) as i32;
            canvas.draw_text(
                "press space or enter to continue",
                skip_x,
                skip_y,
                text_size,
                skip_color,
            );
        }
    }

    pub fn is_transparent(&self) -> bool {
        false
    }

    pub fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        self.crt_shader.as_mut()
    }

    pub fn is_transcendent(&self) -> bool {
        false
    }
}
