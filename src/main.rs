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

#[derive(Debug)]
struct Vel(f32);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Pos(pub f32);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

struct SysA;
struct SysRender<'a, R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    bundle: &'a Bundle<R, pipe::Data<R>>,
    encoder: &'a mut gfx::Encoder<R, C>,
}

impl<'a, R: gfx::Resources, C: gfx::CommandBuffer<R>> System<'a> for SysRender<'a, R, C> {
    type SystemData = (ReadStorage<'a, Pos>);
    fn run(&mut self, pos: Self::SystemData) {
        self.encoder.clear(&self.bundle.data.out_color, [0.1, 0.2, 0.3, 1.0]);
        self.encoder.clear_depth(&self.bundle.data.out_depth, 1.0);
        let vp: cgmath::Matrix4<f32> = self.bundle.data.transform.into();

        for pos in (&pos).join() {
            let m = cgmath::Matrix4::from_translation([pos.0, 0.0f32, 0.0f32].into());
            let locals = Locals {
                transform: (vp * m).into(),
            };
            self.encoder
                .update_constant_buffer(&self.bundle.data.locals, &locals);
            self.bundle.encode(&mut self.encoder);
        }
    }
}

impl<'a> System<'a> for SysA {
    // These are the resources required for execution.
    // You can also define a struct and `#[derive(SystemData)]`,
    // see the `full` example.
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        // The `.join()` combines multiple component storages,
        // so we get access to all entities which have
        // both a position and a velocity.
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += vel.0;
        }
    }
}

pub use gfx_app::{ColorFormat, DepthFormat};

use cgmath::{Deg, Matrix4, Point3, Vector3};
use gfx::{texture, Bundle};

// Declare the vertex format suitable for drawing,
// as well as the constants used by the shaders
// and the pipeline state object format.
// Notice the use of FixedPoint.
gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
        Vertex {
            pos: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
            tex_coord: [t[0] as f32, t[1] as f32],
        }
    }
}

