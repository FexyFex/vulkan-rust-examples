use winit::event::Event;
use winit::event::Event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::vulkan_render_base::{FramePreparation, FrameSubmitData, initialize_vulkan, VulkanRenderBase};


type RecordCommandBufferFunc = fn(vulkan_base: &VulkanRenderBase, frame_preparation: FramePreparation) -> FrameSubmitData;

pub struct RenderApp {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
    pub vulkan_base: VulkanRenderBase,
}

impl RenderApp {
    pub fn main_loop(mut self, record_cmd_function: RecordCommandBufferFunc) {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    self.window.request_redraw()
                }

                Event::RedrawRequested { .. } => {
                    let prep = self.vulkan_base.prepare_frame();
                    let submit = record_cmd_function(&self.vulkan_base, prep);
                    self.vulkan_base.submit_frame(submit);
                }

                Event::LoopDestroyed => {
                    unsafe { self.vulkan_base.device.device_wait_idle().expect("MEH") };
                }

                WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        /*
                        match input.virtual_keycode.unwrap() {
                            winit::event::VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                            _ => ()
                        }
                         */
                    }
                    _ => ()
                }
                _ => ()
            };
        });
    }
}

pub fn create_app() -> RenderApp {
    let event_loop = EventLoop::new();
    let winit_window: winit::window::Window = WindowBuilder::new()
        .with_title("Vulkan Stuff")
        .with_resizable(true)
        .with_visible(true)
        .with_transparent(false)
        .build(&event_loop).unwrap();

    println!("PID: {}", std::process::id());
    let base = initialize_vulkan(&winit_window, 3);

    return RenderApp { event_loop, window: winit_window, vulkan_base: base };
}