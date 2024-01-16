#![allow(dead_code)]

use crate::render_app::Window;
use crate::vulkan_core;
use crate::vulkan_core::{create_device, create_physical_device};
use crate::vulkan_core::vulkan_surface::vulkan_surface::create_surface;

pub struct FramePreparation {
    pub acquire_successful: bool,
    pub image_index: u32
}

pub struct FrameSubmitData {
    pub do_submit: bool,
    pub image_index: u32
}


pub struct VulkanRenderBase {

}


pub fn initialize_vulkan(window: Window) -> VulkanRenderBase {
    let instance = vulkan_core::create_instance();
    let _surface = create_surface(&instance, window);
    let physical_device = create_physical_device(&instance);
    let _device = create_device(&instance, physical_device);

    return VulkanRenderBase {  };
}

pub fn prepare_frame() -> FramePreparation {
    return FramePreparation { acquire_successful: true, image_index: 0 };
}

pub fn submit_frame(submit_data: FrameSubmitData) {
    if !submit_data.do_submit { return };
}

