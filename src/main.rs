use rustahn::prelude::*;
use sola_raylib::prelude::RaylibAudio;

pub fn main() {
    let audio_device = RaylibAudio::init_audio_device().expect("Failed to initialize audio device");
    let mut app = Application::new(&audio_device);
    app.run();
}