use std::ptr::null;
use vulkan_raw::{DeviceLevelFunctions, InstanceLevelFunctions, VkSemaphore, VkSemaphoreCreateFlagBits, VkSemaphoreCreateInfo, VkStructureType};
use crate::vulkan_core::{Device, Instance};

pub struct Semaphore {
    pub handle: VkSemaphore,
    instance: Instance,
    device: Device
}
impl Semaphore {
    pub fn destroy(&self) {
        let lib1 = InstanceLevelFunctions::load_from_instance(self.instance.handle);
        let lib2 = DeviceLevelFunctions::load_from_device(&lib1, self.device.handle);

        unsafe { lib2.vkDestroySemaphore(self.device.handle, self.handle, null()) };
    }
}


pub fn create_semaphore(instance: Instance, device: Device) -> Semaphore {
    let lib_instance = InstanceLevelFunctions::load_from_instance(instance.handle);
    let lib_device = DeviceLevelFunctions::load_from_device(&lib_instance, device.handle);

    let create_info = VkSemaphoreCreateInfo {
        sType: VkStructureType::SEMAPHORE_CREATE_INFO,
        pNext: null(),
        flags: VkSemaphoreCreateFlagBits::empty(),
    };

    let mut semaphore_handle = VkSemaphore::none();
    unsafe { lib_device.vkCreateSemaphore(device.handle, &create_info, null(), &mut semaphore_handle) };
    return Semaphore {
        handle: semaphore_handle,
        instance, device
    }
}