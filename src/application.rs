use crate::prelude::*;

#[allow(dead_code)]
pub struct Application{
    window: Window,
    resource_manager: ResourceManager,
    gamestate: Gamestate,
    statemanager: StateManager,
}

impl Application{
    pub fn new() -> Self{
        let mut window = Window::init();
        let mut resource_manager = ResourceManager::new();

        let (rl, thread) = window.get_handles();
        if let Err(e) = resource_manager.load_texture(
            rl,
            thread,
            SpriteID::MenuBackground,
            "resources/graphics/main_menu.png",
        ) {
            eprintln!("Failed to load menu background: {}", e);
        }

        Self{
            window,
            resource_manager,
            gamestate: Gamestate::new(),
            statemanager: StateManager::new(StateTag::Menu),
        }
    }
    pub fn update(&mut self, delta_time: f32){
        self.statemanager.update(delta_time);
        self.statemanager.late_update();
    }
    pub fn draw(&mut self){
        let resource_manager = &self.resource_manager;
        let statemanager = &self.statemanager;
        self.window.draw(|canvas|{
            statemanager.draw(canvas, resource_manager);
        });
    }
    // main game loop
    pub fn run(&mut self){
        while !self.window.should_close() {
            self.update(self.window.get_delta_time());
            self.draw();
        }
    }
}