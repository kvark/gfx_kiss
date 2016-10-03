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
            context: context::Context::new(device, factory, cbuf, main_color, main_depth),
            background: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn get_frame_size(&self) -> (u32, u32) {
        self.window.get_inner_size().unwrap()
    }

    pub fn render(&mut self) -> bool {
        use cgmath;
        for event in self.window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => return false,
                _ => {},
            }
        }

        self.context.draw(self.background, cgmath::SquareMatrix::identity()); //TODO
        self.window.swap_buffers().unwrap();
        true
    }

    pub fn with<T, P: FnOnce(&mut context::Object)->T>(&mut self, h: &context::Handle, fun: P) -> T {
        self.context.with(h, fun)
    }

    pub fn add_point(&mut self, c: [f32; 3]) -> context::Handle {
        let vert = context::Vertex {
            pos: [c[0], c[1], c[2], 1.0],
            tc: [0.0, 0.0],
        };
        self.context.add(context::Kind::Point, &[vert], ())
    }

    pub fn add_line(&mut self, a: [f32; 3], b: [f32; 3]) -> context::Handle {
        let va = context::Vertex {
            pos: [a[0], a[1], a[2], 1.0],
            tc: [0.0, 0.0],
        };
        let vb = context::Vertex {
            pos: [b[0], b[1], b[2], 1.0],
            tc: [1.0, 0.0],
        };
        self.context.add(context::Kind::Line, &[va, vb], ())
    }

    pub fn remove(&mut self, h: context::Handle) {
        self.context.remove(h)
    }
}
