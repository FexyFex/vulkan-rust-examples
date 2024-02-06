use std::ptr::null;
use ash::vk;


pub fn create_render_pass(device: &ash::Device) -> vk::RenderPass {

    let subpasses = [
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: null(),
            color_attachment_count: 0,
            p_color_attachments: null(),
            p_resolve_attachments: null(),
            p_depth_stencil_attachment: null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: null(),
        }
    ];

    let render_pass_create_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: 0,
        p_attachments: null(),
        subpass_count: subpasses.len() as u32,
        p_subpasses: subpasses.as_ptr(),
        dependency_count: 0,
        p_dependencies: null(),
    };

    return unsafe { device.create_render_pass(&render_pass_create_info, None).expect("MEH") };
}