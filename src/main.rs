// Copyright 2014 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate matches;

extern crate cgmath;
#[macro_use]
extern crate gfx;

#[allow(unused_imports)]
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate winit;
// extern crate gfx_window_glfw;

#[cfg(target_os = "windows")]
extern crate gfx_device_dx11;
#[cfg(target_os = "windows")]
extern crate gfx_window_dxgi;

#[cfg(feature = "metal")]
extern crate gfx_device_metal;
#[cfg(feature = "metal")]
extern crate gfx_window_metal;

#[cfg(feature = "vulkan")]
extern crate gfx_device_vulkan;
#[cfg(feature = "vulkan")]
extern crate gfx_window_vulkan;

use specs::prelude::*;

mod gfx_app;
mod shade;

mod manager;

mod rendering;
mod transform;
use transform::*;

mod layout_system;
mod color;
mod style_system;
mod styling;
use style_system::*;

#[allow(dead_code)]
struct SysA;
impl<'a> System<'a> for SysA {
    type SystemData = (WriteStorage<'a, Transform>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.position.x += vel.0;
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum EventType {
    Pressed,
    Released,
}
#[derive(Debug)]
pub struct Event {
    pub target: Entity,
    pub event_type: EventType,
}

impl Component for Event {
    type Storage = HashMapStorage<Self>;
}

type Callback = Box<dyn Fn(Entity) + Send>;
pub struct Events {
    pub map: std::sync::Mutex<std::collections::HashMap<Entity, Callback>>,
}

impl Default for Events {
    fn default() -> Self {
        Events {
            map: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl Events {
    pub fn register(&mut self, e: Entity, c: Callback) {
        self.map.lock().unwrap().insert(e, c);
    }
    pub fn invoke(&self, e: Entity) {
        if let Some(cb) = self.map.lock().unwrap().get(&e) {
            let cb2 = cb.clone();
            cb2(e.clone());
        }
    }
}

struct ConsumeEventsSystem;
impl<'a> System<'a> for ConsumeEventsSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, Event>);

    fn run(&mut self, (entities, event): Self::SystemData) {
        for (_e, event) in (&entities, &event).join() {
            println!("Event {:?}", event);
        }
    }
}

struct CleanEventsSystem;
impl<'a> System<'a> for CleanEventsSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, Event>);

    fn run(&mut self, (entities, event): Self::SystemData) {
        for (e, _event) in (&entities, &event).join() {
            entities.delete(e).unwrap();
        }
    }
}

struct PickSystem;
impl<'a> System<'a> for PickSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Pseudo>,
        WriteStorage<'a, Event>,
        Read<'a, MouseEvent>,
        Read<'a, Events>,
        Read<'a, rendering::Screen>,
    );

    #[allow(dead_code)]
    fn run(
        &mut self,
        (entities, pos, tr, mut pseudo, mut event, mouse, events, _screen): Self::SystemData,
    ) {
        use cgmath::Transform;
        let p: cgmath::Point3<f32> =
            cgmath::Point3::new(mouse.position.0 as f32, mouse.position.1 as f32, 0.0);

        for (e, pos, _tr, mut pseudo) in (&entities, &pos, &tr, &mut pseudo).join() {
            let p2 = pos.0.transform_point(cgmath::Point3::new(0.0, 0.0, 0.0));
            let size = pos.1;
            if p.x as f32 >= p2.x
                && p.x as f32 <= p2.x + size.0
                && p.y as f32 >= p2.y
                && p.y as f32 <= p2.y + size.1
            {
                pseudo.hover = true;
                if mouse.left_click == ButtonState::Pressed {
                    events.invoke(e.clone());
                    entities
                        .build_entity()
                        .with(
                            Event {
                                target: e,
                                event_type: EventType::Pressed,
                            },
                            &mut event,
                        )
                        .build();
                } else if mouse.left_click == ButtonState::Released {
                    entities
                        .build_entity()
                        .with(
                            Event {
                                target: e,
                                event_type: EventType::Released,
                            },
                            &mut event,
                        )
                        .build();
                }
            // println!("  {:?} {:?}", pos.0, p2);
            } else {
                pseudo.hover = false;
            }
        }
    }
}

