use gfx;


pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub struct Context<D: gfx::Device, F> {
    pub device: D,
    pub factory: F,
    pub encoder: gfx::Encoder<D::Resources, D::CommandBuffer>,
    pub out_color: gfx::handle::RenderTargetView<D::Resources, ColorFormat>,
    pub out_depth: gfx::handle::DepthStencilView<D::Resources, DepthFormat>,
}
