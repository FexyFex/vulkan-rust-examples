use ash::vk;


pub fn create_semaphore(device: &ash::Device) -> vk::Semaphore {
    let create_info = vk::SemaphoreCreateInfo::builder();

    return unsafe { device.create_semaphore(&create_info, None).expect("MEH") };
}


pub fn create_fence(device: &ash::Device) -> vk::Fence {
    let create_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    return unsafe { device.create_fence(&create_info, None).expect("MEH") };
}

