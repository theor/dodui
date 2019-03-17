use specs::prelude::*;
use specs_hierarchy::Hierarchy;

#[derive(Debug, Clone)]
pub struct Parent {
    pub entity: Entity,
}

pub type ParentHierarchy = Hierarchy<Parent>;

impl Component for Parent {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl specs_hierarchy::Parent for Parent {
    fn parent_entity(&self) -> Entity {
        self.entity
    }
}