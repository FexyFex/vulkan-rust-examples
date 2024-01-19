#![allow(dead_code)]

use std::ptr::{null, null_mut};
use vulkan_raw::*;
use vulkan_raw::VkResult::{ERROR_OUT_OF_DATE_KHR, SUBOPTIMAL_KHR, SUCCESS};
use crate::render_app::Window;
use crate::util::image_extent::ImageExtent;
use crate::vulkan_core;
use crate::vulkan_core::{create_device, create_physical_device, Device, get_unique_queue_families, Instance, QueueFamily};
use crate::vulkan_core::cmd::{CommandBuffer, CommandPool, create_command_buffer, create_command_pool};
use crate::vulkan_core::sync::{create_fence, create_semaphore, Fence, Semaphore};
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

    pub unique_queue_families: Vec<QueueFamily>,
    pub graphics_queue_family: QueueFamily,
    pub present_queue_family: QueueFamily,
    pub graphics_queue: VkQueue,
    pub compute_queue: VkQueue,
    pub present_queue: VkQueue,

    pub command_pool: CommandPool,
    pub command_buffers: Vec<CommandBuffer>,

    pub image_available_semaphores: Vec<Semaphore>,
    pub render_finished_semaphores: Vec<Semaphore>,
    pub in_flight_fences: Vec<Fence>,

    pub buffering_strategy: u32,
    pub frames_in_flight: u32,

    pub frame_index: u32,
    pub frame_in_flight_index: u32
}
impl VulkanRenderBase {
    pub fn prepare_frame(&mut self) -> FramePreparation { unsafe {
        let wait_fence: Fence = self.in_flight_fences[self.frame_in_flight_index as usize].clone();
        let timeout: u64 = !0;
        self.device.lib.vkWaitForFences(
            self.device.handle, 1, &wait_fence.handle,
            VkBool32::from(true), timeout
        );

        let mut image_index: u32 = !0;
        let available_semaphore: Semaphore = self.image_available_semaphores[self.frame_in_flight_index as usize].clone();
        let result_acquire = self.device.lib.vkAcquireNextImageKHR(
            self.device.handle, self.swapchain.handle,
            timeout, available_semaphore.handle, VkFence::none(),
            &mut image_index
        );

        if result_acquire == ERROR_OUT_OF_DATE_KHR {
            self.resize_swapchain();
            return FramePreparation { acquire_successful: false, image_index };
        }

        let reset_fence: Fence = self.in_flight_fences[self.frame_in_flight_index as usize].clone();
        reset_fence.reset();

        if result_acquire == SUCCESS || result_acquire == SUBOPTIMAL_KHR {
            return FramePreparation { acquire_successful: true, image_index };
        }

        unimplemented!()
    } }

    pub fn submit_frame(&mut self, submit_data: FrameSubmitData) { unsafe {
        if !submit_data.do_submit { return };

        let in_flight_index = self.frame_in_flight_index as usize;

        let wait_semaphore = self.image_available_semaphores[in_flight_index].clone();
        let command_buffer = self.command_buffers[in_flight_index].clone();
        let signal_semaphore = self.render_finished_semaphores[in_flight_index].clone();
        let wait_stage = VkPipelineStageFlags::COLOR_ATTACHMENT_OUTPUT_BIT;

        let submit_info = VkSubmitInfo {
            sType: VkStructureType::SUBMIT_INFO,
            pNext: null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: &wait_semaphore.handle,
            pWaitDstStageMask: &wait_stage,
            commandBufferCount: 1,
            pCommandBuffers: &command_buffer.handle,
            signalSemaphoreCount: 1,
            pSignalSemaphores: &signal_semaphore.handle,
        };

        let submit_fence = self.in_flight_fences[in_flight_index].clone();
        self.device.lib.vkQueueSubmit(self.graphics_queue, 1, &submit_info, submit_fence.handle);

        let present_info = VkPresentInfoKHR {
            sType: VkStructureType::PRESENT_INFO_KHR,
            pNext: null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: &signal_semaphore.handle,
            swapchainCount: 1,
            pSwapchains: &self.swapchain.handle,
            pImageIndices: &submit_data.image_index,
            pResults: null_mut(),
        };

        let result_present = self.device.lib.vkQueuePresentKHR(self.present_queue, &present_info);
        if result_present == ERROR_OUT_OF_DATE_KHR || result_present == SUBOPTIMAL_KHR {
            self.resize_swapchain();
        }

        self.frame_index = (self.frame_index + 1) % self.buffering_strategy;
        self.frame_in_flight_index = (self.frame_in_flight_index + 1) % self.frames_in_flight;
    }}

