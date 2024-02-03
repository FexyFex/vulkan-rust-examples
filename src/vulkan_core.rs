#![allow(dead_code)]

pub(crate) mod surface;
pub mod debug;
pub mod swapchain;
pub mod cmd;
pub mod descriptor;
pub mod sync;
pub mod pipeline;
pub mod structs;

use std::ffi::{c_void, CStr, CString};
use std::iter::Iterator;
use std::ops::BitOr;
use std::ptr::{null, null_mut};
use ash::*;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk::{QueueFlags};
use crate::vulkan_core::debug::debug_callback;


const REQUIRED_INSTANCE_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
const REQUIRED_INSTANCE_EXTENSIONS: [&str; 4] = [
    "VK_EXT_debug_utils",
    "VK_EXT_debug_report",
    "VK_KHR_surface",
    "VK_KHR_win32_surface"
];
const REQUIRED_DEVICE_EXTENSIONS: [&str; 3] = [
    "VK_KHR_swapchain",
    "VK_EXT_descriptor_indexing",
    "VK_KHR_dynamic_rendering"
];

fn vk_true() -> vk::Bool32 { return vk::Bool32::from(true) }
fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    return (major << 22) | (minor << 12) | (patch);
}
fn make_api_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    return (variant << 29) | (major << 22) | (minor << 12) | patch;
}


pub fn create_instance(entry: &ash::Entry) -> ash::Instance {
    let p_application_name = CString::new("vulkan-rust-example").unwrap();
    let p_engine_name = CString::new("FexEngine_Rust_Variant").unwrap();

    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: null(),
        p_application_name: p_application_name.as_ptr(),
        application_version: make_version(0, 0, 1),
        p_engine_name: p_engine_name.as_ptr(),
        engine_version: make_version(0, 0, 1),
        api_version: make_api_version(0, 1, 2, 0),
    };

    // Instance Layers
    let layers = entry.enumerate_instance_layer_properties().expect("MEH");
    let layer_readable_names = layers.iter()
        .map(|e| unsafe { CStr::from_ptr(e.layer_name.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for layer_name in REQUIRED_INSTANCE_LAYERS {
        if !layer_readable_names.contains(&layer_name) {
            println!("MISSING INSTANCE LAYER: {}", layer_name)
        }
    }
    let layer_c_names = layers.iter()
        .filter(|l| unsafe { REQUIRED_INSTANCE_LAYERS.contains(&CStr::from_ptr(l.layer_name.as_ptr()).to_str().unwrap()) })
        .map(|l| l.layer_name.as_ptr())
        .collect::<Vec<_>>();

    // Instance Extensions
    let extensions = entry.enumerate_instance_extension_properties().expect("MEH");
    let extension_readable_names = extensions.iter()
        .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for extension_name in REQUIRED_INSTANCE_EXTENSIONS {
        if !extension_readable_names.contains(&extension_name) {
            println!("MISSING INSTANCE EXTENSION: {}", extension_name)
        }
    }
    let extension_c_names = extensions.iter()
        .filter(|e| unsafe { REQUIRED_INSTANCE_EXTENSIONS.contains(&CStr::from_ptr(e.extension_name.as_ptr()).to_str().unwrap()) })
        .map(|e| e.extension_name.as_ptr())
        .collect::<Vec<_>>();

    let mut debug_create_info: vk::DebugUtilsMessengerCreateInfoEXT = vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::all(),
        pfn_user_callback: Some(debug_callback),
        p_user_data: null_mut(),
    };

    let instance_create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: &mut debug_create_info as *mut _ as *mut c_void,
        flags: vk::InstanceCreateFlags::all(),
        p_application_info: &app_info,
        enabled_layer_count: layer_c_names.len() as u32,
        pp_enabled_layer_names: layer_c_names.as_ptr(),
        enabled_extension_count: extension_c_names.len() as u32,
        pp_enabled_extension_names: extension_c_names.as_ptr(),
    };

    let instance_handle = unsafe {
        entry
            .create_instance(&instance_create_info, None)
            .expect("Instance creation failed!")
    };

    return instance_handle;
}


