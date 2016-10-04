extern crate gfx_kiss;

fn main() {
    let mut win = gfx_kiss::Window::new("Test window");
    win.background = [0.1, 0.2, 0.3, 1.0];
    let point = win.add_points(&[[0.0, 0.0, 0.0]]);
    win.with(&point, |q| q.color = [1.0, 0.0, 0.0, 1.0]);
    let line = win.add_lines(&[[-0.8, -0.6, 0.0], [0.6, 0.8, 0.0]]);
    win.with(&line, |q| q.color = [0.0, 1.0, 0.0, 1.0]);
    let cube = win.add_cube([1.0, 1.0, 1.0]);
    win.set_texture_by_path(&cube, "assets/demo.jpg");
    while win.render() {
    }
}
