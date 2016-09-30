use gfx;


pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Position",
        tc: [f32; 2] = "a_TexCoord",
    }
    constant VsParams {
        mvp: [[f32; 4]; 4] = "u_ModelViewProj",
    }
    constant PsParams {
        color: [f32; 4] = "u_Color",
    }
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        v_par: gfx::ConstantBuffer<VsParams> = "b_VsParams",
        p_par: gfx::ConstantBuffer<PsParams> = "b_PsParams",
        texture: gfx::TextureSampler<[f32; 4]> = "u_Texture",
        ocolor: gfx::RenderTarget<ColorFormat> = "Target0",
        odepth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

const VS: &'static [u8] = b"
    #version 150 core

    uniform b_VsParams {
        mat4 u_ModelViewProj;
    };

    in vec4 a_Position;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = u_ModelViewProj * a_Position;
    }
";
const FS: &'static [u8] = b"
    #version 150 core

    uniform sampler2D u_Texture;
    uniform b_PsParams {
        vec4 u_Color;
    };

    in vec2 v_TexCoord;
    out vec4 Target0;

    void main() {
        Target0 = u_Color * texture(u_Texture, v_TexCoord);
    }
";

pub struct Context<D: gfx::Device, F> {
    device: D,
    factory: F,
    encoder: gfx::Encoder<D::Resources, D::CommandBuffer>,
    out_color: gfx::handle::RenderTargetView<D::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<D::Resources, DepthFormat>,
    pso: gfx::PipelineState<D::Resources, pipe::Meta>,
}

impl<D: gfx::Device, F: gfx::traits::FactoryExt<D::Resources>> Context<D, F> {
    pub fn new(d: D, mut f: F, cb: D::CommandBuffer,
               oc: gfx::handle::RenderTargetView<D::Resources, ColorFormat>,
               od: gfx::handle::DepthStencilView<D::Resources, DepthFormat>)
               -> Self
    {
        let pso = f.create_pipeline_simple(VS, FS, pipe::new()).unwrap();
        Context {
            device: d,
            factory: f,
            encoder: cb.into(),
            out_color: oc,
            out_depth: od,
            pso: pso,
        }
    }

    pub fn begin(&mut self, background: [f32; 4]) {
        self.device.cleanup();
        self.encoder.clear(&self.out_color, background);
        self.encoder.clear_depth(&self.out_depth, 1.0);
    }

    pub fn end(&mut self) {
        self.encoder.flush(&mut self.device);
    }
}
