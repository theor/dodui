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

struct SysA;
impl<'a> System<'a> for SysA {
    type SystemData = (WriteStorage<'a, Transform>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.position.x += vel.0;
        }
    }
}

struct PickSystem;
impl<'a> System<'a> for PickSystem {
    type SystemData = (ReadStorage<'a, GlobalTransform>, Read<'a, MouseEvent>);
    fn run(&mut self, (pos, mouse): Self::SystemData) {
        use cgmath::SquareMatrix;
        use cgmath::Transform;

        for pos in (&pos).join() {
            let cam = rendering::cam(1.33f32) * rendering::default_view() * pos.0;
            let p2 = cam.transform_point(cgmath::Point3::new(0.0, 0.0, 0.0));
            // println!("{:?}", p2);
        }

        let p: cgmath::Point3<f32> =
            cgmath::Point3::new(mouse.position.0 as f32, mouse.position.1 as f32, 0.0);
        let cam = rendering::cam(1.33f32) * rendering::default_view();
        let p2 = cam.invert().unwrap().transform_point(p);
        // println!("{:?} {:?}", p, p2);
    }
}

//----------------------------------------
struct App<'a, 'b, R: gfx::Resources, F: gfx::Factory<R>> {
    renderer: rendering::Renderer<R, F>,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    store: manager::ResourceManager,
}

#[derive(Debug, Default)]
struct MouseEvent {
    position: (i32, i32),
}

impl<'a, 'b, R: gfx::Resources, F: gfx::Factory<R>> gfx_app::Application<R, F>
    for App<'a, 'b, R, F>
{
    fn new(
        mut factory: F,
        backend: shade::Backend,
        window_targets: gfx_app::WindowTargets<R>,
    ) -> Self {
        println!("Backend: {:?}", backend);

        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Vel>();
        world.register::<rendering::Material>();
        world.add_resource::<MouseEvent>(Default::default());
        world.add_resource::<rendering::Screen>(rendering::Screen {
            size: window_targets.size,
        });

        let mut dispatcher = DispatcherBuilder::new()
            // .with(SysA, "sys_vel", &[])
            .with(PickSystem, "sys_pick", &[])
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
            .build();
        dispatcher.setup(&mut world.res);

        let e1 = world
            .create_entity()
            .with(Vel(0.01))
            .with(Transform::new(0.0, 0.0))
            .with(rendering::Material::from_color(1.0, 0.0, 0.0, 1.0))
            .build();
        let e2 = world
            .create_entity()
            .with(Transform::new(2.0, 2.0))
            .with(rendering::Material::from_color(0.0, 1.0, 0.0, 1.0))
            .with(Vel(0.005))
            .with(Parent { entity: e1 })
            .build();
        let _e3 = world
            .create_entity()
            .with(rendering::Material::from_color(0.0, 0.0, 1.0, 1.0))
            .with(Parent { entity: e1 })
            .with(Transform::new(2.0, 4.0))
            .build();

        let _e4 = world
            .create_entity()
            .with(Transform::new(4.0, 7.0))
            .with(rendering::Material::from_color(1.0, 1.0, 0.0, 1.0))
            .with(Parent { entity: e2 })
            .build();

        let renderer = rendering::Renderer::new(factory, backend, window_targets);

        use manager::*;
        let mut ctx = Ctx::new();
        println!("current path {:?}", std::env::current_dir());
        let mut store: manager::ResourceManager =
            Store::new(StoreOpt::default()).expect("store creation");
        use std::path::Path;

        // let my_resource = store.get::<FromFS>(&Path::new("shader/cube.hlsl").into(), &mut ctx);
        // println!("loaded {:?}", my_resource);

        let my_resource = store
            .get::<ShaderSet>(&"shader/cube.hlsl".into(), &mut ctx)
            .unwrap();
        println!("loaded {:?}", my_resource);
        {
            // let mut m = my_resource.borrow_mut();
            // let asd =  (*m);
            // m.0 = m.0 + 1;
        }
        println!("loaded {:?}", my_resource);

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
                self.world.write_resource::<MouseEvent>().position = p;
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
