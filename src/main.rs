use sola_raylib::prelude::*;

use rustahn::prelude::*;



pub fn main(){
    let(mut rl, thread) = sola_raylib::init()
    .size(800,600)
    .title("Rabahn")
    .build();
    physics::update(rl.get_frame_time());
    behaviour::update(rl.get_frame_time());

    while !rl.window_should_close(){

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(sola_raylib::color::Color::LIGHTBLUE);
        d.draw_text("hello, world", 12, 12, 20, sola_raylib::color::Color::BLACK);
    }
}