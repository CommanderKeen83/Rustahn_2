use std::collections::HashMap;
use sola_raylib::prelude::*;

#[allow(dead_code)]
pub struct ResourceManager{
    textures: HashMap<TextureID, Texture2D>,
}
impl ResourceManager{
    
}


pub enum TextureID{
    MenuBackground,
}