//----------------------------------------
struct App<'a, 'b, R: gfx::Resources, F: gfx::Factory<R> + Clone> {
    renderer: rendering::Renderer<R, F>,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    store: manager::ResourceManager,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ButtonState {
    Up,
    Pressed,
    Down,
    Released,
}

#[derive(Debug)]
struct MouseEvent {
    position: (i32, i32),
    left_click: ButtonState,
}

impl Default for MouseEvent {
    fn default() -> Self {
        MouseEvent {
            position: (-1, -1),
            left_click: ButtonState::Up,
        }
    }
}

impl<'a, 'b, R: gfx::Resources, F: gfx::Factory<R> + Clone> gfx_app::Application<R, F>
    for App<'a, 'b, R, F>
{
    fn new(factory: F, backend: shade::Backend, window_targets: gfx_app::WindowTargets<R>) -> Self {
        println!("Backend: {:?}", backend);

        // let r = styling::parse("* { background-color: #ff0000; }")[0].clone();
        // r.selectors.matches()
        // println!("{:?}", r);

        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Vel>();
        world.register::<rendering::Material>();
        world.register::<rendering::Text>();
        world.register::<style_system::EElement>();
        world.register::<Event>();
        world.add_resource::<MouseEvent>(Default::default());
        world.add_resource::<Events>(Default::default());
        world.add_resource::<rendering::Screen>(rendering::Screen {
            size: window_targets.size,
        });

        let mut dispatcher = DispatcherBuilder::new()
            // .with(SysA, "sys_vel", &[])
            .with(
                specs_hierarchy::HierarchySystem::<Parent>::new(),
                "parent_hierarchy_system",
                &[],
            )
            .with(
                TransformSystem::new(),
                "transform_system",
                &["parent_hierarchy_system"],
            )
            .with(PickSystem, "sys_pick", &["transform_system"])
            .with(ConsumeEventsSystem, "sys_consume", &["sys_pick"])
            .with(StyleSystem::new(), "sys_style", &["sys_consume"])
            .with(layout_system::LayoutSystem, "sys_layout", &["sys_style"])
            .with(CleanEventsSystem, "sys_clean_events", &["sys_layout"])
            .build();
        dispatcher.setup(&mut world.res);

        let e1 = world
            .create_entity()
            .with(Transform::new(50.0, 50.0).with_size(200.0, 50.0))
            .with(EElement::new("Button".into()))
            .with(StyleBackground::from_color(255, 0, 0, 255))
            .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            .with(Vel(0.01))
            .build();

        {
            let mut events = world.write_resource::<Events>();
            events.register(e1, Box::new(move |e| println!("clicked {:?}", e)));
        }

        let e2 = world
            .create_entity()
            .with(Transform::new(5.0, 5.0).with_size(190.0, 40.0))
            .with(EElement::new("Border".into()))
            .with(StyleBackground::from_color(0, 255, 0, 255))
            .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            .with(Vel(0.005))
            .with(Parent { entity: e1 })
            .build();
        let _e3 = world
            .create_entity()
            .with(Transform::new(5.0, 5.0).with_size(180.0, 30.0))
            .with(EElement::new("Label".into()))
            .with(StyleBackground::from_color(0, 0, 255, 255))
            .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            .with(Parent { entity: e2 })
            .with(rendering::Text {
                text: "button".to_string(),
            })
            .build();

        let _e4 = world
            .create_entity()
            .with(Transform::new(400.0, 400.0))
            .with(EElement::new("Button".into()))
            .with(StyleBackground::from_color(255, 255, 0, 255))
            // .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            // .with(Parent { entity: e2 })
            .with(rendering::Text {
                text: "ent 4 child of 2".to_string(),
            })
            .build();

        let renderer = rendering::Renderer::new(factory, backend, window_targets);

        use manager::*;
        println!("current path {:?}", std::env::current_dir());
        let store: manager::ResourceManager =
            Store::new(StoreOpt::default()).expect("store creation");

        App {
            world,
            dispatcher,
            renderer,
            store: store,
        }
    }

    fn render<C2: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C2>) {
        self.renderer
            .render(&self.world.res, encoder, &mut self.store);
        self.dispatcher.dispatch(&mut self.world.res);
        self.store.sync(&mut manager::Ctx::new());

        {
            let mut m = self.world.write_resource::<MouseEvent>();
            // println!("  left click {:?}", m.left_click);
            match m.left_click {
                ButtonState::Released => m.left_click = ButtonState::Up,
                ButtonState::Pressed => m.left_click = ButtonState::Down,
                _ => {}
            }
        }
        self.world.maintain();
    }

    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.world.write_resource::<rendering::Screen>().size = window_targets.size;
        self.renderer.on_resize(window_targets);
    }

    fn on(&mut self, event: winit::WindowEvent) {
        match event {
            winit::WindowEvent::MouseInput {
                button: _button,
                state,
                ..
            } => {
                let mut m = self.world.write_resource::<MouseEvent>();
                let prev = m.left_click.clone();
                //state;
                use winit::ElementState;
                match (prev, state) {
                    (ButtonState::Up, ElementState::Pressed)
                    | (ButtonState::Released, ElementState::Pressed) => {
                        m.left_click = ButtonState::Pressed
                    }
                    (ButtonState::Down, ElementState::Released)
                    | (ButtonState::Pressed, ElementState::Released) => {
                        m.left_click = ButtonState::Released
                    }
                    _ => {}
                };
            }
            winit::WindowEvent::CursorMoved { position: p, .. } => {
                let p: (i32, i32) = p.into();
                let mut m = self.world.write_resource::<MouseEvent>();

                // hack: a first CursorMoved 0,0 event is sent on start even if the mouse is not in the window
                if m.position.0 == -1 && m.position.1 == -1 && p.0 == 0 && p.1 == 0 {
                    m.position = (-2, -2);
                } else {
                    m.position = p;
                }
            }
            _ => (),
        };
        // println!("{:?}",event);
    }
}

pub fn main() {
    use gfx_app::Application;
    App::launch_simple("Cube example");
}
