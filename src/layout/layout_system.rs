use crate::transform::{GlobalTransform, Transform};
use crate::transform::{Parent, ParentHierarchy};

use specs::prelude::*;
use stretch::geometry::{Rect, Size};
use stretch::layout::Node as LayoutNode;
use stretch::style::*;

use crate::manager::*;
use crate::rendering::Text;
use specs::prelude::*;

// use hashbrown::HashMap;

pub struct Dimensions {
    pub size: stretch::geometry::Size<stretch::style::Dimension>,
}

impl Component for Dimensions {
    type Storage = DenseVecStorage<Self>;
}

pub struct LayoutSystem;
impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, crate::manager::ResourceManager>,
        ReadExpect<'a, crate::rendering::Screen>,
        ReadExpect<'a, ParentHierarchy>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Parent>,
        ReadStorage<'a, Dimensions>,
        WriteStorage<'a, GlobalTransform>,
        ReadStorage<'a, crate::rendering::Text>,
    );

    fn run(
        &mut self,
        (entities, store, screen, hierarchy, locals, parents, dimensions, mut globals, text): Self::SystemData,
    ) {
        let mut root = Node {
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            size: Size {
                width: Dimension::Points(screen.size.0 as f32),
                height: Dimension::Points(screen.size.1 as f32),
            },
            ..Default::default()
        };

        let key = SimpleKey::Path(("style/NotoSans-Regular.ttf").into());

        let font = store.get::<crate::layout::BitmapFont>(&key);
        let font = match font {
            Ok(ref font) => font,
            _ => return,
        };

        for (entity, /*_,*/ local, _) in (
            &*entities, // &self.local_modified,
            &locals, !&parents,
        )
            .join()
        {
            let branch = Self::make(&hierarchy, entity, &dimensions, &text);

            root.children.push(branch);
            // self.global_modified.add(entity.id());
            // global.0 = local.matrix();
        }

        let layout = stretch::compute(&root, Size::undefined()).unwrap();

        let mut i = 0;
        for (entity, /*_,*/ _local, _) in (
            &*entities, // &self.local_modified,
            &locals, !&parents,
        )
            .join()
        {
            let n = &layout.children[i];
            Self::apply(&hierarchy, entity, &mut globals, n);
            i = i + 1;
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
            t.0 = cgmath::Matrix4::from_translation(
                [node.location.x, node.location.y, 0.0f32].into(),
            );
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
        dimensions: &ReadStorage<'_, Dimensions>,
        text: &ReadStorage<'_, crate::rendering::Text>,
    ) -> Node {
        use stretch::style::Dimension;
        let size = {
            dimensions.get(e.clone()).map_or(
                Size {
                    width: Dimension::Auto,
                    height: Dimension::Auto,
                },
                |d| d.size,
            )
        };
        let mut n = Node {
            flex_grow: 1.0,

            size: size,
            padding: Rect {
                start: Dimension::Points(10.0),
                end: Dimension::Points(10.0),
                top: Dimension::Points(10.0),
                bottom: Dimension::Points(10.0),
            },
            ..Default::default()
        };

        for c in hierarchy.children(e) {
            n.children
                .push(Self::make(hierarchy, c.clone(), dimensions, text));
        }

        n
    }
}
