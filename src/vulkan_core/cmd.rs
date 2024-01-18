use std::ptr::null;
use vulkan_raw::{DeviceLevelFunctions, InstanceLevelFunctions, VkCommandBuffer, VkCommandBufferAllocateInfo, VkCommandBufferLevel, VkCommandBufferResetFlags, VkCommandPool, VkCommandPoolCreateFlagBits, VkCommandPoolCreateInfo, VkStructureType};
use crate::vulkan_core::{Device, Instance, QueueFamily};


#[derive(Copy, Clone)]
pub struct CommandPool {
    pub handle: VkCommandPool,
    instance: Instance,
    device: Device
}
impl CommandPool {
    pub fn destroy(&self) {
        let lib1 = InstanceLevelFunctions::load_from_instance(self.instance.handle);
        let lib2 = DeviceLevelFunctions::load_from_device(&lib1, self.device.handle);
        unsafe { lib2.vkDestroyCommandPool(self.device.handle, self.handle, null()) };
    }
}


pub fn create_command_pool(instance: Instance, device: Device, queue_family: QueueFamily) -> CommandPool {
    let lib_instance = InstanceLevelFunctions::load_from_instance(instance.handle);
    let lib_device = DeviceLevelFunctions::load_from_device(&lib_instance, device.handle);
    
    let create_info = VkCommandPoolCreateInfo {
        sType: VkStructureType::COMMAND_POOL_CREATE_INFO,
        pNext: null(),
        flags: VkCommandPoolCreateFlagBits::RESET_COMMAND_BUFFER_BIT,
        queueFamilyIndex: queue_family.index,
    };
    
    let mut command_pool_handle = VkCommandPool::none();
    unsafe { lib_device.vkCreateCommandPool(device.handle, &create_info, null(), &mut command_pool_handle) };

    return CommandPool { handle: command_pool_handle, instance, device };
}


#[derive(Copy, Clone)]
pub struct CommandBuffer {
    pub handle: VkCommandBuffer,
    pub command_pool: CommandPool,
    instance: Instance,
    device: Device
}
impl CommandBuffer {
    pub fn free(&self) {
        let lib1 = InstanceLevelFunctions::load_from_instance(self.instance.handle);
        let lib2 = DeviceLevelFunctions::load_from_device(&lib1, self.device.handle);

        unsafe {
            lib2.vkFreeCommandBuffers(
                self.device.handle,
                self.command_pool.handle,
                1,
                &self.handle
        )};
    }

    pub fn reset(&self) {
        let lib1 = InstanceLevelFunctions::load_from_instance(self.instance.handle);
        let lib2 = DeviceLevelFunctions::load_from_device(&lib1, self.device.handle);

        unsafe { lib2.vkResetCommandBuffer(self.handle, VkCommandBufferResetFlags::empty()) };
    }
}

pub fn create_command_buffer(instance: Instance, device: Device, command_pool: CommandPool, level: VkCommandBufferLevel) -> CommandBuffer {
    let lib1 = InstanceLevelFunctions::load_from_instance(instance.handle);
    let lib2 = DeviceLevelFunctions::load_from_device(&lib1, device.handle);

    let alloc_info = VkCommandBufferAllocateInfo {
        sType: VkStructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: null(),
        commandPool: command_pool.handle,
        level,
        commandBufferCount: 1
    };

    let mut cmdbuf_handle = VkCommandBuffer::none();
    unsafe { lib2.vkAllocateCommandBuffers(device.handle, &alloc_info, &mut cmdbuf_handle) };
    return CommandBuffer {
        handle: cmdbuf_handle,
        command_pool,
        instance,
        device
    };
}