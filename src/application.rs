use crate::prelude::*;

#[allow(dead_code)]
pub struct Application{
    resource_manager: ResourceManager, // must drop before window, since window holds Raylib context, otherwise SEGFAULT
    statemanager: StateManager,        // must drop before window to unload shaders safely, otherwise SEGFAULT
    gamestate: Gamestate,
    window: Window,
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

        let mut statemanager = StateManager::new(StateTag::Menu);
        statemanager.init_resources(rl, thread);

        Self{
            resource_manager,
            statemanager,
            gamestate: Gamestate::new(),
            window,
        }
    }
    pub fn update(&mut self, delta_time: f32){
        self.statemanager.update(self.window.rl(), delta_time);
        self.statemanager.late_update();
    }
    pub fn draw(&mut self){
        let resource_manager = &self.resource_manager;
        let shader_ptr = self.statemanager.get_post_process_shader();
        let shader = shader_ptr.map(|p| unsafe { &mut *p });
        
        let statemanager = &mut self.statemanager;
        self.window.draw(shader, |canvas|{
            statemanager.draw(canvas, resource_manager);
        });
    }
    // main game loop
    pub fn run(&mut self){
        while !self.window.should_close() && !self.statemanager.is_empty() {
            self.update(self.window.get_delta_time());
            self.draw();
        }
    }
}