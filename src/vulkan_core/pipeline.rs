use std::ffi::{c_void, CString};
use std::ptr::{null};
use ash::vk;


pub struct PushConstantsLayout {
    pub size_bytes: u32,
    pub offset: u32,
    pub shader_stages: vk::ShaderStageFlags
}

pub struct SpecializationConstant {
    pub id: u32,
    pub value: i32
}

pub struct VertexAttribute {
    pub location: u32,
    pub format: vk::Format,
    pub size_bytes: u32,
    pub offset: u32
}

pub struct GraphicsPipelineConfiguration {
    pub vertex_attributes: Vec<VertexAttribute>,

    pub vertex_shader_code: Vec<u32>,
    pub fragment_shader_code: Vec<u32>,

    pub color_format: vk::Format,
    pub depth_format: vk::Format,

    pub set_layouts: Vec<vk::DescriptorSetLayout>,

    pub push_constants_layout: PushConstantsLayout,

    pub spec_constants: Vec<SpecializationConstant>,

    pub primitive_topology: vk::PrimitiveTopology,
    pub polygon_mode: vk::PolygonMode,
    pub cull_mode: vk::CullModeFlags,
    pub multisampling: vk::SampleCountFlags,
    pub blend_enable: bool,
    pub depth_test: bool,
    pub depth_write: bool,
}

pub struct GraphicsPipeline {
    pub handle: vk::Pipeline,
    pub layout_handle: vk::PipelineLayout,
    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule
}

pub fn create_pipeline(device: &ash::Device, config: &GraphicsPipelineConfiguration) -> GraphicsPipeline {
    let push_constant_ranges = [vk::PushConstantRange {
        stage_flags: config.push_constants_layout.shader_stages,
        offset: config.push_constants_layout.offset,
        size: config.push_constants_layout.size_bytes,
    }];

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&config.set_layouts)
        .push_constant_ranges(&push_constant_ranges);

    let pipeline_layout_handle = unsafe {
        device.create_pipeline_layout(&pipeline_layout_create_info, None).expect("MEH")
    };

    let shader_entry_point = CString::new("main").expect("MEH");

    let vertex_shader_module = create_shader_module(device, &config.vertex_shader_code);
    let vertex_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertex_shader_module)
        .name(&shader_entry_point)
        .build();

    let fragment_shader_module = create_shader_module(device, &config.fragment_shader_code);
    let fragment_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(fragment_shader_module)
        .name(&shader_entry_point)
        .build();

    let shader_stages = [vertex_stage, fragment_stage];

    let vertex_stride: u32 = config.vertex_attributes
        .iter()
        .map(|e| e.size_bytes)
        .sum();
    let vertex_binding_description = [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: vertex_stride,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    ];
    let mut vertex_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> = Vec::new();
    for vertex_attribute in &config.vertex_attributes {
        let vertex_attrib_desc = vk::VertexInputAttributeDescription {
            location: vertex_attribute.location,
            binding: 0,
            format: vertex_attribute.format,
            offset: vertex_attribute.offset,
        };

        vertex_attribute_descriptions.push(vertex_attrib_desc);
    }

    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_binding_description)
        .vertex_attribute_descriptions(&vertex_attribute_descriptions);

    let input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(config.primitive_topology)
        .primitive_restart_enable(false);

    // These values here don't really matter. They will be overwritten by the dynamic states
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: 1024.0,
        height: 600.0,
        min_depth: 0.0,
        max_depth: 1.0,
    }];

    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 1024, height: 600 },
    }];

    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&viewports)
        .scissors(&scissors);

    let rasterization_state_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(config.polygon_mode)
        .cull_mode(config.cull_mode)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.0)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(0.0)
        .line_width(1.0)
        .build();

    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(config.multisampling)
        .sample_shading_enable(false)
        .min_sample_shading(1.0)
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false);

    let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(config.blend_enable)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ZERO)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .build()
    ];

    let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let stencil_op_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        compare_mask: 0,
        write_mask: 0,
        reference: 0,
    };
    
    let depth_stencil_state_info = vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: vk::FALSE,
        depth_write_enable: vk::FALSE,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        depth_bounds_test_enable: vk::FALSE,
        stencil_test_enable: vk::FALSE,
        front: stencil_op_state,
        back: stencil_op_state,
        max_depth_bounds: 1.0,
        min_depth_bounds: 0.0,
    };

    let dynamic_states_array = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let dynamic_states_info = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_states_array);

    let color_attachment_formats = [config.color_format];

    let mut dynamic_rendering_state_info = vk::PipelineRenderingCreateInfo::builder()
        .color_attachment_formats(&color_attachment_formats);

    // Here I tried constructing the GraphicsPipelineCreateInfo with the builder offered by ash but it also segfaults
    let pipeline_create_info_alternative = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_state_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_stencil_state_info)
        .color_blend_state(&color_blend_state_info)
        .dynamic_state(&dynamic_states_info)
        //.push_next(&mut dynamic_rendering_state_info)
        .layout(pipeline_layout_handle);

    /*
    let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: &mut dynamic_rendering_state_info as *mut _ as *mut c_void,
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: shader_stages.len() as u32,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_state_info,
        p_input_assembly_state: &input_assembly_state_info,
        p_tessellation_state: null(),
        p_viewport_state: &viewport_state_info,
        p_rasterization_state: &rasterization_state_info,
        p_multisample_state: &multisample_state_info,
        p_depth_stencil_state: &depth_stencil_state_info,
        p_color_blend_state: &color_blend_state_info,
        p_dynamic_state: &dynamic_states_info,
        layout: pipeline_layout_handle,
        render_pass: vk::RenderPass::null(), // no render pass due to dynamic rendering
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: -1,
    };
     */

    //let create_infos = [pipeline_create_info_alternative];
    let pipeline_handle = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_create_info_alternative), None).expect("MEH")
    };
    return GraphicsPipeline {
        handle: pipeline_handle[0],
        layout_handle: pipeline_layout_handle,
        vertex_shader_module,
        fragment_shader_module,
    };
}


fn create_shader_module(device: &ash::Device, shader_code: &Vec<u32>) -> vk::ShaderModule {
    let module_create_info = vk::ShaderModuleCreateInfo::builder().code(&shader_code);

    return unsafe {
        device.create_shader_module(&module_create_info, None).expect("MEH")
    };
}