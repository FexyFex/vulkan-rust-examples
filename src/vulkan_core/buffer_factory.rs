use ash::vk;
use crate::vulkan_core::tools::find_memory_type_index;


pub struct VulkanBufferConfiguration {
    pub size: vk::DeviceSize,
    pub memory_property_flags: vk::MemoryPropertyFlags,
    pub buffer_usage: vk::BufferUsageFlags
}

pub struct VulkanBuffer {
    pub handle: vk::Buffer,
    pub memory: vk::DeviceMemory
}

pub fn create_buffer(
    device: &ash::Device,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    config: &VulkanBufferConfiguration
) -> VulkanBuffer { unsafe {
    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(config.size)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .usage(config.buffer_usage);

    let buffer_handle = device.create_buffer(&buffer_create_info, None).expect("MEH");

    let memory_requirements = device.get_buffer_memory_requirements(buffer_handle);
    let memory_type_index = find_memory_type_index(
        memory_requirements, memory_properties, config.memory_property_flags
    );

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(config.size)
        .memory_type_index(memory_type_index);

    let buffer_memory_handle = device.allocate_memory(&alloc_info, None).expect("MEH");

    device.bind_buffer_memory(buffer_handle, buffer_memory_handle, 0).expect("MEH");

    return VulkanBuffer {
        handle: buffer_handle,
        memory: buffer_memory_handle
    };
} }