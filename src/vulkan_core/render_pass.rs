use std::ptr::null;
use ash::vk;


pub fn create_render_pass(device: &ash::Device) -> vk::RenderPass {

    let subpasses = [
        vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()
    ];

    let render_pass_create_info = vk::RenderPassCreateInfo::builder()
        .subpasses(&subpasses);

    return unsafe { device.create_render_pass(&render_pass_create_info, None).expect("MEH") };
}