//----------------------------------------
struct App<'a, 'b, R: gfx::Resources> {
    bundle: Bundle<R, pipe::Data<R>>,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b, R: gfx::Resources> gfx_app::Application<R> for App<'a, 'b, R> {
    fn new<F: gfx::Factory<R>>(
        factory: &mut F,
        backend: gfx_app::shade::Backend,
        window_targets: gfx_app::WindowTargets<R>,
    ) -> Self {
        use gfx::traits::FactoryExt;

        let vs = gfx_app::shade::Source {
            glsl_120: include_bytes!("shader/cube_120.glslv"),
            glsl_150: include_bytes!("shader/cube_150_core.glslv"),
            glsl_es_100: include_bytes!("shader/cube_100_es.glslv"),
            glsl_es_300: include_bytes!("shader/cube_300_es.glslv"),
            hlsl_40: include_bytes!("data/vertex.fx"),
            msl_11: include_bytes!("shader/cube_vertex.metal"),
            vulkan: include_bytes!("data/vert.spv"),
            ..gfx_app::shade::Source::empty()
        };
        let ps = gfx_app::shade::Source {
            glsl_120: include_bytes!("shader/cube_120.glslf"),
            glsl_150: include_bytes!("shader/cube_150_core.glslf"),
            glsl_es_100: include_bytes!("shader/cube_100_es.glslf"),
            glsl_es_300: include_bytes!("shader/cube_300_es.glslf"),
            hlsl_40: include_bytes!("data/pixel.fx"),
            msl_11: include_bytes!("shader/cube_frag.metal"),
            vulkan: include_bytes!("data/frag.spv"),
            ..gfx_app::shade::Source::empty()
        };

        let vertex_data = [
            // top (0, 0, 1)
            Vertex::new([-1, -1, 1], [0, 0]),
            Vertex::new([1, -1, 1], [1, 0]),
            Vertex::new([1, 1, 1], [1, 1]),
            Vertex::new([-1, 1, 1], [0, 1]),
            // bottom (0, 0, -1)
            Vertex::new([-1, 1, -1], [1, 0]),
            Vertex::new([1, 1, -1], [0, 0]),
            Vertex::new([1, -1, -1], [0, 1]),
            Vertex::new([-1, -1, -1], [1, 1]),
            // right (1, 0, 0)
            Vertex::new([1, -1, -1], [0, 0]),
            Vertex::new([1, 1, -1], [1, 0]),
            Vertex::new([1, 1, 1], [1, 1]),
            Vertex::new([1, -1, 1], [0, 1]),
            // left (-1, 0, 0)
            Vertex::new([-1, -1, 1], [1, 0]),
            Vertex::new([-1, 1, 1], [0, 0]),
            Vertex::new([-1, 1, -1], [0, 1]),
            Vertex::new([-1, -1, -1], [1, 1]),
            // front (0, 1, 0)
            Vertex::new([1, 1, -1], [1, 0]),
            Vertex::new([-1, 1, -1], [0, 0]),
            Vertex::new([-1, 1, 1], [0, 1]),
            Vertex::new([1, 1, 1], [1, 1]),
            // back (0, -1, 0)
            Vertex::new([1, -1, 1], [0, 0]),
            Vertex::new([-1, -1, 1], [1, 0]),
            Vertex::new([-1, -1, -1], [1, 1]),
            Vertex::new([1, -1, -1], [0, 1]),
        ];

        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

        let texels = [[0x20, 0xA0, 0xC0, 0xFF]];
        let (_, texture_view) = factory
            .create_texture_immutable::<gfx::format::Rgba8>(
                texture::Kind::D2(1, 1, texture::AaMode::Single),
                texture::Mipmap::Provided,
                &[&texels],
            )
            .unwrap();

        let sinfo =
            texture::SamplerInfo::new(texture::FilterMethod::Bilinear, texture::WrapMode::Clamp);

        let pso = factory
            .create_pipeline_simple(
                vs.select(backend).unwrap(),
                ps.select(backend).unwrap(),
                pipe::new(),
            )
            .unwrap();

        let proj = cgmath::perspective(Deg(45.0f32), window_targets.aspect_ratio, 1.0, 10.0);

        let data = pipe::Data {
            vbuf: vbuf,
            transform: (proj * default_view()).into(),
            locals: factory.create_constant_buffer(1),
            color: (texture_view, factory.create_sampler(sinfo)),
            out_color: window_targets.color,
            out_depth: window_targets.depth,
        };

        let mut world = World::new();
        world.register::<Pos>();
        world.register::<Vel>();

        world.create_entity().with(Vel(0.01)).with(Pos(0.0)).build();
        world.create_entity().with(Vel(0.01)).with(Pos(1.6)).build();
        world.create_entity().with(Vel(0.01)).with(Pos(5.4)).build();

        // This entity does not have `Vel`, so it won't be dispatched.
        world.create_entity().with(Pos(-2.0)).build();

        let mut dispatcher = DispatcherBuilder::new()
            .with(SysA, "sys_a", &[])
            // .with_thread_local(SysRender { bundle: Bundle::new(slice, pso, data), phantomc: PhantomData })
            .build();
        // This will call the `setup` function of every system.
        // In this example this has no effect since we already registered our components.
        dispatcher.setup(&mut world.res);

        App {
            world,
            dispatcher,
            bundle: Bundle::new(slice, pso, data),
        }
    }

    fn render<C2: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C2>) {
        let mut sys = SysRender {
            bundle: &self.bundle,
            encoder: encoder,
        };
        sys.run_now(&self.world.res);
        self.dispatcher.dispatch(&mut self.world.res);
    }

    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.bundle.data.out_color = window_targets.color;
        self.bundle.data.out_depth = window_targets.depth;

        // In this example the transform is static except for window resizes.
        let proj = cgmath::perspective(Deg(45.0f32), window_targets.aspect_ratio, 1.0, 10.0);
        self.bundle.data.transform = (proj * default_view()).into();
    }
}

pub fn main() {
    use gfx_app::Application;
    App::launch_simple("Cube example");
}

fn default_view() -> Matrix4<f32> {
    Matrix4::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    )
}
