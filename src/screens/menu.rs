use crate::prelude::*;
use sola_raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    NewGame,
    ContinueGame,
    Options,
    Quit,
}

struct MenuItem {
    name: &'static str,
    action: MenuAction,
}

const MENU_ITEMS: [MenuItem; 4] = [
    MenuItem {
        name: "New Game",
        action: MenuAction::NewGame,
    },
    MenuItem {
        name: "Continue Game",
        action: MenuAction::ContinueGame,
    },
    MenuItem {
        name: "Options",
        action: MenuAction::Options,
    },
    MenuItem {
        name: "Quit",
        action: MenuAction::Quit,
    },
];

pub struct Menu {
    selected: usize,
    flicker: f32,
    has_save: bool,
    crt_shader: Option<Shader>,
    time_loc: i32,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            selected: 0,
            flicker: 0.0,
            has_save: false,
            crt_shader: None,
            time_loc: 0,
        }
    }

    fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn select_next(&mut self) {
        if self.selected < MENU_ITEMS.len() - 1 {
            self.selected += 1;
        }
    }
}

impl Screen for Menu {
    fn on_enter(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, resources: &mut ResourceManager) {
        if let Err(e) = resources.load_texture(rl, thread, SpriteID::MenuBackground, "resources/graphics/main_menu.png") {
            eprintln!("Failed to load menu background: {}", e);
        }

        let shader = rl.load_shader(thread, None, Some("src/shaders/crt_shader.fs"));
        self.time_loc = shader.get_shader_location("time");
        self.crt_shader = Some(shader);
    }

    fn update(&mut self, ctx: &mut Context) -> Option<Transition> {
        ctx.audio.play_music(MusicID::MainMenuTheme);
        self.flicker += ctx.dt;
        self.has_save = ctx.has_state(StateTag::Game);

        if let Some(ref mut shader) = self.crt_shader {
            shader.set_shader_value(self.time_loc, self.flicker);
        }

        if ctx.rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            ctx.audio.play_sfx(SoundID::MenuSelect);
            let selected_action = MENU_ITEMS[self.selected].action;
            match selected_action {
                MenuAction::NewGame => {
                    let transitions = vec![
                        Transition::Remove(StateTag::Game),
                        Transition::Remove(StateTag::GameIntro),
                        Transition::Switch(StateTag::GameIntro),
                    ];
                    return Some(Transition::Multiple(transitions));
                }
                MenuAction::ContinueGame => {
                    if self.has_save {
                        return Some(Transition::Switch(StateTag::Game));
                    }
                }
                MenuAction::Options => {
                    return Some(Transition::Push(State::from_tag(StateTag::Options)));
                }
                MenuAction::Quit => {
                    return Some(Transition::Clear);
                }
            }
        } else if ctx.rl.is_key_pressed(KeyboardKey::KEY_UP) {
            ctx.audio.play_sfx(SoundID::MenuSelect);
            self.select_previous();
            if MENU_ITEMS[self.selected].action == MenuAction::ContinueGame && !self.has_save {
                self.select_previous();
            }
        } else if ctx.rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            ctx.audio.play_sfx(SoundID::MenuSelect);
            self.select_next();
            if MENU_ITEMS[self.selected].action == MenuAction::ContinueGame && !self.has_save {
                self.select_next();
            }
        } else if ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            ctx.audio.play_sfx(SoundID::MenuSelect);
            if self.has_save {
                return Some(Transition::Switch(StateTag::Game));
            } else {
                return Some(Transition::Clear);
            }
        }
        None
    }

    fn draw(&mut self, canvas: &mut RaylibTextureMode<RaylibHandle>, resources: &ResourceManager) {
        draw_menu_items(canvas, self.selected, self.flicker, self.has_save, resources);
    }

    fn wants_4_3(&self) -> bool {
        true
    }

    fn get_post_process_shader(&mut self) -> Option<&mut Shader> {
        self.crt_shader.as_mut()
    }
}

fn measure_text_raw(text: &str, font_size: i32) -> i32 {
    let c_text = std::ffi::CString::new(text).unwrap();
    unsafe { sola_raylib::ffi::MeasureText(c_text.as_ptr(), font_size) }
}

fn draw_menu_items(
    canvas: &mut impl RaylibDraw,
    selected: usize,
    flicker: f32,
    has_save: bool,
    resources: &ResourceManager,
) {
    if let Some(tex) = resources.get_texture(&SpriteID::MenuBackground) {
        let source = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);
        let dest = Rectangle::new(0.0, 0.0, GAME_WIDTH as f32, GAME_HEIGHT as f32);
        canvas.draw_texture_pro(tex, source, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
    }

    let start_y = 440;
    let spacing = 48;
    let item_fontsize = 32;
    let margin_text_to_right_wall = 64;

    for (i, item) in MENU_ITEMS.iter().enumerate() {
        let y = start_y + (i as i32) * spacing;
        let is_selected = i == selected;
        let is_disabled = item.action == MenuAction::ContinueGame && !has_save;

        let col = if is_disabled {
            Color::new(120, 110, 90, 140)
        } else if is_selected {
            Color::new(255, 230, 120, 255)
        } else {
            Color::new(220, 210, 185, 210)
        };

        let lw = measure_text_raw(item.name, item_fontsize);

        if is_selected && !is_disabled {
            canvas.draw_rectangle(
                GAME_WIDTH - margin_text_to_right_wall - lw - 60,
                y - 6,
                600,
                item_fontsize + 12,
                Color::new(255, 210, 80, 18),
            );
            canvas.draw_rectangle(
                GAME_WIDTH - margin_text_to_right_wall - lw - 60,
                y - 6,
                3,
                item_fontsize + 12,
                Color::new(255, 210, 80, 200),
            );
        }

        canvas.draw_text(item.name, GAME_WIDTH - margin_text_to_right_wall - lw, y, item_fontsize, col);

        if is_selected && !is_disabled {
            let flicker_alpha = (((flicker * 6.0).sin() * 0.5 + 0.5) * 255.0) as u8;
            canvas.draw_text(
                ">>",
                GAME_WIDTH - margin_text_to_right_wall - lw - 50,
                y,
                item_fontsize,
                Color::new(255, 210, 80, flicker_alpha),
            );
        }
    }
}
