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
        let mut statemanager = StateManager::new(StateTag::Menu);
        statemanager.apply_lifecycles(rl, thread, &mut resource_manager);
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
        
        let (rl, thread) = self.window.get_handles();
        self.statemanager.apply_lifecycles(rl, thread, &mut self.resource_manager);
    }
    pub fn draw(&mut self){
        let resource_manager = &self.resource_manager;
        
        self.window.render_to_target(|canvas|{
            self.statemanager.draw(canvas, resource_manager);
        });
        
        let shader = self.statemanager.get_post_process_shader();
        self.window.draw_to_screen(shader);
    }
    // main game loop
    pub fn run(&mut self){
        while !self.window.should_close() && !self.statemanager.is_empty() {
            self.update(self.window.get_delta_time());
            self.draw();
        }
    }
}