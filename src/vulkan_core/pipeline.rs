use std::ffi::{c_void, CString};
use std::ptr::{null, null_mut};
use ash::vk;
use ash::vk::GraphicsPipelineCreateInfo;


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

    pub vertex_shader_code: Vec<u8>,
    pub fragment_shader_code: Vec<u8>,

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

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: config.set_layouts.len() as u32,
        p_set_layouts: config.set_layouts.as_ptr(),
        push_constant_range_count: push_constant_ranges.len() as u32,
        p_push_constant_ranges: push_constant_ranges.as_ptr(),
    };

    let pipeline_layout_handle = unsafe {
        device.create_pipeline_layout(&pipeline_layout_create_info, None).expect("MEH")
    };

    let mut index = 0;
    let mut spec_map: Vec<vk::SpecializationMapEntry> = Vec::new();
    let mut p_spec_data: Vec<i32> = Vec::new();
    for spec_const in config.spec_constants.iter() {
        let spec_entry = vk::SpecializationMapEntry { constant_id: spec_const.id, offset: 4 * index, size: 4 };
        spec_map.push(spec_entry);
        p_spec_data.push(spec_const.value);
        index += 1;
    }
    let spec_info = vk::SpecializationInfo {
        map_entry_count: spec_map.len() as u32,
        p_map_entries: spec_map.as_ptr(),
        data_size: p_spec_data.len() * 4,
        p_data: p_spec_data.as_mut_ptr() as *const c_void,
    };

    let shader_entry_point = CString::new("main").unwrap();

    let vertex_shader_module = create_shader_module(device, &config.vertex_shader_code);
    let vertex_stage = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::VERTEX,
        module: vertex_shader_module,
        p_name: shader_entry_point.as_ptr(),
        p_specialization_info: if config.spec_constants.is_empty() { null() } else { &spec_info },
    };

    let fragment_shader_module = create_shader_module(device, &config.fragment_shader_code);
    let fragment_stage = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: fragment_shader_module,
        p_name: shader_entry_point.as_ptr(),
        p_specialization_info: if config.spec_constants.is_empty() { null() } else { &spec_info },
    };

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

    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: if config.vertex_attributes.is_empty() { 0 } else { 1 },
        p_vertex_binding_descriptions: vertex_binding_description.as_ptr(),
        vertex_attribute_description_count: vertex_attribute_descriptions.len() as u32,
        p_vertex_attribute_descriptions: vertex_attribute_descriptions.as_ptr(),
    };

    let input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: config.primitive_topology,
        primitive_restart_enable: vk::FALSE,
    };

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

    let viewport_state_info = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: viewports.as_ptr(),
        scissor_count: 1,
        p_scissors: scissors.as_ptr(),
    };

    let rasterization_state_info = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: config.polygon_mode,
        cull_mode: config.cull_mode,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: vk::FALSE,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        line_width: 1.0,
    };

    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: config.multisampling,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 0.0,
        p_sample_mask: null(),
        alpha_to_coverage_enable: vk::FALSE,
        alpha_to_one_enable: vk::FALSE,
    };

    let color_blend_attachments = [vk::PipelineColorBlendAttachmentState {
        blend_enable: if config.blend_enable { vk::TRUE } else { vk::FALSE },
        src_color_blend_factor: vk::BlendFactor::ONE,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];

    let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: 1,
        p_attachments: color_blend_attachments.as_ptr(),
        blend_constants: [0.0, 0.0, 0.0, 0.0],
    };

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

    let dynamic_state_array = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let dynamic_states_info = vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: dynamic_state_array.len() as u32,
        p_dynamic_states: dynamic_state_array.as_ptr(),
    };

    let color_attachment_formats = [vk::Format::B8G8R8A8_SRGB];

    let mut dynamic_rendering_state_info = vk::PipelineRenderingCreateInfo {
        s_type: vk::StructureType::PIPELINE_RENDERING_CREATE_INFO,
        p_next: null(),
        view_mask: 0,
        color_attachment_count: color_attachment_formats.len() as u32,
        p_color_attachment_formats: color_attachment_formats.as_ptr(),
        depth_attachment_format: vk::Format::D32_SFLOAT,
        stencil_attachment_format: vk::Format::UNDEFINED,
    };

    // Here I tried constructing the GraphicsPipelineCreateInfo with the builder offered by ash but it also segfaults
    let pipeline_create_info_alternative = vk::GraphicsPipelineCreateInfo::builder()
        .flags(vk::PipelineCreateFlags::empty())
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_state_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_stencil_state_info)
        .color_blend_state(&color_blend_state_info)
        .dynamic_state(&dynamic_states_info)
        .layout(pipeline_layout_handle)
        .render_pass(vk::RenderPass::null()) // no render pass due to dynamic rendering
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::null())
        .base_pipeline_index(-1)
        .push_next(&mut dynamic_rendering_state_info)
        .build();

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: &dynamic_rendering_state_info as *const vk::PipelineRenderingCreateInfo as *const c_void,
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

    let create_infos = [pipeline_create_info];
    //let create_infos = [pipeline_create_info_alternative];
    let pipeline_handle = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &create_infos, None).expect("MEH")
    };
    return GraphicsPipeline {
        handle: pipeline_handle[0],
        layout_handle: pipeline_layout_handle,
        vertex_shader_module,
        fragment_shader_module,
    };
}


fn create_shader_module(device: &ash::Device, shader_code: &Vec<u8>) -> vk::ShaderModule {
    let module_create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: shader_code.len(),
        p_code: shader_code.as_ptr() as *const u32,
    };

    return unsafe {
        device.create_shader_module(&module_create_info, None).expect("MEH")
    };
}