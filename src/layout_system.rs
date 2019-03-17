use crate::transform::*;
use crate::transform::{Parent, ParentHierarchy};
use specs::prelude::*;
use stretch::geometry::{Rect, Size};
use stretch::style::*;
use stretch::layout::Node as LayoutNode;

pub struct LayoutSystem;
impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, ParentHierarchy>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Parent>,
        WriteStorage<'a, GlobalTransform>,
    );

    fn run(&mut self, (entities, hierarchy, locals, parents, mut globals): Self::SystemData) {
        let mut root = Node {
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            size: Size {
                width: Dimension::Points(1024.0),
                height: Dimension::Points(768.0),
            },
            ..Default::default()
        };

        for (entity, /*_,*/ local, _) in (
            &*entities,
            // &self.local_modified,
            &locals,
            !&parents,
        )
            .join()
        {
            let branch = Self::make(&hierarchy, entity, local, &locals);

            root.children.push(branch);
            // self.global_modified.add(entity.id());
            // global.0 = local.matrix();
        }

        let layout = stretch::compute(&root, Size::undefined()).unwrap();

        let mut i = 0;
        for (entity, /*_,*/ local, _) in (
            &*entities,
            // &self.local_modified,
            &locals,
            !&parents,
        )
            .join()
        {
            let n = &layout.children[i];
            Self::apply(&hierarchy, entity, &mut globals, n);
            i = i+1;
        }

        // println!("{:#?}", layout);
    }
}
impl LayoutSystem {
    fn apply(
        hierarchy: &ParentHierarchy,
        e: Entity,
        mut globals: &mut WriteStorage<'_, GlobalTransform>,
        node: &LayoutNode,
    ) {
        let mut i = 0;

        {
            let t: &mut GlobalTransform = globals.get_mut(e).unwrap();
            t.0 = cgmath::Matrix4::from_translation([node.location.x, node.location.y, 0.0f32].into());
            t.1 = (node.size.width, node.size.height);
        }
        for c in hierarchy.children(e) {
            Self::apply(&hierarchy, c.clone(), &mut globals, &node.children[i]);
            i += 1;
        }
    }

    fn make(
        hierarchy: &ParentHierarchy,
        e: Entity,
        t: &Transform,
        locals: &ReadStorage<'_, Transform>,
    ) -> Node {
        let mut n = Node {
            flex_grow: 1.0,
            size: Size {
                width: Dimension::Points(t.size.x),
                height: Dimension::Points(t.size.y),
            },
            padding: Rect {
                start: Dimension::Points(10.0),
                end: Dimension::Points(10.0),
                top: Dimension::Points(10.0),
                bottom: Dimension::Points(10.0),
            },
            ..Default::default()
        };

        for c in hierarchy.children(e) {
            n.children.push(Self::make(hierarchy, c.clone(), &locals.get(c.clone()).unwrap(), locals));
        }

        n
    }
}
