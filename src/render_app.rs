use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::window::WindowBuilder;
use crate::vulkan_core::create_instance;


pub fn run_app() {
    let window = unsafe { Window::new() };
    let mut app = unsafe { RenderApp::new(window) };
}


struct RenderApp {
    window: Window
}
impl RenderApp {
    unsafe fn new(p_window: Window) -> Self {
        window = p_window;
        let _instance: u64 = create_instance();

        return Self;
    }

    unsafe fn render(&mut self, _window: &Window) {}

    unsafe fn destroy(&mut self) {

    }
}


pub struct Window{
    pub window_handle: RawWindowHandle,
    pub display_handle: RawDisplayHandle,
}
impl Window {
    unsafe fn new() -> Window {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    println!("Closing the Application");
                    elwt.exit();
                },
                Event::AboutToWait => {
                    window.request_redraw();
                },
                _ => ()
            }
        }).unwrap();

        window_handle = window.raw_window_handle().unwrap();
        display_handle = window.raw_display_handle().unwrap();

        Ok(Self)
    }
}