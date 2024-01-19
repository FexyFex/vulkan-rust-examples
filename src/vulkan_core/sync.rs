use std::ptr::null;
use vulkan_raw::{VkFence, VkFenceCreateFlagBits, VkFenceCreateInfo, VkSemaphore, VkSemaphoreCreateFlagBits, VkSemaphoreCreateInfo, VkStructureType};
use crate::vulkan_core::{Device};

#[derive(Clone)]
pub struct Semaphore {
    pub handle: VkSemaphore,
    device: Device
}
impl Semaphore {
    pub fn destroy(&self) {
        unsafe { self.device.lib.vkDestroySemaphore(self.device.handle, self.handle, null()) };
    }
}

pub fn create_semaphore(device: Device) -> Semaphore {
    let create_info = VkSemaphoreCreateInfo {
        sType: VkStructureType::SEMAPHORE_CREATE_INFO,
        pNext: null(),
        flags: VkSemaphoreCreateFlagBits::empty(),
    };

    let mut semaphore_handle = VkSemaphore::none();
    unsafe { device.lib.vkCreateSemaphore(device.handle, &create_info, null(), &mut semaphore_handle) };
    return Semaphore { handle: semaphore_handle, device };
}


#[derive(Clone)]
pub struct Fence {
    pub handle: VkFence,
    device: Device
}
impl Fence {
    pub fn reset(&self) {
        unsafe { self.device.lib.vkResetFences(self.device.handle, 1, &self.handle) };
    }

    pub fn destroy(&self) {
        unsafe { self.device.lib.vkDestroyFence(self.device.handle, self.handle, null()) };
    }
}

pub fn create_fence(device: Device) -> Fence {
    let create_info = VkFenceCreateInfo {
        sType: VkStructureType::FENCE_CREATE_INFO,
        pNext: null(),
        flags: VkFenceCreateFlagBits::SIGNALED_BIT,
    };

    let mut fence_handle = VkFence::none();
    unsafe { device.lib.vkCreateFence(device.handle, &create_info, null(), &mut fence_handle) };
    return Fence { handle: fence_handle, device };
}

