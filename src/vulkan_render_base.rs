#![allow(dead_code)]

use vulkan_raw::{DeviceLevelFunctions, InstanceLevelFunctions, VkPhysicalDevice, VkQueue, VkQueueFlagBits};
use crate::render_app::Window;
use crate::util::image_extent::ImageExtent;
use crate::vulkan_core;
use crate::vulkan_core::{create_device, create_physical_device, Device, get_unique_queue_families, Instance, QueueFamily};
use crate::vulkan_core::vulkan_surface::vulkan_surface::{create_surface, Surface};
use crate::vulkan_core::vulkan_swapchain::{create_swapchain, Swapchain};


const BUFFER_STRATEGY: u32 = 3;


pub struct FramePreparation {
    pub acquire_successful: bool,
    pub image_index: u32
}

pub struct FrameSubmitData {
    pub do_submit: bool,
    pub image_index: u32
}


pub struct VulkanRenderBase {
    pub instance: Instance,
    pub physical_device: VkPhysicalDevice,
    pub device: Device,
    pub surface: Surface,
    pub swapchain: Swapchain,

    pub unique_queue_families: Vec<QueueFamily>
}
impl VulkanRenderBase {
    pub fn get_queue_with_flags(instance: Instance, device: Device, queue_families: Vec<QueueFamily>, flags: VkQueueFlagBits) -> VkQueue {
        let mut queue_family_index: u32 = 0;
        let mut queue_index: u32 = 0;
        for queue_family in queue_families {
            if queue_family.flags.contains(flags) {
                queue_family_index = queue_family.index;
                break;
            }
        }

        return VulkanRenderBase::get_queue(instance, device, queue_family_index, queue_index);
    }

    pub fn get_queue(instance: Instance, device: Device, family_index: u32, queue_index: u32) -> VkQueue {
        let lib_instance = InstanceLevelFunctions::load_from_instance(instance.handle);
        let lib_device = DeviceLevelFunctions::load_from_device(&lib_instance, device.handle);

        let mut queue: VkQueue = VkQueue::none();
        unsafe { lib_device.vkGetDeviceQueue(device.handle, family_index, queue_index, &mut queue) };
        return queue;
    }
}


pub fn initialize_vulkan(window: Window) -> VulkanRenderBase {
    let instance = vulkan_core::create_instance();
    let surface = create_surface(instance, window);
    let physical_device = create_physical_device(instance);
    let unique_queue_families = get_unique_queue_families(instance, surface, physical_device);
    let device = create_device(instance, physical_device, unique_queue_families.clone());

    let graphics_queue_family = *unique_queue_families.iter().find(|q| q.flags.contains(VkQueueFlagBits::GRAPHICS_BIT)).unwrap();
    let present_queue_family = *unique_queue_families.iter().find(|q| q.present_supported).unwrap();

    let graphics_queue = VulkanRenderBase::get_queue_with_flags(instance, device, unique_queue_families.clone(), VkQueueFlagBits::GRAPHICS_BIT);
    let compute_queue = VulkanRenderBase::get_queue_with_flags(instance, device, unique_queue_families.clone(), VkQueueFlagBits::COMPUTE_BIT);
    let present_queue = VulkanRenderBase::get_queue(instance, device, present_queue_family.index, 0);

    let swapchain = create_swapchain(
        instance, surface, physical_device, device, BUFFER_STRATEGY,
        ImageExtent { width: 800, height: 600 },
        graphics_queue_family, present_queue_family
    );

    return VulkanRenderBase {
        instance, physical_device, device,
        surface, swapchain,
        unique_queue_families,
    };
}

pub fn prepare_frame() -> FramePreparation {
    return FramePreparation { acquire_successful: true, image_index: 0 };
}

pub fn submit_frame(submit_data: FrameSubmitData) {
    if !submit_data.do_submit { return };
}