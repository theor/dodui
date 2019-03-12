use specs::prelude::*;



pub struct StyleSystem;
impl<'a> System<'a> for StyleSystem {
    type SystemData = (
        ReadStorage<'a, Pseudo>,
        ReadStorage<'a, StyleBackground>,
        WriteStorage<'a, crate::rendering::Material>,
    );

    #[allow(dead_code)]
    fn run(&mut self, (pseudo, bg, mut mat): Self::SystemData) {
        for (pseudo, bg, mut mat) in (pseudo.maybe(), &bg, &mut mat).join() {
            mat.color = if pseudo.map_or(false, |v| v.hover) {
                bg.color
            } else {
                bg.color / 2
            };
        }
    }
}

impl StyleSystem {
    pub fn new() -> Self {
        Self {}
    }
}


#[derive(Debug)]
pub struct StyleBackground {
    pub color: cgmath::Vector4<u8>,
}

impl Component for StyleBackground {
    type Storage = DenseVecStorage<Self>;
}

impl StyleBackground {
    pub fn from_color(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            color: cgmath::Vector4::new(r, g, b, a),
        }
    }
}

#[derive(Debug, Default)]
pub struct Pseudo {
    pub hover: bool,
}

impl Component for Pseudo {
    type Storage = DenseVecStorage<Self>;
}
