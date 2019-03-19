mod material;
pub use material::*;

use crate::gfx_app;
use crate::gfx_app::{ColorFormat, DepthFormat};
use crate::shade;
use cgmath::{Matrix4, Point3, Vector3};
use gfx;
use gfx::texture;

use crate::transform::{GlobalTransform, Transform};
use specs::prelude::*;

#[derive(Debug, Default)]
pub struct Screen {
    pub size: (u32, u32),
}

pub struct Text {
    pub text: String,
}

impl Component for Text {
    type Storage = DenseVecStorage<Self>;
}

pub struct SysRender<'a, R: gfx::Resources, C: gfx::CommandBuffer<R>, F: Clone + gfx::Factory<R>> {
    slice: &'a gfx::Slice<R>,
    data: &'a pipe::Data<R>,
    pso: &'a gfx::PipelineState<R, pipe::Meta>,
    encoder: &'a mut gfx::Encoder<R, C>,
    text: &'a mut Option<gfx_text::Renderer<R, F>>,
}

impl<'a, R: gfx::Resources, C: gfx::CommandBuffer<R>, F: Clone + gfx::Factory<R>> System<'a>
    for SysRender<'a, R, C, F>
{
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, Text>,
        // Read<'a, Screen>,
    );
    fn run(&mut self, (tr, pos, mat, text): Self::SystemData) {
        self.encoder
            .clear(&self.data.out_color, [0.1, 0.2, 0.3, 1.0]);
        self.encoder.clear_depth(&self.data.out_depth, 1.0);
        let vp: cgmath::Matrix4<f32> = self.data.transform.into();

        for (tr, pos, mat, text) in (&tr, &pos, &mat, text.maybe()).join() {
            let m = pos.0;
            let locals = Locals {
                transform: (vp * m).into(),
                color: [
                    mat.color.x as f32 / 255.0,
                    mat.color.y as f32 / 255.0,
                    mat.color.z as f32 / 255.0,
                    mat.color.w as f32 / 255.0,
                ],
                size: tr.size.into(),
            };
            self.encoder
                .update_constant_buffer(&self.data.locals, &locals);
            self.encoder.draw(self.slice, self.pso, self.data);

            if let Some(text_renderer) = self.text {
                if let Some(text) = text {
                    use cgmath::Transform;
                    let p = m.transform_point(cgmath::Point3::new(0.0,0.0,0.0));
                    text_renderer.add(
                        &text.text, // Text to add
                        [p.x as i32, p.y as i32],                                      // Position
                        [0.9, 0.9, 0.9, 1.0],                       // Text color
                    );
                    if let Err(e) = text_renderer.draw(self.encoder, &self.data.out_color) {
                        match e {
                            gfx_text::Error::PipelineError(p) => { println!("{}", p); },
                            _ => println!("{:?}", e),
                        }
                    }
                }
            }
        }
    }
}

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
        color: [f32; 4] = "u_Color",
        size: [f32; 2] = "u_Size",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
        screen: gfx::Global<[f32; 2]> = "u_Screen",
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

pub struct Renderer<R: gfx::Resources, F: Clone + gfx::Factory<R>> {
    factory: F,
    slice: gfx::Slice<R>,
    data: pipe::Data<R>,
    pso: Option<gfx::PipelineState<R, pipe::Meta>>,
    version: u8,
    text_version: u8,
    text: Option<gfx_text::Renderer<R, F>>,
}

impl<R: gfx::Resources, F: Clone + gfx::Factory<R>> Renderer<R, F> {
    pub fn new(
        mut factory: F,
        _backend: shade::Backend,
        window_targets: gfx_app::WindowTargets<R>,
    ) -> Self {
        use gfx::traits::FactoryExt;
        println!("size {:?}", window_targets.size);

        let v = 1;
        let vertex_data = [
            Vertex::new([0, 0, 0], [0, 0]),
            Vertex::new([v, 0, 0], [1, 0]),
            Vertex::new([v, v, 0], [1, 1]),
            Vertex::new([0, v, 0], [0, 1]),
        ];

        let index_data: &[u16] = &[0, 1, 2, 2, 3, 0];

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

        let proj = cam(window_targets.size);

        let data = pipe::Data {
            vbuf: vbuf,
            transform: (proj * default_view()).into(),
            screen: [window_targets.size.0 as f32, window_targets.size.1 as f32],
            locals: factory.create_constant_buffer(1),
            color: (texture_view, factory.create_sampler(sinfo)),
            out_color: window_targets.color,
            out_depth: window_targets.depth,
        };

        Renderer {
            factory,
            slice,
            data,
            pso: None,
            version: 0,
            text_version: 0,
            text: None,
            // bundle: Bundle::new(slice, pso, data),
        }
    }

    pub fn render<C: gfx::CommandBuffer<R>>(
        &mut self,
        res: &specs::Resources,
        // factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>,
    ) {
        use crate::manager::*;
        use gfx::traits::FactoryExt;

        let store = res.fetch_mut::<crate::manager::ResourceManager>();

        let dep = SimpleKey::Logical(("shader/cube.hlsl").into());
        match store.get::<ShaderSet>(&dep) {
            Ok(set) => {
                let set = set.borrow_mut();
                if set.version != self.version {
                    self.version = set.version;
                    self.pso = self
                        .factory
                        .create_pipeline_simple(&set.vx, &set.px, crate::rendering::pipe::new())
                        .ok();
                }
            }
            e => {
                println!("Error {:?}", e);
            }
        }
        
        let dep = SimpleKey::Logical(("shader/text.hlsl").into());

        match store.get::<ShaderSet>(&dep) {
            Ok(set) => {
                let set = set.borrow_mut();
                if set.version != self.text_version {
                    self.text_version = set.version;

                    let text = gfx_text::new(self.factory.clone()).build(&set.vx, &set.px);
                    if text.is_err() {
                        println!("{:?}", text.as_ref().err());
                    }
                    self.text = text.ok();
                }
            }
            e => {
                println!("Error {:?}", e);
            }
        }

        match self.pso.as_ref() {
            Some(pso) => {
                let mut sys = SysRender {
                    slice: &self.slice,
                    pso: pso,
                    data: &self.data,
                    encoder: encoder,
                    text: &mut self.text,
                };
                sys.run_now(res);
            }
            None => {}
        }
    }

    pub fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.data.out_color = window_targets.color;
        self.data.out_depth = window_targets.depth;

        // In this example the transform is static except for window resizes.
        let proj = cam(window_targets.size); // cgmath::perspective(Deg(45.0f32), window_targets.aspect_ratio, 1.0, 10.0);
        self.data.transform = (proj * default_view()).into();
    }
}

pub fn default_view() -> Matrix4<f32> {
    Matrix4::look_at(
        Point3::new(0f32, 0f32, 1f32),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_y(),
    )
}

pub fn cam((w, h): (u32, u32)) -> Matrix4<f32> {
    cgmath::ortho(0.0f32, w as f32, h as f32, 0.0f32, -1.0f32, 1.0f32)
}
