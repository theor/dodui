use crate::transform::GlobalTransform;
use crate::transform::{Parent, ParentHierarchy};

use specs::prelude::*;
use stretch::layout::Node as LayoutNode;
use stretch::{
    geometry::{Rect, Size},
    number::Number,
    style::*,
};

use crate::manager::*;
use crate::style_system::EElement;

pub struct Dimensions {
    pub display: Display,

    pub position_type: PositionType,
    pub direction: Direction,
    pub flex_direction: FlexDirection,

    pub flex_wrap: FlexWrap,
    pub overflow: Overflow,

    pub align_items: AlignItems,
    pub align_self: AlignSelf,
    pub align_content: AlignContent,

    pub justify_content: JustifyContent,

    pub position: Rect<Dimension>,
    pub margin: Rect<Dimension>,
    pub padding: Rect<Dimension>,
    pub border: Rect<Dimension>,

    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Dimension,

    pub size: Size<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,

    pub aspect_ratio: Number,
}

impl Component for Dimensions {
    type Storage = DenseVecStorage<Self>;
}

impl Dimensions {
    pub fn fill_node(&self, node: &mut stretch::style::Node) {
        node.display = self.display;
        node.position_type = self.position_type;
        node.direction = self.direction;
        node.flex_direction = self.flex_direction;
        node.flex_wrap = self.flex_wrap;
        node.overflow = self.overflow;
        node.align_items = self.align_items;
        node.align_self = self.align_self;
        node.align_content = self.align_content;
        node.justify_content = self.justify_content;
        node.position = self.position;
        node.margin = self.margin;
        node.padding = self.padding;
        node.border = self.border;
        node.flex_grow = self.flex_grow;
        node.flex_shrink = self.flex_shrink;
        node.flex_basis = self.flex_basis;
        node.size = self.size;
        node.min_size = self.min_size;
        node.max_size = self.max_size;
        node.aspect_ratio = self.aspect_ratio;
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            display: Default::default(),

            position_type: Default::default(),
            direction: Default::default(),
            flex_direction: Default::default(),

            flex_wrap: Default::default(),
            overflow: Default::default(),

            align_items: Default::default(),
            align_self: Default::default(),
            align_content: Default::default(),

            justify_content: Default::default(),

            position: Default::default(),
            margin: Default::default(),
            padding: Default::default(),
            border: Default::default(),

            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,

            size: Default::default(),
            min_size: Default::default(),
            max_size: Default::default(),

            aspect_ratio: Default::default(),
        }
    }
}

pub struct LayoutSystem;
impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, crate::manager::ResourceManager>,
        ReadExpect<'a, crate::rendering::Screen>,
        ReadExpect<'a, ParentHierarchy>,
        ReadStorage<'a, EElement>,
        ReadStorage<'a, Parent>,
        ReadStorage<'a, Dimensions>,
        WriteStorage<'a, GlobalTransform>,
        ReadStorage<'a, crate::rendering::Text>,
    );

    fn run(
        &mut self,
        (entities, store, screen, hierarchy, eelements, parents, dimensions, mut globals, text): Self::SystemData,
    ) {
        let mut root = Node {
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            size: Size {
                width: Dimension::Points(screen.size.0 as f32),
                height: Dimension::Points(screen.size.1 as f32),
            },
            ..Default::default()
        };

        for (entity, _, _) in (&*entities, &eelements, !&parents).join() {
            let branch = Self::make(&hierarchy, entity, &dimensions, &text, &store);

            root.children.push(branch);
        }

        // println!("layout {:?}", root);
        let layout = stretch::compute(&root, Size::undefined()).unwrap();

        let mut i = 0;
        for (entity, _, _) in (&*entities, &eelements, !&parents).join() {
            let n = &layout.children[i];
            Self::apply(&hierarchy, entity, &mut globals, n);
            i = i + 1;
        }

        // println!("computed {:#?}", layout);
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
            // println!("Layout {:?}: {:?}", e, t);
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
        store: &ReadExpect<crate::manager::ResourceManager>,
    ) -> Node {
        let mut n: Node = Default::default();
        if let Some(dimensions) = dimensions.get(e.clone()) {
            dimensions.fill_node(&mut n);
        }

        if let Some(text) = text.get(e) {
            let key = SimpleKey::Path(("style/NotoSans-Regular.ttf").into());
            let font = store.get::<crate::layout::BitmapFont>(&key).unwrap();
            let measured = font.borrow().measure(&text.text);
            let e = e.clone();
            n.measure = Some(Box::new(move |s| { 
                // println!("measure input {:?} {:?}", e, s);
                Ok(measured)
            }));
        }

        for c in hierarchy.children(e) {
            n.children
                .push(Self::make(hierarchy, c.clone(), dimensions, text, store));
        }

        n
    }
}
