use std::ffi::c_void;
use ash::vk;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceDynamicRenderingFeaturesKHR.html>"]
pub(crate) struct VkPhysicalDeviceDynamicRenderingFeatures {
    pub s_type: vk::StructureType,
    pub p_next: *const c_void,
    pub dynamic_rendering: vk::Bool32
}