pub fn create_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe { instance.enumerate_physical_devices().expect("MEH") };

    // to do: Choose a VkPhysicalDevice based on their properties and available features
    let physical_device = physical_devices[0];

    let physical_device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let device_name = unsafe { CStr::from_ptr(physical_device_properties.device_name.as_ptr()) }.to_str().unwrap();
    println!("Physical Device Chosen: {}", device_name);

    return physical_device;
}


#[derive(Clone, Copy)]
pub struct QueueFamily {
    pub index: u32,
    pub flags: QueueFlags,
    pub present_supported: bool
}

pub fn get_unique_queue_families(instance: &ash::Instance, surface: &SurfaceInfo, physical_device: vk::PhysicalDevice)-> Vec<QueueFamily> {
    let mut unique_queue_families = Vec::new();

    let queue_families_props = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let relevant_flags = [QueueFlags::GRAPHICS, QueueFlags::COMPUTE, QueueFlags::TRANSFER];

    for i in 0..queue_families_props.len() {
        let properties: &vk::QueueFamilyProperties = queue_families_props.get(i).unwrap();
        let mut queue_flags: QueueFlags = QueueFlags::empty();

        for target_flag in relevant_flags {
            if properties.queue_flags.contains(target_flag) { queue_flags = queue_flags.bitor(target_flag) }
        }

        let present_support = unsafe {
            surface.loader.get_physical_device_surface_support(physical_device, i as u32, surface.handle)
        };
        let queue_family = QueueFamily {
            index: i as u32,
            flags: queue_flags,
            present_supported: present_support
        };
        unique_queue_families.push(queue_family);
    }

    return unique_queue_families;
}


pub fn create_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, unique_queue_families: &Vec<QueueFamily>) -> ash::Device {
    let queue_count = unique_queue_families.len();
    let queue_priority: f32 = 1.0;
    let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
    for i in 0..queue_count {
        let queue_family = unique_queue_families.get(i).unwrap();
        let create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_family.index,
            queue_count: 1,
            p_queue_priorities: &queue_priority,
        };

        queue_create_infos.push(create_info);
    }
    unsafe { queue_create_infos.set_len(queue_count) };

    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("MEH")
    };
    let available_extensions_readable: Vec<&str> = available_extensions.iter()
        .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for extension in REQUIRED_DEVICE_EXTENSIONS {
        if !available_extensions_readable.contains(&extension) {
            println!("MISSING DEVICE EXTENSION: {}", extension);
        }
    }
    let mut extension_c_names = available_extensions.iter()
        .filter(|e| unsafe { REQUIRED_DEVICE_EXTENSIONS.contains(&CStr::from_ptr(e.extension_name.as_ptr()).to_str().unwrap()) })
        .map(|e| e.extension_name.as_ptr())
        .collect::<Vec<_>>();
    unsafe { extension_c_names.set_len(REQUIRED_DEVICE_EXTENSIONS.len()) };

    let base_device_features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true)
        .multi_draw_indirect(true)
        .build();

    let mut dynamic_rendering = structs::VkPhysicalDeviceDynamicRenderingFeatures {
        s_type: vk::StructureType::from_raw(1000044003),
        p_next: null(),
        dynamic_rendering: vk::Bool32::from(true)
    };

    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: &mut dynamic_rendering as *mut _ as *mut c_void,
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        enabled_layer_count: 0,
        pp_enabled_layer_names: null(),
        enabled_extension_count: extension_c_names.len() as u32,
        pp_enabled_extension_names: extension_c_names.as_ptr(),
        p_enabled_features: &base_device_features,
    };

    let device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("MEH")
    };

    return device;
}


pub struct SurfaceInfo {
    pub handle: vk::SurfaceKHR,
    pub loader: ash::extensions::khr::Surface
}
pub fn create_surface(entry: &Entry, instance: &ash::Instance, window: &winit::window::Window) -> SurfaceInfo {
    let surface = unsafe {
        surface::create_surface(entry, instance, window).expect("Failed to create Surface!")
    };
    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

    return SurfaceInfo { handle: surface, loader: surface_loader };
}