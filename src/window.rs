use gfx_device_gl;
use glutin;

use context;


pub struct Window {
    window: glutin::Window,
    context: context::Context<gfx_device_gl::Device, gfx_device_gl::Factory>,
    pub background: [f32; 4],
}

impl Window {
    pub fn new(title: &str) -> Window {
        use env_logger;
        use gfx_window_glutin;

        env_logger::init().unwrap();

        let gl_version = glutin::GlRequest::GlThenGles {
            opengl_version: (3, 2),
            opengles_version: (2, 0),
        };
        let builder = glutin::WindowBuilder::new()
            .with_title(title.to_string())
            .with_dimensions(920, 600)  //TODO?
            .with_gl(gl_version)
            .with_vsync();
        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<context::ColorFormat, context::DepthFormat>(builder);

        let cbuf = factory.create_command_buffer();
        Window {
            window: window,
            context: context::Context {
                device: device,
                factory: factory,
                encoder: cbuf.into(),
                out_color: main_color,
                out_depth: main_depth,
            },
            background: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn get_frame_size(&self) -> (u32, u32) {
        self.window.get_inner_size().unwrap()
    }

    pub fn render(&mut self) -> bool {
        use gfx::Device;

        for event in self.window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => return false,
                _ => {},
            }
        }

        self.context.encoder.clear(&self.context.out_color, self.background);
        self.context.encoder.clear_depth(&self.context.out_depth, 1.0);
        //TODO: actual rendering
        self.context.encoder.flush(&mut self.context.device);
        self.window.swap_buffers().unwrap();
        self.context.device.cleanup();

        true
    }
}
