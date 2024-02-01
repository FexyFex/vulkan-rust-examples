use winit::event::Event;
use winit::event::Event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::vulkan_render_base::{initialize_vulkan};


struct RenderApp {
    window: winit::window::Window
}

impl RenderApp {
    pub fn main_loop(self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    self.window.request_redraw()
                }

                Event::RedrawRequested { .. } => {
                    //println!("Draw here");
                }

                Event::LoopDestroyed => {
                    // device wait idle!
                }

                WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        destroy();
                        *control_flow = ControlFlow::Exit;
                    }

                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode.unwrap() {
                            winit::event::VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                            _ => ()
                        }
                    }
                    _ => ()
                }
                _ => ()
            };
        });
    }
}

pub fn run_app() {
    let event_loop = EventLoop::new();
    let winit_window: winit::window::Window = WindowBuilder::new()
        .with_title("Vulkan Stuff")
        .with_resizable(true)
        .with_visible(true)
        .with_transparent(false)
        .build(&event_loop).unwrap();

    let _base = initialize_vulkan(&winit_window, 3);

    let render_app = RenderApp { window: winit_window };
    render_app.main_loop(event_loop);
}

fn destroy() {
    println!("Goodbye");
}