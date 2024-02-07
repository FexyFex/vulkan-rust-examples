#![allow(dead_code)]

use std::ptr::{null, null_mut};
use ash::vk;
use ash::vk::QueueFlags;
use crate::vulkan_core;
use crate::vulkan_core::{create_device, create_physical_device, create_surface, get_unique_queue_families, QueueFamily, SurfaceInfo};
use crate::vulkan_core::buffer_factory::{VulkanBuffer, VulkanBufferConfiguration};
use crate::vulkan_core::cmd::{create_command_buffer, create_command_pool};
use crate::vulkan_core::sync::{create_fence, create_semaphore};
use crate::vulkan_core::swapchain::{create_swapchain, SwapchainInfo};


pub struct FramePreparation {
    pub acquire_successful: bool,
    pub image_index: u32
}

pub struct FrameSubmitData {
    pub do_submit: bool,
    pub image_index: u32
}


pub struct VulkanRenderBase {
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub surface: SurfaceInfo,
    pub swapchain: SwapchainInfo,

    pub memory_properties: vk::PhysicalDeviceMemoryProperties,

    pub unique_queue_families: Vec<QueueFamily>,
    pub graphics_queue_family: QueueFamily,
    pub present_queue_family: QueueFamily,
    pub graphics_queue: vk::Queue,
    pub compute_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,

    pub buffering_strategy: u32,
    pub frames_in_flight: u32,

    pub frame_index: u32,
    pub frame_in_flight_index: u32
}
impl VulkanRenderBase {
    pub fn prepare_frame(&mut self) -> FramePreparation { unsafe {
        let wait_fence = self.in_flight_fences[self.frame_in_flight_index as usize];
        let wait_fences = [wait_fence];
        self.device.wait_for_fences(&wait_fences, true, u64::MAX)
            .expect("MEH");

        let available_semaphore = self.image_available_semaphores[self.frame_in_flight_index as usize];
        let result_acquire = self.swapchain.loader
            .acquire_next_image(self.swapchain.handle, u64::MAX, available_semaphore, vk::Fence::null())
            .expect("MEH");

        let image_index: u32 = result_acquire.0;
        let out_of_date = result_acquire.1;

        if out_of_date {
            self.resize_swapchain();
            return FramePreparation { acquire_successful: false, image_index };
        }

        let reset_fence = self.in_flight_fences[self.frame_in_flight_index as usize];
        self.device.reset_fences(&[reset_fence])
            .expect("MEH");

        if !out_of_date {
            return FramePreparation { acquire_successful: true, image_index };
        }

        panic!()
    }}

    pub fn submit_frame(&mut self, submit_data: FrameSubmitData) { unsafe {
        if !submit_data.do_submit { return };

        let in_flight_index = self.frame_in_flight_index as usize;

        let wait_semaphore = self.image_available_semaphores[in_flight_index];
        let command_buffer = self.command_buffers[in_flight_index];
        let signal_semaphore = self.render_finished_semaphores[in_flight_index];
        let wait_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &wait_semaphore,
            p_wait_dst_stage_mask: &wait_stage,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: &signal_semaphore,
        };
        let submit_infos = [submit_info];

        let submit_fence = self.in_flight_fences[in_flight_index].clone();
        self.device.queue_submit(self.graphics_queue, &submit_infos, submit_fence)
            .expect("MEH");

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &signal_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.swapchain.handle,
            p_image_indices: &submit_data.image_index,
            p_results: null_mut(),
        };

        let result_present = self.swapchain.loader.queue_present(self.present_queue, &present_info)
            .expect("MEH") as i32;
        if result_present == -1000001004 /* OUT OF DATE*/ || result_present == 1000001003 /* SUBOPTIMAL_KHR */ {
            self.resize_swapchain();
        }

