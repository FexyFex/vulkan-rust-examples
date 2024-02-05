use std::ptr::null;
use ash::vk;


pub fn create_semaphore(device: &ash::Device) -> vk::Semaphore {
    let create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next: null(),
        flags: vk::SemaphoreCreateFlags::empty(),
    };

    return unsafe { device.create_semaphore(&create_info, None).expect("MEH") };
}


pub fn create_fence(device: &ash::Device) -> vk::Fence {
    let create_info = vk::FenceCreateInfo {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: null(),
        flags: vk::FenceCreateFlags::SIGNALED,
    };

    return unsafe { device.create_fence(&create_info, None).expect("MEH") };
}

