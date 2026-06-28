use sola_raylib::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum MusicID {
    MainMenuTheme,
    GameIntroScreen,
    Level1_Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum SoundID {
    FireballShot,
    Fireball_hit_Gobbler,
    Fireball_hit_Wall,
    Gobbler_Collision,
    MenuSelect,
}

pub struct AudioManager<'a, 'aud> {
    current_music: &'a mut Option<MusicID>,
    music_tracks: &'a HashMap<MusicID, Music<'aud>>,
    sound_effects: &'a HashMap<SoundID, Sound<'aud>>,
}

impl<'a, 'aud> AudioManager<'a, 'aud> {
    pub fn new(
        current_music: &'a mut Option<MusicID>,
        music_tracks: &'a HashMap<MusicID, Music<'aud>>,
        sound_effects: &'a HashMap<SoundID, Sound<'aud>>,
    ) -> Self {
        Self {
            current_music,
            music_tracks,
            sound_effects,
        }
    }

    pub fn play_music(&mut self, id: MusicID) {
        if *self.current_music == Some(id) {
            return;
        }

        if let Some(old_id) = *self.current_music {
            if let Some(old_music) = self.music_tracks.get(&old_id) {
                old_music.stop_stream();
            }
        }

        *self.current_music = Some(id);
        if let Some(new_music) = self.music_tracks.get(&id) {
            new_music.play_stream();
        }
    }

    pub fn play_sfx(&self, id: SoundID) {
        if let Some(sound) = self.sound_effects.get(&id) {
            sound.play();
        }
    }
}
