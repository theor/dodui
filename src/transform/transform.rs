
use specs::prelude::*;
use specs_hierarchy::HierarchyEvent;
use hibitset::BitSet;

use cgmath::{Deg, Matrix4, Point2, Point3, Vector2, Vector3};

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Point2<f32>,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Transform {
            position: (x, y).into(),
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        cgmath::Matrix4::from_translation([self.position.x, self.position.y, 0.0f32].into())
    }
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}