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
extern crate gfx_app;

use specs::prelude::*;

mod transform;
mod rendering;
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
    type SystemData = (ReadStorage<'a, Transform>, Read<'a, MouseEvent>);
    fn run(&mut self, (_pos, _mouse): Self::SystemData) {}
}



//----------------------------------------
struct App<'a, 'b, R: gfx::Resources> {
    renderer: rendering::Renderer<R>,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

#[derive(Default)]
struct MouseEvent {
    position: (i32, i32),
}

impl<'a, 'b, R: gfx::Resources> gfx_app::Application<R> for App<'a, 'b, R> {
    fn new<F: gfx::Factory<R>>(
        factory: &mut F,
        backend: gfx_app::shade::Backend,
        window_targets: gfx_app::WindowTargets<R>,
    ) -> Self {
        

        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Vel>();
        world.add_resource::<MouseEvent>(Default::default());

        let mut dispatcher = DispatcherBuilder::new()
            .with(SysA, "sys_vel", &[])
            .with(
                specs_hierarchy::HierarchySystem::<Parent>::new(),
                "parent_hierarchy_system",
                &["sys_vel"],
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
            .build();
        let e2 = world.create_entity().with(Transform::new(2.0, 2.0)).build();
        let e3 = world
            .create_entity()
            // .with(Vel(0.01))
            .with(Transform::new(2.0, 4.0))
            .build();

        // This entity does not have `Vel`, so it won't be dispatched.
        let e4 = world.create_entity().with(Transform::new(4.0, 8.0)).build();

        {
            let mut parents = world.write_storage::<Parent>();
            parents.insert(e2, Parent { entity: e1 }).unwrap();
            parents.insert(e3, Parent { entity: e1 }).unwrap();
            parents.insert(e4, Parent { entity: e2 }).unwrap();
        }

        let renderer = rendering::Renderer::new(factory, backend, window_targets);

        App {
            world,
            dispatcher,
            renderer,
        }
    }

    fn render<C2: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C2>) {
        self.renderer.render(&self.world.res, encoder);
        self.dispatcher.dispatch(&mut self.world.res);
    }

    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
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