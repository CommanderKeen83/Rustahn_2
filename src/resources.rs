use std::collections::HashMap;
use sola_raylib::prelude::*;
use crate::prelude::SpriteID;

pub struct ResourceManager {
    textures: HashMap<SpriteID, Texture2D>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        id: SpriteID,
        path: &str,
    ) -> Result<(), String> {
        let texture = rl.load_texture(thread, path).map_err(|e| format!("{:?}", e))?;
        self.textures.insert(id, texture);
        Ok(())
    }

    pub fn get_texture(&self, id: &SpriteID) -> Option<&Texture2D> {
        self.textures.get(id)
    }
}