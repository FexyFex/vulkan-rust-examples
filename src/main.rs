mod vulkan_core;
mod render_app;
mod vulkan_render_base;
mod math;
mod util;
mod hello_triangle;


fn main() {
    //let b = include_bytes!("shaders/test.vert");
    //println!("{}", String::from_utf8_lossy(b));

    hello_triangle::main();
}