        self.frame_index = (self.frame_index + 1) % self.buffering_strategy;
        self.frame_in_flight_index = (self.frame_in_flight_index + 1) % self.frames_in_flight;
    }}

    pub fn create_buffer(&self, buffer_config: &VulkanBufferConfiguration) -> VulkanBuffer {
        return vulkan_core::buffer_factory::create_buffer(
            &self.device,
            &self.memory_properties,
            buffer_config
        );
    }

    pub fn resize_swapchain(&mut self) {
        unsafe { self.device.device_wait_idle().expect("MEH") };

        unsafe { self.swapchain.loader.destroy_swapchain(self.swapchain.handle, None) };
        // on resize destroy

        self.swapchain = create_swapchain(
            &self.instance,
            &self.surface,
            self.physical_device,
            &self.device,
            self.buffering_strategy,
            &self.graphics_queue_family,
            &self.present_queue_family
        );
        // on resize recreate
    }
}


pub fn initialize_vulkan(window: &winit::window::Window, buffering_strategy: u32) -> VulkanRenderBase {
    let frames_in_flight = buffering_strategy - 1;

    let entry = unsafe { ash::Entry::load().expect("Filed to initialize!") };

    let instance = vulkan_core::create_instance(&entry);
    let surface_info = create_surface(&entry, &instance, window);
    let physical_device = create_physical_device(&instance);

    let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    let unique_queue_families = get_unique_queue_families(&instance, &surface_info, physical_device);
    let device = create_device(&instance, physical_device, &unique_queue_families);

    let graphics_queue_family = *unique_queue_families.iter()
        .find(|q| q.flags.contains(QueueFlags::GRAPHICS))
        .expect("MEH");
    let present_queue_family = *unique_queue_families.iter()
        .find(|q| q.present_supported)
        .expect("MEH");

    let graphics_queue = get_first_queue_with_flags(&device, unique_queue_families.clone(), QueueFlags::GRAPHICS);
    let compute_queue = get_first_queue_with_flags(&device, unique_queue_families.clone(), QueueFlags::COMPUTE);
    let present_queue = get_queue(&device, present_queue_family.index, 0);

    let swapchain = create_swapchain(
        &instance, &surface_info, physical_device, &device, buffering_strategy,
        &graphics_queue_family, &present_queue_family
    );

    let command_pool = create_command_pool(&device, graphics_queue_family);
    let mut command_buffers: Vec<vk::CommandBuffer> = Vec::new();
    let mut image_available_semaphores: Vec<vk::Semaphore> = Vec::new();
    let mut render_finished_semaphores: Vec<vk::Semaphore> = Vec::new();
    let mut in_flight_fences: Vec<vk::Fence> = Vec::new();
    for _i in 0..frames_in_flight {
        command_buffers.push(create_command_buffer(&device, command_pool, vk::CommandBufferLevel::PRIMARY));
        image_available_semaphores.push(create_semaphore(&device));
        render_finished_semaphores.push(create_semaphore(&device));
        in_flight_fences.push(create_fence(&device));
    }
    unsafe {
        command_buffers.set_len(frames_in_flight as usize);
        image_available_semaphores.set_len(frames_in_flight as usize);
        render_finished_semaphores.set_len(frames_in_flight as usize);
        in_flight_fences.set_len(frames_in_flight as usize);
    };

    return VulkanRenderBase {
        instance, physical_device, device,
        surface: surface_info, swapchain,
        memory_properties,
        unique_queue_families, graphics_queue_family, present_queue_family,
        graphics_queue, compute_queue, present_queue,
        command_pool: command_pool.clone(), command_buffers: command_buffers.clone(),
        image_available_semaphores, render_finished_semaphores, in_flight_fences,
        buffering_strategy, frames_in_flight, frame_index: 0, frame_in_flight_index: 0
    };
}


pub fn get_first_queue_with_flags(device: &ash::Device, queue_families: Vec<QueueFamily>, flags: QueueFlags) -> vk::Queue {
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

pub fn get_queue(device: &ash::Device, family_index: u32, queue_index: u32) -> vk::Queue {
    return unsafe { device.get_device_queue(family_index, queue_index) };
}