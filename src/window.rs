use sola_raylib::prelude::*;


const GAME_WIDTH: i32 = 1280;
const GAME_HEIGHT: i32 = 720;


pub struct Window{
    thread: RaylibThread,
    source_rec: Rectangle,
    dest_rec: Rectangle,
    target: RenderTexture2D,
    rl: RaylibHandle, // Drops happen top to bottom, so RaylibHandle must always be dropped last!
}

impl Window{
pub fn init() -> Self {
    let (mut rl, thread) = sola_raylib::init()
        .size(GAME_WIDTH, GAME_HEIGHT)
        .title("Rabahn")
        .undecorated()
        .build();

    let mut screen_w = rl.get_render_width() as f32;
    let mut screen_h = rl.get_render_height() as f32;

    #[cfg(not(target_os = "macos"))]
    {
        let monitor = sola_raylib::core::window::get_current_monitor();
        let mon_w = sola_raylib::core::window::get_monitor_width(monitor);
        let mon_h = sola_raylib::core::window::get_monitor_height(monitor);
        rl.set_window_size(mon_w, mon_h);
        rl.set_window_position(0, 0);
        screen_w = mon_w as f32;
        screen_h = mon_h as f32;
    }

    #[cfg(target_os = "macos")]
    {
        rl.toggle_fullscreen();
        screen_w = rl.get_render_width() as f32;
        screen_h = rl.get_render_height() as f32;
    }
    
    let game_w = GAME_WIDTH as f32;
    let game_h = GAME_HEIGHT as f32;
    let scale = (screen_w / game_w).min(screen_h / game_h);

    let source_rec = Rectangle::new(
        0.0,
        0.0,
        GAME_WIDTH as f32,
        -(GAME_HEIGHT as f32),
    );
    
    let dest_rec = Rectangle::new(
        (screen_w - (game_w * scale)) * 0.5,
        (screen_h - (game_h * scale)) * 0.5,
        game_w * scale,
        game_h * scale,
    );

    let target = rl
        .load_render_texture(&thread, GAME_WIDTH as u32, GAME_HEIGHT as u32)
        .expect("could not create target in main.rs");

        Self { rl, thread, source_rec, dest_rec, target }

    }
    pub fn should_close(&self) -> bool{
        self.rl.window_should_close()
    }

    pub fn get_delta_time(&self) -> f32{
        self.rl.get_frame_time()
    }

    pub fn draw(&mut self, mut draw_fn: impl FnMut(&mut RaylibTextureMode<RaylibHandle>)) {
    {
        let mut d_texture = self.rl.begin_texture_mode(&self.thread, &mut self.target);
        d_texture.clear_background(Color::LIGHTBLUE);
        d_texture.draw_text("hello, world", 12, 12, 20, Color::BLACK);
        draw_fn(&mut d_texture);
    }
    let mut d = self.rl.begin_drawing(&self.thread);
    d.clear_background(Color::BLACK);
    d.draw_texture_pro(
        &self.target,
        self.source_rec,
        self.dest_rec,
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );
}

}