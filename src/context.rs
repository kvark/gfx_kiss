use std::collections::HashMap;
use cgmath;
use gfx;


pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Position",
        tc: [f32; 2] = "a_TexCoord",
    }
    constant Locals {
        world: [[f32; 4]; 4] = "u_World",
        color: [f32; 4] = "u_Color",
    }
    constant Globals {
        view_proj: [[f32; 4]; 4] = "u_ViewProj",
    }
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "b_Locals",
        globals: gfx::ConstantBuffer<Globals> = "b_Globals",
        texture: gfx::TextureSampler<[f32; 4]> = "u_Texture",
        ocolor: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        odepth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}


const VS: &'static [u8] = b"
    #version 150 core

    uniform b_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
    uniform b_Globals {
        mat4 u_ViewProj;
    };

    in vec4 a_Position;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = u_ViewProj * (u_World * a_Position);
    }
";
const FS: &'static [u8] = b"
    #version 150 core

    uniform sampler2D u_Texture;
    uniform b_Locals {
        mat4 u_World;
        vec4 u_Color;
    };

    in vec2 v_TexCoord;
    out vec4 Target0;

    void main() {
        Target0 = u_Color * texture(u_Texture, v_TexCoord);
    }
";

#[derive(Eq, Hash, PartialEq)]
pub struct Handle(u32);

pub struct Object {
    pub pos: cgmath::Vector3<f32>,
    pub orient: cgmath::Quaternion<f32>,
    pub scale: f32,
    pub color: [f32; 4],
}

impl Object {
    fn new() -> Object {
        Object {
            pos: cgmath::Zero::zero(),
            orient: cgmath::One::one(),
            scale: 1.0,
            color: [1.0; 4],
        }
    }

    fn to_locals(&self) -> Locals {
        Locals {
            world: cgmath::Matrix4::from(cgmath::Decomposed {
                disp: self.pos,
                rot: self.orient,
                scale: self.scale,
            }).into(),
            color: self.color,
        }
    }
}

pub enum Kind {
    Point,
    Line,
    Mesh,
}

struct Entry<R: gfx::Resources> {
    object: Object,
    kind: Kind,
    slice: gfx::Slice<R>,
    pso: pipe::Data<R>,
}

pub struct Context<D: gfx::Device, F> {
    last_id: u32,
    device: D,
    factory: F,
    encoder: gfx::Encoder<D::Resources, D::CommandBuffer>,
    out_color: gfx::handle::RenderTargetView<D::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<D::Resources, DepthFormat>,
    pso_point: gfx::PipelineState<D::Resources, pipe::Meta>,
    pso_line: gfx::PipelineState<D::Resources, pipe::Meta>,
    pso_mesh: gfx::PipelineState<D::Resources, pipe::Meta>,
    cb_globals: gfx::handle::Buffer<D::Resources, Globals>,
    tex_dummy: gfx::handle::ShaderResourceView<D::Resources, [f32; 4]>,
    sampler: gfx::handle::Sampler<D::Resources>,
    data: HashMap<Handle, Entry<D::Resources>>,
}

impl<D: gfx::Device, F: gfx::traits::FactoryExt<D::Resources>> Context<D, F> {
    pub fn new(d: D, mut f: F, cb: D::CommandBuffer,
               oc: gfx::handle::RenderTargetView<D::Resources, ColorFormat>,
               od: gfx::handle::DepthStencilView<D::Resources, DepthFormat>)
               -> Self
    {
        let prog = f.link_program(VS, FS).unwrap();
        let p1 = f.create_pipeline_from_program(&prog, gfx::Primitive::PointList,
            gfx::state::Rasterizer::new_fill(), pipe::new()).unwrap();
        let p2 = f.create_pipeline_from_program(&prog, gfx::Primitive::LineList,
            gfx::state::Rasterizer::new_fill(), pipe::new()).unwrap();
        let p3 = f.create_pipeline_from_program(&prog, gfx::Primitive::TriangleList,
            gfx::state::Rasterizer::new_fill().with_cull_back(), pipe::new()).unwrap();
        let globals = f.create_constant_buffer(1);
        let (_, texture) = f.create_texture_const::<gfx::format::Rgba8>(
            gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single),
            &[&[[0xFFu8, 0xFF, 0xFF, 0xFF]] as &[[u8; 4]]]).unwrap();
        let sampler = f.create_sampler_linear();
        Context {
            last_id: 0,
            device: d,
            factory: f,
            encoder: cb.into(),
            out_color: oc,
            out_depth: od,
            pso_point: p1,
            pso_line: p2,
            pso_mesh: p3,
            cb_globals: globals,
            tex_dummy: texture,
            sampler: sampler,
            data: HashMap::new(),
        }
    }

    pub fn draw(&mut self, background: [f32; 4], view_proj: cgmath::Matrix4<f32>) {
        self.device.cleanup();
        self.encoder.update_constant_buffer(&self.cb_globals, &Globals {
            view_proj: view_proj.into(),
        });
        self.encoder.clear(&self.out_color, background);
        self.encoder.clear_depth(&self.out_depth, 1.0);
        for data in self.data.values() {
            let pso = match data.kind {
                Kind::Point => &self.pso_point,
                Kind::Line => &self.pso_line,
                Kind::Mesh => &self.pso_mesh,
            };
            self.encoder.draw(&data.slice, pso, &data.pso);
        }
        self.encoder.flush(&mut self.device);
    }

    pub fn with<T, P: FnOnce(&mut Object)->T>(&mut self, h: &Handle, fun: P) -> T {
        let mut data = self.data.get_mut(h).unwrap();
        let ret = fun(&mut data.object);
        let locals = data.object.to_locals();
        self.encoder.update_constant_buffer(&data.pso.locals, &locals);
        ret
    }

    pub fn add<I: gfx::IntoIndexBuffer<D::Resources>>(&mut self, kind: Kind, vertices: &[Vertex], indices: I) -> Handle {
        self.last_id += 1;
        let object = Object::new();
        let locals = self.factory.create_constant_buffer(1);
        let (vbuf, slice) = self.factory.create_vertex_buffer_with_slice(vertices, indices);
        self.encoder.update_constant_buffer(&locals, &object.to_locals());
        self.data.insert(Handle(self.last_id), Entry {
            object: object,
            kind: kind,
            slice: slice,
            pso: pipe::Data {
                vbuf: vbuf,
                locals: locals,
                globals: self.cb_globals.clone(),
                texture: (self.tex_dummy.clone(), self.sampler.clone()),
                ocolor: self.out_color.clone(),
                odepth: self.out_depth.clone(),
            },
        });
        Handle(self.last_id)
    }

    pub fn remove(&mut self, h: Handle) {
        self.data.remove(&h);
    }
}
