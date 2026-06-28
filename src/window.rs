use sola_raylib::prelude::*;


pub const GAME_WIDTH: i32 = 1280;
pub const GAME_HEIGHT: i32 = 720;


pub struct Window{
    thread: RaylibThread,
    source_rec: Rectangle,
    dest_rec: Rectangle,
    dest_rec_4_3: Rectangle,
    target: RenderTexture2D, // everything gets drawn to this virtual Canvas (1280x720). It gets scaled to the size of the window (GAME_WIDTH, GAME_HEIGHT).
    rl: RaylibHandle, // Drops happen from top to bottom, so RaylibHandle must always be dropped last, otherwise the lost Raylib Context will cause segfaults!
}

impl Window{
pub fn init() -> Self {
    let (mut rl, thread) = sola_raylib::init()
        .size(GAME_WIDTH, GAME_HEIGHT)
        .title("Rabahn")
        .undecorated()
        .build();

    let target = rl
        .load_render_texture(&thread, GAME_WIDTH as u32, GAME_HEIGHT as u32)
        .expect("could not create target in main.rs");

    #[cfg(not(target_os = "macos"))]
    let (screen_w, screen_h) = {
        let monitor = sola_raylib::core::window::get_current_monitor();
        let mon_w = sola_raylib::core::window::get_monitor_width(monitor);
        let mon_h = sola_raylib::core::window::get_monitor_height(monitor);
        rl.set_window_size(mon_w, mon_h);
        rl.set_window_position(0, 0);
        (mon_w as f32, mon_h as f32)
    };

    #[cfg(target_os = "macos")]
    let (screen_w, screen_h) = {
        rl.toggle_fullscreen();
        (rl.get_render_width() as f32, rl.get_render_height() as f32)
    };
    
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

    let scale_4_3 = (screen_w / 4.0).min(screen_h / 3.0);
    let dest_w_4_3 = 4.0 * scale_4_3;
    let dest_h_4_3 = 3.0 * scale_4_3;
    let dest_rec_4_3 = Rectangle::new(
        (screen_w - dest_w_4_3) * 0.5,
        (screen_h - dest_h_4_3) * 0.5,
        dest_w_4_3,
        dest_h_4_3,
    );

        Self { rl, thread, source_rec, dest_rec, dest_rec_4_3, target }

    } // end of Window::init()
    pub fn should_close(&self) -> bool{
        self.rl.window_should_close()
    }

    pub fn get_delta_time(&self) -> f32{
        self.rl.get_frame_time()
    }

    pub fn rl(&self) -> &RaylibHandle {
        &self.rl
    }

    pub fn rl_mut(&mut self) -> &mut RaylibHandle {
        &mut self.rl
    }

    pub fn thread(&self) -> &RaylibThread {
        &self.thread
    }

    pub fn get_handles(&mut self) -> (&mut RaylibHandle, &RaylibThread) {
        (&mut self.rl, &self.thread)
    }

    pub fn render_to_target(&mut self, mut draw_fn: impl FnMut(&mut RaylibTextureMode<RaylibHandle>)) {
        let mut d_texture = self.rl.begin_texture_mode(&self.thread, &mut self.target);
        d_texture.clear_background(Color::LIGHTBLUE);
        d_texture.draw_text("hello, world", 12, 12, 20, Color::BLACK);
        draw_fn(&mut d_texture);
    }

    pub fn draw_to_screen(&mut self, shader: Option<&mut Shader>, use_4_3: bool) {
        let dest = if use_4_3 { self.dest_rec_4_3 } else { self.dest_rec };
        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::BLACK);
        if let Some(sh) = shader {
            let mut shader_mode = d.begin_shader_mode(sh);
            shader_mode.draw_texture_pro(
                &self.target,
                self.source_rec,
                dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
        } else {
            d.draw_texture_pro(
                &self.target,
                self.source_rec,
                dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
        }
    }

}