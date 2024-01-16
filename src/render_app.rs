use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::windows::{WindowExtWindows};
use winit::window::WindowBuilder;


static DESIRED_FPS: f64 = 144.0;


pub fn run_app() {
    let event_loop = EventLoop::new();
    let winit_window = &WindowBuilder::new()
        .with_title("Vulkan Stuff")
        .with_resizable(true)
        .build(&event_loop).unwrap();

    let window = Window { hinstance: winit_window.hinstance(), hwnd: winit_window.hwnd(), closed: false };

    event_loop.run(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    destroy();
                    *control_flow = ControlFlow::ExitWithCode(0);
                }

                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    match input.virtual_keycode.unwrap() {
                        winit::event::VirtualKeyCode::Escape => *control_flow = ControlFlow::ExitWithCode(0),
                        _ => ()
                    }
                }
                _ => ()
            }
            _ => ()
        };
    });
}

fn destroy() {
    println!("Goodbye");
}


pub struct Window{
    pub hinstance: isize,
    pub hwnd: isize,

    pub closed: bool
}