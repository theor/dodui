
use specs::prelude::*;
use specs_hierarchy::HierarchyEvent;
use hibitset::BitSet;

use cgmath::{Deg, Matrix4, Point2, Point3, Vector2, Vector3};

#[derive(Debug)]
pub struct Vel(pub f32);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

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

use specs_hierarchy::{Hierarchy, HierarchySystem};

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

pub struct TransformSystem {
    local_modified: BitSet,
    global_modified: BitSet,

    locals_events_id: Option<ReaderId<ComponentEvent>>,

    parent_events_id: Option<ReaderId<HierarchyEvent>>,

    scratch: Vec<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        TransformSystem {
            locals_events_id: None,
            parent_events_id: None,
            local_modified: BitSet::default(),
            global_modified: BitSet::default(),
            scratch: Vec::new(),
        }
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, ParentHierarchy>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Parent>,
        WriteStorage<'a, GlobalTransform>,
    );
    fn run(&mut self, (entities, hierarchy, locals, parents, mut globals): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");

        self.scratch.clear();
        self.scratch
            .extend((&*entities, &locals, !&globals).join().map(|d| d.0));
        for entity in &self.scratch {
            globals
                .insert(*entity, GlobalTransform::default())
                .expect("unreachable");
        }

        self.local_modified.clear();
        self.global_modified.clear();

        locals
            .channel()
            .read(
                self.locals_events_id.as_mut().expect(
                    "`TransformSystem::setup` was not called before `TransformSystem::run`",
                ),
            )
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.local_modified.add(*id);
                }
                ComponentEvent::Removed(_id) => {}
            });

        for event in hierarchy.changed().read(
            self.parent_events_id
                .as_mut()
                .expect("`TransformSystem::setup` was not called before `TransformSystem::run`"),
        ) {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    // Sometimes the user may have already deleted the entity.
                    // This is fine, so we'll ignore any errors this may give
                    // since it can only fail due to the entity already being dead.
                    let _ = entities.delete(entity);
                }
                HierarchyEvent::Modified(entity) => {
                    self.local_modified.add(entity.id());
                }
            }
        }

        // Compute transforms without parents.
        for (entity, _, local, global, _) in (
            &*entities,
            &self.local_modified,
            &locals,
            &mut globals,
            !&parents,
        )
            .join()
        {
            self.global_modified.add(entity.id());
            global.0 = local.matrix();
            debug_assert!(
                global.is_finite(),
                format!("Entity {:?} had a non-finite `Transform`", entity)
            );
        }

        // Compute transforms with parents.
        for entity in hierarchy.all() {
            let self_dirty = self.local_modified.contains(entity.id());
            if let (Some(parent), Some(local)) = (parents.get(*entity), locals.get(*entity)) {
                let parent_dirty = self.global_modified.contains(parent.entity.id());
                if parent_dirty || self_dirty {
                    let combined_transform = if let Some(parent_global) = globals.get(parent.entity)
                    {
                        (parent_global.0 * local.matrix())
                    } else {
                        local.matrix()
                    };

                    if let Some(global) = globals.get_mut(*entity) {
                        self.global_modified.add(entity.id());
                        global.0 = combined_transform;
                    }
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use specs::prelude::SystemData;
        Self::SystemData::setup(res);
        let mut hierarchy = res.fetch_mut::<ParentHierarchy>();
        let mut locals = WriteStorage::<Transform>::fetch(res);
        self.parent_events_id = Some(hierarchy.track());
        self.locals_events_id = Some(locals.register_reader());
    }
}
