
use specs::prelude::*;

use cgmath::{Matrix4, Point2};

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Point2<f32>,
    pub size: Point2<f32>,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Transform {
            position: (x, y).into(),
            size: (100.0, 100.0).into(),
        }
    }

    pub fn with_size(self, w:f32, h:f32) -> Self {
        Self { size: (w,h).into(), ..self }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        cgmath::Matrix4::from_translation([self.position.x, self.position.y, 0.0f32].into())
    }
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}