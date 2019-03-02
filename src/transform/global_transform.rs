
use specs::prelude::*;
use specs_hierarchy::HierarchyEvent;
use hibitset::BitSet;

use cgmath::{Deg, Matrix4, Point2, Point3, Vector2, Vector3};

#[derive(Debug)]
pub struct GlobalTransform(pub Matrix4<f32>);

impl Component for GlobalTransform  {
    type Storage = VecStorage<Self>;
}

impl GlobalTransform {
    /// Checks whether each `f32` of the `GlobalTransform` is finite (not NaN or inf).
    pub fn is_finite(&self) -> bool {
        AsRef::<[f32;16]>::as_ref(&self.0).iter().all(|f| f32::is_finite(*f))
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
         GlobalTransform(cgmath::SquareMatrix::identity())
    }
}