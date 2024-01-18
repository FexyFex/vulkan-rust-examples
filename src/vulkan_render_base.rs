#![allow(dead_code)]

use vulkan_raw::{DeviceLevelFunctions, InstanceLevelFunctions, VkCommandBufferLevel, VkPhysicalDevice, VkQueue, VkQueueFlagBits};
use crate::render_app::Window;
use crate::util::image_extent::ImageExtent;
use crate::vulkan_core;
use crate::vulkan_core::{create_device, create_physical_device, Device, get_unique_queue_families, Instance, QueueFamily};
use crate::vulkan_core::cmd::{CommandBuffer, CommandPool, create_command_buffer, create_command_pool};
use crate::vulkan_core::surface::vulkan_surface::{create_surface, Surface};
use crate::vulkan_core::swapchain::{create_swapchain, Swapchain};


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

    pub graphics_queue: VkQueue,
    pub compute_queue: VkQueue,
    pub present_queue: VkQueue,

    pub command_pool: CommandPool,
    pub command_buffers: Vec<CommandBuffer>,

    pub unique_queue_families: Vec<QueueFamily>,

    pub buffering_strategy: u32,
}
impl VulkanRenderBase {
    pub fn prepare_frame(&self) -> FramePreparation {
        return FramePreparation { acquire_successful: true, image_index: 0 };
    }

    pub fn submit_frame(&self, submit_data: FrameSubmitData) {
        if !submit_data.do_submit { return };
    }

    pub fn resize(&self) {

    }
}


pub fn initialize_vulkan(window: Window, buffering_strategy: u32) -> VulkanRenderBase {
    let instance = vulkan_core::create_instance();
    let surface = create_surface(instance, window);
    let physical_device = create_physical_device(instance);
    let unique_queue_families = get_unique_queue_families(instance, surface, physical_device);
    let device = create_device(instance, physical_device, &unique_queue_families);

    let graphics_queue_family = *unique_queue_families.iter().find(|q| q.flags.contains(VkQueueFlagBits::GRAPHICS_BIT)).unwrap();
    let present_queue_family = *unique_queue_families.iter().find(|q| q.present_supported).unwrap();

    let graphics_queue = get_first_queue_with_flags(instance, device, unique_queue_families.clone(), VkQueueFlagBits::GRAPHICS_BIT);
    let compute_queue = get_first_queue_with_flags(instance, device, unique_queue_families.clone(), VkQueueFlagBits::COMPUTE_BIT);
    let present_queue = get_queue(instance, device, present_queue_family.index, 0);

    let swapchain = create_swapchain(
        instance, surface, physical_device, device, buffering_strategy,
        ImageExtent { width: 800, height: 600 },
        graphics_queue_family, present_queue_family
    );

    let command_pool = create_command_pool(instance, device, graphics_queue_family);
    let mut command_buffers: Vec<CommandBuffer> = Vec::new();
    for _i in 0..buffering_strategy {
        command_buffers.push(create_command_buffer(instance, device, command_pool, VkCommandBufferLevel::PRIMARY));
    }
    unsafe { command_buffers.set_len(buffering_strategy as usize) };

    return VulkanRenderBase {
        instance, physical_device, device,
        surface, swapchain,
        graphics_queue, compute_queue, present_queue,
        command_pool, command_buffers,
        unique_queue_families,
        buffering_strategy
    };
}

pub fn get_first_queue_with_flags(instance: Instance, device: Device, queue_families: Vec<QueueFamily>, flags: VkQueueFlagBits) -> VkQueue {
    let mut queue_family_index: u32 = 0;
    let queue_index: u32 = 0;
    for queue_family in queue_families {
        if queue_family.flags.contains(flags) {
            queue_family_index = queue_family.index;
            break;
        }
    }

    return get_queue(instance, device, queue_family_index, queue_index);
}

pub fn get_queue(instance: Instance, device: Device, family_index: u32, queue_index: u32) -> VkQueue {
    let lib_instance = InstanceLevelFunctions::load_from_instance(instance.handle);
    let lib_device = DeviceLevelFunctions::load_from_device(&lib_instance, device.handle);

    let mut queue: VkQueue = VkQueue::none();
    unsafe { lib_device.vkGetDeviceQueue(device.handle, family_index, queue_index, &mut queue) };
    return queue;
}