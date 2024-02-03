use crate::render_app;

pub fn main() {
    let render_app = render_app::run_app();
    render_app.main_loop();
}