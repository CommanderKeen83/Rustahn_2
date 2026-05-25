use sola_raylib::prelude::*;

const GAME_WIDTH: i32 = 1280;
const GAME_HEIGHT: i32 = 720;

pub fn main() {
    let (mut rl, thread) = sola_raylib::init()
        .size(GAME_WIDTH, GAME_HEIGHT)
        .title("Rabahn")
        .undecorated()   // <-- Add this so you can actually test the scaling!
        .build();

    let monitor = sola_raylib::core::window::get_current_monitor();
    let mon_w = sola_raylib::core::window::get_monitor_width(monitor);
    let mon_h = sola_raylib::core::window::get_monitor_height(monitor);
    #[cfg(not(target_os = "macos"))]{
        rl.set_window_size(mon_w, mon_h);
        rl.set_window_position(0, 0);
    }
    #[cfg(target_os = "macos")]{
        rl.toggle_fullscreen();
    }

    // Read back what raylib actually sees after fullscreen/resize is applied
    let screen_w = rl.get_screen_width() as f32;
    let screen_h = rl.get_screen_height() as f32;

    let game_w = GAME_WIDTH as f32;
    let game_h = GAME_HEIGHT as f32;
    let scale = (screen_w / game_w).min(screen_h / game_h);


    let mut target = rl
        .load_render_texture(&thread, GAME_WIDTH as u32, GAME_HEIGHT as u32)
        .expect("could not create target in main.rs");

    // This stays outside the loop. Notice the NEGATIVE height!
    let source_rec = Rectangle::new(
        0.0,
        0.0,
        GAME_WIDTH as f32,
        -(GAME_HEIGHT as f32), 
    );

        let game_w = GAME_WIDTH as f32;
        let game_h = GAME_HEIGHT as f32;
        
        let scale = (mon_w as f32 / game_w).min(mon_h as f32 / game_h);
    while !rl.window_should_close() {
        // 1. Calculate scaling INSIDE the loop so it updates if the window resizes


        let dest_rec = Rectangle::new(
            (screen_w as f32 - (game_w * scale)) * 0.5,
            (screen_h as f32 - (game_h * scale)) * 0.5,
            game_w * scale,
            game_h * scale,
        );

        // 2. FIRST PASS: Draw your game to the RenderTexture
        {
            // We use a block { } to ensure d_texture goes out of scope 
            // and safely finishes the texture pass before we draw to the main screen.
            // d_texture basically draws to the RenderTexture (=> our variable target!) 
            // and d.draw_texture_pro draws that target to the screen.
            let mut d_texture = rl.begin_texture_mode(&thread, &mut target);
            d_texture.clear_background(Color::LIGHTBLUE);
            d_texture.draw_text("hello, world", 12, 12, 20, Color::BLACK);
        }

        // 3. SECOND PASS: Draw the RenderTexture to the actual window
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK); // This becomes the color of your letterbox borders
        d.draw_texture_pro(
            &target, 
            source_rec, 
            dest_rec, 
            Vector2::new(0.0, 0.0), 
            0.0, 
            Color::WHITE
        );
    }
}