    pub fn resize_swapchain(&mut self) {
        self.device.wait_idle();

        self.swapchain.destroy();
        // on resize destroy

        self.swapchain = create_swapchain(
            self.instance.clone(),
            self.surface.clone(),
            self.physical_device,
            self.device.clone(),
            self.buffering_strategy,
            ImageExtent { width: 800, height: 600 },
            self.graphics_queue_family,
            self.present_queue_family
        );
        // on resize recreate
    }
}


pub fn initialize_vulkan(window: Window, buffering_strategy: u32) -> VulkanRenderBase {
    let frames_in_flight = buffering_strategy - 1;

    let instance = vulkan_core::create_instance();
    let surface = create_surface(instance.clone(), window);
    let physical_device = create_physical_device(instance.clone());
    let unique_queue_families = get_unique_queue_families(instance.clone(), surface.clone(), physical_device);
    let device = create_device(instance.clone(), physical_device, &unique_queue_families);

    let graphics_queue_family = *unique_queue_families.iter().find(|q| q.flags.contains(VkQueueFlagBits::GRAPHICS_BIT)).unwrap();
    let present_queue_family = *unique_queue_families.iter().find(|q| q.present_supported).unwrap();

    let graphics_queue = get_first_queue_with_flags(device.clone(), unique_queue_families.clone(), VkQueueFlagBits::GRAPHICS_BIT);
    let compute_queue = get_first_queue_with_flags(device.clone(), unique_queue_families.clone(), VkQueueFlagBits::COMPUTE_BIT);
    let present_queue = get_queue(device.clone(), present_queue_family.index, 0);

    let swapchain = create_swapchain(
        instance.clone(), surface.clone(), physical_device, device.clone(), buffering_strategy,
        ImageExtent { width: 800, height: 600 },
        graphics_queue_family, present_queue_family
    );

    let command_pool = create_command_pool(device.clone(), graphics_queue_family);
    let mut command_buffers: Vec<CommandBuffer> = Vec::new();
    let mut image_available_semaphores: Vec<Semaphore> = Vec::new();
    let mut render_finished_semaphores: Vec<Semaphore> = Vec::new();
    let mut in_flight_fences: Vec<Fence> = Vec::new();
    for _i in 0..frames_in_flight {
        command_buffers.push(create_command_buffer(device.clone(), command_pool.clone(), VkCommandBufferLevel::PRIMARY));
        image_available_semaphores.push(create_semaphore(device.clone()));
        render_finished_semaphores.push(create_semaphore(device.clone()));
        in_flight_fences.push(create_fence(device.clone()));
    }
    unsafe {
        command_buffers.set_len(frames_in_flight as usize);
        image_available_semaphores.set_len(frames_in_flight as usize);
        render_finished_semaphores.set_len(frames_in_flight as usize);
        in_flight_fences.set_len(frames_in_flight as usize);
    };

    return VulkanRenderBase {
        instance: instance.clone(), physical_device, device: device.clone(),
        surface: surface.clone(), swapchain: swapchain.clone(),
        unique_queue_families, graphics_queue_family, present_queue_family,
        graphics_queue, compute_queue, present_queue,
        command_pool: command_pool.clone(), command_buffers: command_buffers.clone(),
        image_available_semaphores, render_finished_semaphores, in_flight_fences,
        buffering_strategy, frames_in_flight, frame_index: 0, frame_in_flight_index: 0
    };
}


pub fn get_first_queue_with_flags(device: Device, queue_families: Vec<QueueFamily>, flags: VkQueueFlagBits) -> VkQueue {
    let mut queue_family_index: u32 = 0;
    let queue_index: u32 = 0;
    for queue_family in queue_families {
        if queue_family.flags.contains(flags) {
            queue_family_index = queue_family.index;
            break;
        }
    }

    return get_queue(device, queue_family_index, queue_index);
}

pub fn get_queue(device: Device, family_index: u32, queue_index: u32) -> VkQueue {
    let mut queue: VkQueue = VkQueue::none();
    unsafe { device.lib.vkGetDeviceQueue(device.handle, family_index, queue_index, &mut queue) };
    return queue;
}