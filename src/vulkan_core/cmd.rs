use std::ptr::null;
use vulkan_raw::*;
use crate::vulkan_core::{Device, QueueFamily};


#[derive(Clone)]
pub struct CommandPool {
    pub handle: VkCommandPool,
    device: Device
}
impl CommandPool {
    pub fn destroy(&self) {
        unsafe { self.device.lib.vkDestroyCommandPool(self.device.handle, self.handle, null()) };
    }
}


pub fn create_command_pool(device: Device, queue_family: QueueFamily) -> CommandPool {
    let create_info = VkCommandPoolCreateInfo {
        sType: VkStructureType::COMMAND_POOL_CREATE_INFO,
        pNext: null(),
        flags: VkCommandPoolCreateFlagBits::RESET_COMMAND_BUFFER_BIT,
        queueFamilyIndex: queue_family.index,
    };
    
    let mut command_pool_handle = VkCommandPool::none();
    unsafe { device.lib.vkCreateCommandPool(device.handle, &create_info, null(), &mut command_pool_handle) };

    return CommandPool { handle: command_pool_handle, device };
}


#[derive(Clone)]
pub struct CommandBuffer {
    pub handle: VkCommandBuffer,
    pub command_pool: CommandPool,
    device: Device
}
impl CommandBuffer {
    pub fn free(&self) {
        unsafe {
            self.device.lib.vkFreeCommandBuffers(
                self.device.handle,
                self.command_pool.handle,
                1,
                &self.handle
        )};
    }

    pub fn reset(&self) {
        unsafe { self.device.lib.vkResetCommandBuffer(self.handle, VkCommandBufferResetFlags::empty()) };
    }
}

pub fn create_command_buffer(device: Device, command_pool: CommandPool, level: VkCommandBufferLevel) -> CommandBuffer {
    let alloc_info = VkCommandBufferAllocateInfo {
        sType: VkStructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: null(),
        commandPool: command_pool.handle,
        level,
        commandBufferCount: 1
    };

    let mut cmdbuf_handle = VkCommandBuffer::none();
    unsafe { device.lib.vkAllocateCommandBuffers(device.handle, &alloc_info, &mut cmdbuf_handle) };
    return CommandBuffer {
        handle: cmdbuf_handle,
        command_pool,
        device
    };
}