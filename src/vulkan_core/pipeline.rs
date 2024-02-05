use std::ffi::{c_void, CString};
use std::ptr::null;
use ash::vk;
use ash::vk::{PipelineCache, Viewport};


struct PushConstantsLayout {
    pub size_bytes: u32,
    pub offset: u32,
    pub shader_stages: vk::ShaderStageFlags
}

struct SpecializationConstant {
    pub id: u32,
    pub value: i32
}

struct VertexAttribute {
    pub location: u32,
    pub format: u32,
    pub size_bytes: u32,
    pub offset: u32
}

struct GraphicsPipelineConfiguration {
    pub vertex_attributes: Vec<VertexAttribute>,

    pub vertex_shader_code: [u8],
    pub fragment_shader_code: [u8],

    pub set_layouts: Vec<vk::DescriptorSetLayout>,

    pub push_constants_layout: PushConstantsLayout,

    pub spec_constants: Vec<SpecializationConstant>,

    pub primitive_topology: vk::PrimitiveTopology,
    pub polygon_mode: vk::PolygonMode,
    pub cull_mode: vk::CullModeFlags,
    pub multisampling: vk::SampleCountFlags,
    pub blend_enable: bool
}

struct GraphicsPipeline {
    pub handle: vk::Pipeline,
    pub layout_handle: vk::PipelineLayout,
    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule
}

pub fn create_pipeline(device: &ash::Device, config: &GraphicsPipelineConfiguration) -> GraphicsPipeline {
    let push_constant_range = vk::PushConstantRange {
        stage_flags: config.push_constants_layout.shader_stages,
        offset: config.push_constants_layout.offset,
        size: config.push_constants_layout.size_bytes,
    };

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: config.set_layouts.len() as u32,
        p_set_layouts: config.set_layouts.as_ptr(),
        push_constant_range_count: 1,
        p_push_constant_ranges: &push_constant_range,
    };

    let pipeline_layout_handle = unsafe {
        device.create_pipeline_layout(&pipeline_layout_create_info, None).expect("MEH")
    };

    let mut index = 0;
    let mut spec_map: Vec<vk::SpecializationMapEntry> = Vec::new();
    let mut p_spec_data = config.spec_constants
        .iter()
        .map(|spec_const| spec_const.value)
        .collect::<[u32]>();
    for spec_const in config.spec_constants.iter() {
        let spec_entry = vk::SpecializationMapEntry { constant_id: spec_const.id, offset: 4 * index, size: 4 };
        spec_map.push(spec_entry);
        index += 1;
    }
    let spec_info = vk::SpecializationInfo {
        map_entry_count: spec_map.len() as u32,
        p_map_entries: spec_map.as_ptr(),
        data_size: p_spec_data.len() * 4,
        p_data: p_spec_data.as_mut_ptr() as *const c_void,
    };

    let shader_entry_point = unsafe { CString::new("main").unwrap() };

    let vertex_shader_module = create_shader_module(device, &config.vertex_shader_code);
    let vertex_stage = vk::PipelineShaderStageCreateInfo {
        s_type: Default::default(),
        p_next: null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::VERTEX,
        module: vertex_shader_module,
        p_name: shader_entry_point.as_ptr(),
        p_specialization_info: &spec_info,
    };

    let fragment_shader_module = create_shader_module(device, &config.fragment_shader_code);
    let fragment_stage = vk::PipelineShaderStageCreateInfo {
        s_type: Default::default(),
        p_next: null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: fragment_shader_module,
        p_name: shader_entry_point.as_ptr(),
        p_specialization_info: &spec_info,
    };

    let stages = [vertex_stage, fragment_stage];

    let vertex_stride: u32 = config.vertex_attributes
        .iter()
        .map(|e| e.size_bytes)
        .sum();
    let vertex_binding_description = vk::VertexInputBindingDescription {
        binding: 0,
        stride: vertex_stride,
        input_rate: vk::VertexInputRate::VERTEX,
    };
    let vertex_attribute_descriptions = config.vertex_attributes
        .iter()
        .map(|e| vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(e.location)
            .format(vk::Format::R32_SINT)
            .offset(e.offset)
            .build())
        .collect::<[vk::VertexInputAttributeDescription]>();

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: &vertex_binding_description,
        vertex_attribute_description_count: vertex_attribute_descriptions.len() as u32,
        p_vertex_attribute_descriptions: vertex_attribute_descriptions.as_ptr(),
    };

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: config.primitive_topology,
        primitive_restart_enable: vk::FALSE,
    };

    let viewport_state = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: &vk::Viewport::builder().build(),
        scissor_count: 1,
        p_scissors: &vk::Rect2D::builder().build(),
    };

    let rasterizer = vk::PipelineRasterizationStateCreateInfo {
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

    let multisampling = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: config.multisampling,
        sample_shading_enable: vk::TRUE,
        min_sample_shading: 0.2,
        p_sample_mask: null(),
        alpha_to_coverage_enable: vk::FALSE,
        alpha_to_one_enable: vk::FALSE,
    };

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::Bool32::from(config.blend_enable),
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    };
    
    let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: stages.as_ptr(),
        p_vertex_input_state: (),
        p_input_assembly_state: (),
        p_tessellation_state: (),
        p_viewport_state: (),
        p_rasterization_state: (),
        p_multisample_state: (),
        p_depth_stencil_state: (),
        p_color_blend_state: (),
        p_dynamic_state: (),
        layout: Default::default(),
        render_pass: Default::default(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    };

    let pipeline_handle = unsafe {
        device.create_graphics_pipelines(PipelineCache::null(), &[], None).expect("MEH")[0]
    };
    return GraphicsPipeline {
        handle: pipeline_handle,
        layout_handle: pipeline_layout_handle,
        vertex_shader_module,
        fragment_shader_module,
    };
}


fn create_shader_module(device: &ash::Device, shader_code: &[u8]) -> vk::ShaderModule {
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