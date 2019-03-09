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

#[derive(Debug)]
pub struct StyleBackground { color: cgmath::Vector4<u8> }

impl Component for StyleBackground  {
    type Storage = DenseVecStorage<Self>;
}

impl StyleBackground {
    pub fn from_color(r:u8, g:u8, b:u8, a:u8) -> Self {        
        Self {
            color: cgmath::Vector4::new(r,g,b,a)
        }
    }
}

#[derive(Debug, Default)]
pub struct Pseudo { hover: bool }

impl Component for Pseudo  {
    type Storage = DenseVecStorage<Self>;
}


struct PickSystem;
impl<'a> System<'a> for PickSystem {
    type SystemData = (
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Pseudo>,
        Read<'a, MouseEvent>,
        Read<'a, rendering::Screen>);

    #[allow(dead_code)]
    fn run(&mut self, (pos, tr, mut pseudo, mouse, _screen): Self::SystemData) {
        use cgmath::Transform;
        let p: cgmath::Point3<f32> =
            cgmath::Point3::new(mouse.position.0 as f32, mouse.position.1 as f32, 0.0);

        for (pos,tr, mut pseudo) in (&pos, &tr, &mut pseudo).join() {
            let p2 = pos.0.transform_point(cgmath::Point3::new(0.0, 0.0, 0.0));
            if p.x as f32 >= p2.x && p.x as f32 <= p2.x + tr.size.x &&
               p.y as f32 >= p2.y && p.y as f32 <= p2.y + tr.size.y {
                pseudo.hover = true;
                // println!("  {:?} {:?}", pos.0, p2);
            } else {
                pseudo.hover = false;
            }
        }
    }
}

struct StyleSystem;
impl<'a> System<'a> for StyleSystem {
    type SystemData = (
        ReadStorage<'a, Pseudo>,
        ReadStorage<'a, StyleBackground>,
        WriteStorage<'a, rendering::Material>
    );

    #[allow(dead_code)]
    fn run(&mut self, (pseudo, bg, mut mat): Self::SystemData) {
        for (pseudo, bg, mut mat) in (pseudo.maybe(), &bg, &mut mat).join() {
            mat.color = if pseudo.map_or(false, |v| v.hover) { bg.color } else { bg.color / 2 };
        }
    }
}



//----------------------------------------
struct App<'a, 'b, R: gfx::Resources, F: gfx::Factory<R>+Clone> {
    renderer: rendering::Renderer<R, F>,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    store: manager::ResourceManager,
}

#[derive(Debug, Default)]
struct MouseEvent {
    position: (i32, i32),
}

impl<'a, 'b, R: gfx::Resources, F: gfx::Factory<R>+Clone> gfx_app::Application<R, F>
    for App<'a, 'b, R, F>
{
    fn new(
        factory: F,
        backend: shade::Backend,
        window_targets: gfx_app::WindowTargets<R>,
    ) -> Self {
        println!("Backend: {:?}", backend);

        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Vel>();
        world.register::<rendering::Material>();
        world.register::<rendering::Text>();
        world.add_resource::<MouseEvent>(MouseEvent { position:(-1,-1) });
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
            .with(StyleSystem, "sys_style", &["sys_pick"])
            .build();
        dispatcher.setup(&mut world.res);

        let e1 = world
            .create_entity()
            .with(Transform::new(0.0, 0.0))
            .with(StyleBackground::from_color(255, 0, 0, 255))
            .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            .with(Vel(0.01))
            .with(rendering::Text {text: "asdf".to_string()})
            .build();
        let e2 = world
            .create_entity()
            .with(Transform::new(200.0, 200.0))
            .with(StyleBackground::from_color(0, 255, 0, 255))
            .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            .with(Vel(0.005))
            .with(Parent { entity: e1 })
            .build();
        let _e3 = world
            .create_entity()
            .with(Transform::new(200.0, 400.0))
            .with(StyleBackground::from_color(0, 0, 255, 255))
            .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            .with(Parent { entity: e1 })
            .build();

        let _e4 = world
            .create_entity()
            .with(Transform::new(400.0, 400.0))
            .with(StyleBackground::from_color(255, 255, 0, 255))
            // .with(<Pseudo as Default>::default())
            .with(rendering::Material::default())
            .with(Parent { entity: e2 })
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
        self.renderer.render(&self.world.res, encoder, &mut self.store);
        self.dispatcher.dispatch(&mut self.world.res);
        self.store.sync(&mut manager::Ctx::new());
    }

    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.world.write_resource::<rendering::Screen>().size = window_targets.size;
        self.renderer.on_resize(window_targets);
    }

    fn on(&mut self, event: winit::WindowEvent) {
        match event {
            winit::WindowEvent::CursorMoved { position: p, .. } => {
                let p: (i32, i32) = p.into();
                let mut m = self.world.write_resource::<MouseEvent>();

                // hack: a first CursorMoved 0,0 event is sent on start even if the mouse is not in the window
                if m.position.0 == -1 && m.position.1 == -1 && p.0 == 0 && p.1 == 0 {
                    m.position = (-2,-2);
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
