use specs::prelude::*;

use cgmath::Vector4;

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub color: Vector4<u8>,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            color: Vector4::new(255, 255, 255, 255)
        }
    }
}

impl Material {
    pub fn from_color(r:u8, g:u8, b:u8, a:u8) -> Self {        
        Self {
            color: Vector4::new(r,g,b,a)
        }
    }
}

impl Component for Material {
    type Storage = VecStorage<Self>;
}