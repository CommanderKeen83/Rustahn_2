use crate::prelude::*;
use sola_raylib::prelude::{Music, RaylibAudio, Sound};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Application<'aud> {
    resource_manager: ResourceManager, // must drop before window, since window holds Raylib context, otherwise SEGFAULT
    statemanager: StateManager,        // must drop before window to unload shaders safely, otherwise SEGFAULT
    gamestate: Gamestate,
    music_tracks: HashMap<MusicID, Music<'aud>>,  // must drop before window
    sound_effects: HashMap<SoundID, Sound<'aud>>, // must drop before window
    current_music: Option<MusicID>,
    window: Window,
}

impl<'aud> Application<'aud> {
    pub fn new(audio_device: &'aud RaylibAudio) -> Self {
        let mut window = Window::init();
        let mut resource_manager = ResourceManager::new();

        let mut music_tracks = HashMap::new();
        let mut sound_effects = HashMap::new();

        let music_paths = [
            (MusicID::MainMenuTheme, "resources/music/menuscreen2.wav"),
            (MusicID::GameIntroScreen, "resources/music/gameintroscreen2.wav"),
            (MusicID::Level1_Theme, "resources/music/level1_theme.wav"),
        ];

        for &(id, path) in &music_paths {
            if std::path::Path::new(path).exists() {
                match audio_device.new_music(path) {
                    Ok(music) => {
                        music_tracks.insert(id, music);
                    }
                    Err(e) => {
                        eprintln!("Warning: failed to load music track '{path}': {e:?}");
                    }
                }
            } else {
                eprintln!("Warning: music track file '{path}' not found");
            }
        }

        let sfx_paths = [
            (SoundID::FireballShot, "resources/audio/fireball_shot.wav"),
            (SoundID::Fireball_hit_Gobbler, "resources/audio/fireball_hit_gobbler.wav"),
            (SoundID::Fireball_hit_Wall, "resources/audio/fireball_hit_wall.wav"),
            (SoundID::Gobbler_Collision, "resources/audio/gobbler_collision.wav"),
            (SoundID::MenuSelect, "resources/audio/menuselect.wav"),
        ];

        for &(id, path) in &sfx_paths {
            if std::path::Path::new(path).exists() {
                match audio_device.new_sound(path) {
                    Ok(sound) => {
                        sound_effects.insert(id, sound);
                    }
                    Err(e) => {
                        eprintln!("Warning: failed to load SFX '{path}': {e:?}");
                    }
                }
            } else {
                eprintln!("Warning: SFX file '{path}' not found");
            }
        }

        let (rl, thread) = window.get_handles();
        let mut statemanager = StateManager::new(StateTag::Menu);
        statemanager.apply_lifecycles(rl, thread, &mut resource_manager);
        statemanager.init_resources(rl, thread);

        Self {
            resource_manager,
            statemanager,
            gamestate: Gamestate::new(),
            music_tracks,
            sound_effects,
            current_music: None,
            window,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if let Some(id) = self.current_music {
            if let Some(music) = self.music_tracks.get(&id) {
                music.update_stream();
            }
        }

        let mut audio = AudioManager::new(&mut self.current_music, &self.music_tracks, &self.sound_effects);

        self.statemanager.update(self.window.rl(), delta_time, &mut audio);
        self.statemanager.late_update();

        let (rl, thread) = self.window.get_handles();
        self.statemanager.apply_lifecycles(rl, thread, &mut self.resource_manager);
    }

    pub fn draw(&mut self) {
        let resource_manager = &self.resource_manager;

        self.window.render_to_target(|canvas| {
            self.statemanager.draw(canvas, resource_manager);
        });

        let use_4_3 = self.statemanager.current_state_uses_4_3();
        let shader = self.statemanager.get_post_process_shader();
        self.window.draw_to_screen(shader, use_4_3);
    }

    // main game loop
    pub fn run(&mut self) {
        while !self.window.should_close() && !self.statemanager.is_empty() {
            self.update(self.window.get_delta_time());
            self.draw();
        }
    }
}
