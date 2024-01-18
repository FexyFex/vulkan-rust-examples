use winit::event::Event;
use winit::event::Event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::windows::{WindowExtWindows};
use winit::window::WindowBuilder;
use crate::util::image_extent::ImageExtent;
use crate::vulkan_render_base::{initialize_vulkan};


pub fn run_app() {
    let event_loop = EventLoop::new();
    let winit_window: &winit::window::Window = &WindowBuilder::new()
        .with_title("Vulkan Stuff")
        .with_resizable(true)
        .with_active(true)
        .with_visible(true)
        .with_transparent(false)
        .build(&event_loop).unwrap();

    let win_size = winit_window.inner_size();
    let render_area_extent = ImageExtent::new(win_size.width, win_size.height);

    let window = Window { hinstance: winit_window.hinstance(), hwnd: winit_window.hwnd(), closed: false, winit: winit_window };
    let _base = initialize_vulkan(window, 3);

    event_loop.run(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                //println!("Process here");
            }

            Event::RedrawRequested { .. } => {
                //println!("Draw here");
            }

            WindowEvent { event, .. } => match event {
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

    pub closed: bool,

    pub winit: *const winit::window::Window,
}