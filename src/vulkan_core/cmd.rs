use std::ptr::null;
use ash::version::DeviceV1_0;
use ash::vk;
use crate::vulkan_core::{QueueFamily};


pub fn create_command_pool(device: &ash::Device, queue_family: QueueFamily) -> vk::CommandPool {
    let create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: null(),
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: queue_family.index,
    };
    
    return unsafe { device.create_command_pool(&create_info, None).expect("MEH") } ;
}


pub fn create_command_buffer(device: &ash::Device, command_pool: vk::CommandPool, level: vk::CommandBufferLevel) -> vk::CommandBuffer {
    let alloc_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: null(),
        command_pool,
        level,
        command_buffer_count: 1
    };

    return unsafe { device.allocate_command_buffers(&alloc_info).expect("MEH")[0] };
}