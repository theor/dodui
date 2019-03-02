use specs::prelude::*;

use cgmath::Vector4;

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub color: Vector4<f32>,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            color: Vector4::new(1.0, 1.0, 1.0, 1.0)
        }
    }
}

impl Material {
    pub fn from_color(r:f32, g:f32, b:f32, a:f32) -> Self {        
        Material {
            color: Vector4::new(r,g,b,a)
        }
    }
}

impl Component for Material {
    type Storage = VecStorage<Self>;
}