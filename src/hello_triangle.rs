use std::path::Path;
use std::ptr::{null, null_mut};
use ash::vk;
use crate::render_app;
use crate::vulkan_core::buffer_factory::{VulkanBuffer, VulkanBufferConfiguration};
use crate::vulkan_core::pipeline::{create_pipeline, GraphicsPipeline, GraphicsPipelineConfiguration, PushConstantsLayout};
use crate::vulkan_core::tools::read_shader_code;
use crate::vulkan_render_base::{FramePreparation, FrameSubmitData, VulkanRenderBase};


struct HelloTriangle {
    pub pipeline: GraphicsPipeline,
    pub vertex_buffer: VulkanBuffer,
}

static mut HELLO_TRIANGLE: Option<HelloTriangle> = None;

pub fn main() {
    let render_app = render_app::create_app();

    prepare_vulkan(&render_app.vulkan_base);

    render_app.main_loop(record_command_buffer);
}

fn prepare_vulkan(vulkan_base: &VulkanRenderBase) {
    // Pipeline creation
    let pipeline_config = GraphicsPipelineConfiguration {
        vertex_attributes: vec![],
        vertex_shader_code: read_shader_code(Path::new("shaders/hello_triangle/vert.spv")),
        fragment_shader_code: read_shader_code(Path::new("shaders/hello_triangle/frag.spv")),
        color_format: vulkan_base.swapchain.color_format,
        depth_format: vk::Format::D32_SFLOAT,
        set_layouts: Vec::new(),
        push_constants_layout: PushConstantsLayout {
            size_bytes: 64,
            offset: 0,
            shader_stages: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        },
        spec_constants: vec![],
        primitive_topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        multisampling: vk::SampleCountFlags::TYPE_1,
        blend_enable: false,
        depth_test: false,
        depth_write: false,
    };

    let pipeline = create_pipeline(&vulkan_base.device, &pipeline_config);

    // Vertex Buffer creation
    let vertex_buffer_config = VulkanBufferConfiguration {
        size: 64,
        memory_property_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
        buffer_usage: vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST
    };
    let vertex_buffer = vulkan_base.create_buffer(&vertex_buffer_config);

    unsafe { HELLO_TRIANGLE = Some(HelloTriangle { pipeline, vertex_buffer }) };
}

pub fn record_command_buffer(vulkan_base: &VulkanRenderBase, prep: FramePreparation) -> FrameSubmitData {
    frame_process();

    let width = vulkan_base.swapchain.extent.width;
    let height = vulkan_base.swapchain.extent.height;

    let cmd_begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next: null_mut(),
        flags: vk::CommandBufferUsageFlags::empty(),
        p_inheritance_info: null(),
    };

    let command_buffer = vulkan_base.command_buffers[vulkan_base.frame_in_flight_index as usize];
    let swapchain_image = vulkan_base.swapchain.images[prep.image_index as usize];
    let swapchain_image_view = vulkan_base.swapchain.image_views[prep.image_index as usize];

    let clear_color = vk::ClearValue { color: vk::ClearColorValue { float32: [0.2, 0.2, 0.2, 0.2] } };
    //let clear_depth = vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 0.0, stencil: 0 } };

    let color_attachment_info = vk::RenderingAttachmentInfo {
        s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
        p_next: null_mut(),
        image_view: swapchain_image_view,
        image_layout: vk::ImageLayout::ATTACHMENT_OPTIMAL,
        resolve_mode: vk::ResolveModeFlags::NONE,
        resolve_image_view: vk::ImageView::null(),
        resolve_image_layout: vk::ImageLayout::UNDEFINED,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        clear_value: clear_color,
    };

    let render_area = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width, height } };

    let rendering_info = vk::RenderingInfo {
        s_type: vk::StructureType::RENDERING_INFO,
        p_next: null_mut(),
        flags: vk::RenderingFlags::empty(),
        render_area,
        layer_count: 1,
        view_mask: 0,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_info,
        p_depth_attachment: null_mut(),
        p_stencil_attachment: null_mut(),
    };

    unsafe {
        vulkan_base.device.begin_command_buffer(command_buffer, &cmd_begin_info).expect("MEH");

        let subresource = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let swapchain_barrier_begin_render = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: null_mut(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: swapchain_image,
            subresource_range: subresource,
        };

        vulkan_base.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::DependencyFlags::empty(), &[], &[], &[swapchain_barrier_begin_render]
        );

        vulkan_base.device.cmd_begin_rendering(command_buffer, &rendering_info);

        let viewport = vk::Viewport {
            x: 0.0, y: 0.0,
            width: width as f32, height: height as f32,
            min_depth: 0.0, max_depth: 1.0,
        };

        let hello_triangle = &HELLO_TRIANGLE.as_ref().unwrap();
        let pipeline = &hello_triangle.pipeline;
        let vertex_buffer = &hello_triangle.vertex_buffer;
        let push_constants: [u8; 64] = [0; 64];

        vulkan_base.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
        vulkan_base.device.cmd_set_scissor(command_buffer, 0, &[render_area]);
        vulkan_base.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.handle);
        vulkan_base.device.cmd_push_constants(
            command_buffer, pipeline.layout_handle,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT, 0, &push_constants
        );
        vulkan_base.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.handle], &[0]);
        vulkan_base.device.cmd_draw(command_buffer, 3, 1, 0, 0);

        vulkan_base.device.cmd_end_rendering(command_buffer);

        let swapchain_barrier_begin_present = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: null_mut(),
            src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: swapchain_image,
            subresource_range: subresource,
        };

        vulkan_base.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(), &[], &[], &[swapchain_barrier_begin_present]
        );

        vulkan_base.device.end_command_buffer(command_buffer).expect("MEH");
    };

    return FrameSubmitData { do_submit: prep.acquire_successful, image_index: prep.image_index };
}


fn frame_process() {

}