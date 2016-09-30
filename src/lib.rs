extern crate env_logger;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;


mod context;
mod window;

pub use window::Window;
