use sola_raylib::prelude::*;

mod systems;



pub fn main(){
    let(mut rl, thread) = sola_raylib::init()
    .size(800,600)
    .title("Rabahn")
    .build();

    while !rl.window_should_close(){
        systems::physics::update(rl.get_frame_time());
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(sola_raylib::color::Color::LIGHTBLUE);
        d.draw_text("hello, world", 12, 12, 20, sola_raylib::color::Color::BLACK);
    }
}