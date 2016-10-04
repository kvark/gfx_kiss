use std::cmp;
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

    pub fn add_points(&mut self, points: &[[f32; 3]]) -> context::Handle {
        let ratio = 1.0 / cmp::max(1, points.len()) as f32;
        let verts: Vec<_> = points.iter().enumerate().map(|(i, p)| context::Vertex {
            pos: [p[0], p[1], p[2], 1.0],
            tc: [(i as f32 + 0.5) * ratio, 0.0],
        }).collect();
        self.context.add(context::Kind::Point, &verts, ())
    }

    pub fn add_lines(&mut self, lines: &[[f32; 3]]) -> context::Handle {
        let ratio = 1.0 / (cmp::max(2, lines.len()) - 1) as f32;
        let verts: Vec<_> = lines.iter().enumerate().map(|(i, l)| context::Vertex {
            pos: [l[0], l[1], l[2], 1.0],
            tc: [i as f32 * ratio, 0.0],
        }).collect();
        self.context.add(context::Kind::Line, &verts, ())
    }

    pub fn add_cube(&mut self, size: [f32; 3]) -> context::Handle {
        use genmesh::generators::{Cube, SharedVertex, IndexedPolygon};
        let cube = Cube::new();
        let verts: Vec<_> = cube.shared_vertex_iter().map(|v| context::Vertex {
            pos: [v.0 * 0.5 * size[0], v.1 * 0.5 * size[1], v.2 * 0.5 * size[2], 1.0],
            tc: [0.0, 0.0], //TODO
        }).collect();
        let mut indices = Vec::<u16>::new();
        for q in cube.indexed_polygon_iter() {
            indices.extend(&[q.x as u16, q.y as u16, q.z as u16,
                q.x as u16, q.z as u16, q.w as u16]);
        }
        self.context.add(context::Kind::Mesh, &verts, indices.as_slice())
    }

    pub fn remove(&mut self, h: context::Handle) {
        self.context.remove(h)
    }
}
