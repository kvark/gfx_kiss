extern crate gfx_kiss;

fn main() {
    let mut win = gfx_kiss::Window::new("Test window");
    win.background = [0.1, 0.2, 0.3, 1.0];
    while win.render() {
    }
}