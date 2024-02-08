#![allow(dead_code, deprecated)]

pub(crate) mod surface;
pub mod debug;
pub mod swapchain;
pub mod cmd;
pub mod descriptor;
pub mod sync;
pub mod pipeline;
pub mod render_pass;
pub mod buffer_factory;
pub mod tools;

use std::ffi::{c_char, c_void, CStr, CString};
use std::iter::Iterator;
use std::ops::BitOr;
use ash::*;
use ash::vk::{QueueFlags};
use crate::vulkan_core::debug::debug_callback;


//const REQUIRED_INSTANCE_LAYERS: [&str; 2] = ["VK_LAYER_KHRONOS_validation", "VK_LAYER_LUNARG_api_dump"];
const REQUIRED_INSTANCE_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
const REQUIRED_INSTANCE_EXTENSIONS: [&str; 5] = [
    "VK_EXT_debug_utils",
    "VK_EXT_debug_report",
    "VK_KHR_surface",
    "VK_KHR_win32_surface",
    "VK_EXT_validation_features"
];
const REQUIRED_DEVICE_EXTENSIONS: [&str; 2] = [
    "VK_KHR_swapchain",
    "VK_EXT_descriptor_indexing",
  //  "VK_KHR_dynamic_rendering",
   // "VK_KHR_depth_stencil_resolve",
  //  "VK_KHR_synchronization2"
];


pub fn create_instance(entry: &ash::Entry) -> ash::Instance {
    let application_name = CString::new("vulkan-rust-example").unwrap();
    let engine_name = CString::new("FexEngine_Rust_Variant").unwrap();

    let (major, minor) = match entry.try_enumerate_instance_version().expect("MEH") {
        // Vulkan 1.1+
        Some(version) => (
            vk::api_version_major(version),
            vk::api_version_minor(version),
        ),
        // Vulkan 1.0
        None => (1, 0),
    };

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&application_name)
        .application_version(vk::make_version(0, 0, 1))
        .engine_name(&engine_name)
        .engine_version(vk::make_version(0, 0, 1))
        .api_version(vk::make_api_version(0, major, minor, 0));

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
    let extensions = entry.enumerate_instance_extension_properties(None).expect("MEH");
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

    let message_type = vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION |
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;

    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
        .message_type(message_type)
        .pfn_user_callback(Some(debug_callback));

    let mut enabled_validation_features = [
        vk::ValidationFeatureEnableEXT::GPU_ASSISTED, vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION
    ];
    let mut validation_features_info = vk::ValidationFeaturesEXT::builder()
        .enabled_validation_features(&enabled_validation_features);
    validation_features_info.p_next = &mut debug_create_info as *mut _ as *mut c_void;

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layer_c_names)
        .enabled_extension_names(&extension_c_names)
        .push_next(&mut validation_features_info);

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
    let _device_scores = [physical_devices.len() as i32; 0];

    //for physical_device in physical_devices {
    //    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    //    let features = unsafe { instance.get_physical_device_features(physical_device) };
    //}
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
            present_supported: present_support.unwrap()
        };
        unique_queue_families.push(queue_family);
    }

    return unique_queue_families;
}


pub fn create_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, unique_queue_families: &Vec<QueueFamily>) -> ash::Device {
    let queue_count = unique_queue_families.len();
    let queue_priorities = [1.0];
    let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
    for i in 0..queue_count {
        let queue_family = unique_queue_families.get(i).unwrap();
        let create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family.index)
            .queue_priorities(&queue_priorities)
            .build();

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
    let mut extension_c_names: Vec<*const c_char> = available_extensions.iter()
        .filter(|e| unsafe { REQUIRED_DEVICE_EXTENSIONS.contains(&CStr::from_ptr(e.extension_name.as_ptr()).to_str().unwrap()) })
        .map(|e| e.extension_name.as_ptr())
        .collect::<Vec<*const c_char>>();
    unsafe { extension_c_names.set_len(REQUIRED_DEVICE_EXTENSIONS.len()) };

    let base_device_features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true)
        .multi_draw_indirect(true)
        .build();

    let mut features_vk13 = vk::PhysicalDeviceVulkan13Features::builder()
        .dynamic_rendering(true)
        .synchronization2(true);

    let mut features2 = vk::PhysicalDeviceFeatures2::builder().push_next(&mut features_vk13);

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&extension_c_names)
        .push_next(&mut features2);

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