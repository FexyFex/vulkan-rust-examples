#![allow(dead_code)]

pub(crate) mod surface;
pub mod debug;
pub mod swapchain;
mod cmd;
mod descriptor;

use std::ffi::{c_void, CStr, CString};
use std::iter::Iterator;
use std::ops::BitOr;
use std::ptr::{null, null_mut};
use vulkan_raw::*;
use crate::vulkan_core::debug::debug_callback;
use crate::vulkan_core::surface::vulkan_surface::Surface;


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

fn vk_true() -> VkBool32 { return VkBool32::from(true) }
fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    return (major << 22) | (minor << 12) | (patch);
}
fn make_api_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    return (variant << 29) | (major << 22) | (minor << 12) | patch;
}

#[derive(Copy, Clone)]
pub struct Instance {
    pub handle: VkInstance
}

impl Instance {
    fn destroy(&self) {
        let lib = InstanceLevelFunctions::load_from_instance(self.handle);
        unsafe { lib.vkDestroyInstance(self.handle, null()) };
    }
}

pub fn create_instance() -> Instance {
    let p_application_name = CString::new("vulkan-rust-example").unwrap();
    let p_engine_name = CString::new("FexEngine_Rust_Variant").unwrap();

    let app_info = VkApplicationInfo {
        sType: VkStructureType::APPLICATION_INFO,
        pNext: null(),
        pApplicationName: p_application_name.as_ptr(),
        applicationVersion: make_version(0, 0, 1),
        pEngineName: p_engine_name.as_ptr(),
        engineVersion: make_version(0, 0, 1),
        apiVersion: make_api_version(0, 1, 2, 0),
    };

    // Instance Layers
    let mut layer_count: u32 = 0;
    unsafe { vkEnumerateInstanceLayerProperties(&mut layer_count, null_mut()) };
    let mut layers: Vec<VkLayerProperties> = Vec::with_capacity(layer_count as usize);
    unsafe { vkEnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr()) };
    unsafe { layers.set_len(layer_count as usize) };
    let layer_readable_names = layers.iter()
        .map(|e| unsafe { CStr::from_ptr(e.layerName.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for layer_name in REQUIRED_INSTANCE_LAYERS {
        if !layer_readable_names.contains(&layer_name) {
            println!("MISSING INSTANCE LAYER: {}", layer_name)
        }
    }
    let layer_c_names = layers.iter()
        .filter(|l| unsafe { REQUIRED_INSTANCE_LAYERS.contains(&CStr::from_ptr(l.layerName.as_ptr()).to_str().unwrap()) })
        .map(|l| l.layerName.as_ptr())
        .collect::<Vec<_>>();

    // Instance Extensions
    let mut available_extension_count: u32 = 0;
    unsafe { vkEnumerateInstanceExtensionProperties(null(), &mut available_extension_count, null_mut()) };
    let mut extensions: Vec<VkExtensionProperties> = Vec::with_capacity(available_extension_count as usize);
    unsafe { vkEnumerateInstanceExtensionProperties(null(), &mut available_extension_count, extensions.as_mut_ptr()) };
    unsafe { extensions.set_len(available_extension_count as usize) };
    let extension_readable_names = extensions.iter()
        .map(|e| unsafe { CStr::from_ptr(e.extensionName.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for extension_name in REQUIRED_INSTANCE_EXTENSIONS {
        if !extension_readable_names.contains(&extension_name) {
            println!("MISSING INSTANCE EXTENSION: {}", extension_name)
        }
    }
    let extension_c_names = extensions.iter()
        .filter(|e| unsafe { REQUIRED_INSTANCE_EXTENSIONS.contains(&CStr::from_ptr(e.extensionName.as_ptr()).to_str().unwrap()) })
        .map(|e| e.extensionName.as_ptr())
        .collect::<Vec<_>>();

    let mut debug_create_info: VkDebugUtilsMessengerCreateInfoEXT = VkDebugUtilsMessengerCreateInfoEXT {
        sType: VkStructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        pNext: null(),
        flags: VkDebugUtilsMessengerCreateFlagBitsEXT::empty(),
        messageSeverity: VkDebugUtilsMessageSeverityFlagsEXT::WARNING_BIT_EXT | VkDebugUtilsMessageSeverityFlagBitsEXT::ERROR_BIT_EXT,
        messageType: VkDebugUtilsMessageTypeFlagsEXT::all(),
        pfnUserCallback: debug_callback,
        pUserData: null_mut(),
    };

    let instance_create_info = VkInstanceCreateInfo {
        sType: VkStructureType::INSTANCE_CREATE_INFO,
        pNext: &mut debug_create_info as *mut _ as *mut c_void,
        flags: VkInstanceCreateFlagBits::all(),
        pApplicationInfo: &app_info,
        enabledLayerCount: layer_c_names.len() as u32,
        ppEnabledLayerNames: layer_c_names.as_ptr(),
        enabledExtensionCount: extension_c_names.len() as u32,
        ppEnabledExtensionNames: extension_c_names.as_ptr(),
    };

    let mut instance_handle: VkInstance = VkInstance::none();
    unsafe { vkCreateInstance(&instance_create_info, null(), &mut instance_handle) };

    return Instance { handle: instance_handle };
}


pub fn create_physical_device(instance: Instance) -> VkPhysicalDevice {
    let lib = InstanceLevelFunctions::load_from_instance(instance.handle);

    let mut physical_devices_count: u32 = 0;
    unsafe { lib.vkEnumeratePhysicalDevices(instance.handle, &mut physical_devices_count, null_mut()) };
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::with_capacity(physical_devices_count as usize);
    unsafe { lib.vkEnumeratePhysicalDevices(instance.handle, &mut physical_devices_count, physical_devices.as_mut_ptr()) };
    unsafe { physical_devices.set_len(physical_devices_count as usize) }; // not sure why this is needed but it is...

    // to do: Choose a VkPhysicalDevice based on their properties and available features
    let physical_device = physical_devices[0];

    let mut physical_device_properties: VkPhysicalDeviceProperties2 = Default::default();
    unsafe { lib.vkGetPhysicalDeviceProperties2(physical_device, &mut physical_device_properties) };
    let device_name = unsafe { CStr::from_ptr(physical_device_properties.properties.deviceName.as_ptr()) }.to_str().unwrap();
    println!("Physical Device Chosen: {}", device_name);

    return physical_device;
}


#[derive(Copy, Clone)]
pub struct QueueFamily {
    pub index: u32,
    pub flags: VkQueueFlagBits,
    pub present_supported: bool
}

pub fn get_unique_queue_families(instance: Instance, surface: Surface, physical_device: VkPhysicalDevice)-> Vec<QueueFamily> {
    let lib = InstanceLevelFunctions::load_from_instance(instance.handle);
    let mut unique_queue_families = Vec::new();

    let mut queue_family_count: u32 = 0;
    unsafe { lib.vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, null_mut()) };
    let mut queue_families_props = Vec::with_capacity(queue_family_count as usize);
    unsafe { lib.vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families_props.as_mut_ptr()) };
    unsafe { queue_families_props.set_len(queue_family_count as usize) };

    let relevant_flags = [VkQueueFlagBits::GRAPHICS_BIT, VkQueueFlagBits::COMPUTE_BIT, VkQueueFlagBits::TRANSFER_BIT];

    for i in 0..queue_family_count {
        let properties: &VkQueueFamilyProperties = queue_families_props.get(i as usize).unwrap();
        let mut queue_flags: VkQueueFlagBits = VkQueueFlagBits::empty();

        for target_flag in relevant_flags {
            if properties.queueFlags.contains(target_flag) { queue_flags = queue_flags.bitor(target_flag) }
        }

        let mut present_support = VkBool32::from(false);
        unsafe { lib.vkGetPhysicalDeviceSurfaceSupportKHR(physical_device, i, surface.handle, &mut present_support) };
        let queue_family = QueueFamily {
            index: i,
            flags: queue_flags,
            present_supported: present_support == VkBool32::from(true)
        };
        unique_queue_families.push(queue_family);
    }

    return unique_queue_families;
}

#[derive(Copy, Clone)]
pub struct Device {
    pub handle: VkDevice,
    instance: VkInstance
}

impl Device {
    fn destroy(&self) {
        let lib_instance = InstanceLevelFunctions::load_from_instance(self.instance);
        let lib_device = DeviceLevelFunctions::load_from_device(&lib_instance, self.handle);
        unsafe { lib_device.vkDestroyDevice(self.handle, null()) };
    }
}

pub fn create_device(instance: Instance, physical_device: VkPhysicalDevice, unique_queue_families: &Vec<QueueFamily>) -> Device {
    let lib = InstanceLevelFunctions::load_from_instance(instance.handle);

    let queue_count = unique_queue_families.len();
    let queue_priority: f32 = 1.0;
    let mut queue_create_infos: Vec<VkDeviceQueueCreateInfo> = Vec::new();
    for i in 0..queue_count {
        let queue_family = unique_queue_families.get(i).unwrap();
        let create_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType::DEVICE_QUEUE_CREATE_INFO,
            pNext: null(),
            flags: VkDeviceQueueCreateFlagBits::empty(),
            queueFamilyIndex: queue_family.index,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        };

        queue_create_infos.push(create_info);
    }
    unsafe { queue_create_infos.set_len(queue_count) };

    let mut available_layer_count: u32 = 0;
    unsafe { lib.vkEnumerateDeviceLayerProperties(physical_device, &mut available_layer_count, null_mut()) };
    let mut available_layers: Vec<VkLayerProperties> = Vec::with_capacity(available_layer_count as usize);
    unsafe { lib.vkEnumerateDeviceLayerProperties(physical_device, &mut available_layer_count, available_layers.as_mut_ptr()) };
    unsafe { available_layers.set_len(available_layer_count as usize) };
    let available_layers_readable: Vec<&str> = available_layers.iter()
        .map(|l| unsafe { CStr::from_ptr(l.layerName.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for layer in REQUIRED_INSTANCE_LAYERS {
        if !available_layers_readable.contains(&layer) {
            println!("MISSING DEVICE LAYER: {}", layer);
        }
    }
    let layer_c_names = available_layers.iter()
        .filter(|l| unsafe { REQUIRED_INSTANCE_LAYERS.contains(&CStr::from_ptr(l.layerName.as_ptr()).to_str().unwrap()) })
        .map(|l| l.layerName.as_ptr())
        .collect::<Vec<_>>();

    let mut available_extension_count: u32 = 0;
    unsafe { lib.vkEnumerateDeviceExtensionProperties(physical_device, null(), &mut available_extension_count, null_mut()) };
    let mut available_extensions: Vec<VkExtensionProperties> = Vec::with_capacity(available_extension_count as usize);
    unsafe { lib.vkEnumerateDeviceExtensionProperties(physical_device, null(), &mut available_extension_count, available_extensions.as_mut_ptr()) };
    unsafe { available_extensions.set_len(available_extension_count as usize) };
    let available_extensions_readable: Vec<&str> = available_extensions.iter()
        .map(|e| unsafe { CStr::from_ptr(e.extensionName.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for extension in REQUIRED_DEVICE_EXTENSIONS {
        if !available_extensions_readable.contains(&extension) {
            println!("MISSING DEVICE EXTENSION: {}", extension);
        }
    }
    let mut extension_c_names = available_extensions.iter()
        .filter(|e| unsafe { REQUIRED_DEVICE_EXTENSIONS.contains(&CStr::from_ptr(e.extensionName.as_ptr()).to_str().unwrap()) })
        .map(|e| e.extensionName.as_ptr())
        .collect::<Vec<_>>();
    unsafe { extension_c_names.set_len(REQUIRED_DEVICE_EXTENSIONS.len()) };

    let mut base_device_features = VkPhysicalDeviceFeatures::default();
    base_device_features.samplerAnisotropy = vk_true();
    base_device_features.sampleRateShading = vk_true();
    base_device_features.multiDrawIndirect = vk_true();

    let mut features2 = VkPhysicalDeviceVulkan12Features::default();
    features2.descriptorIndexing = vk_true();
    features2.descriptorBindingPartiallyBound = vk_true();
    features2.descriptorBindingVariableDescriptorCount = vk_true();
    features2.shaderStorageBufferArrayNonUniformIndexing = vk_true();
    features2.shaderSampledImageArrayNonUniformIndexing = vk_true();
    features2.descriptorBindingSampledImageUpdateAfterBind = vk_true();
    features2.descriptorBindingStorageBufferUpdateAfterBind = vk_true();

    let device_create_info = VkDeviceCreateInfo {
        sType: VkStructureType::DEVICE_CREATE_INFO,
        pNext: &mut features2 as *mut _ as *mut c_void,
        flags: VkDeviceCreateFlags::empty(),
        queueCreateInfoCount: queue_create_infos.len() as u32,
        pQueueCreateInfos: queue_create_infos.as_ptr(),
        enabledLayerCount: layer_c_names.len() as u32,
        ppEnabledLayerNames: layer_c_names.as_ptr(),
        enabledExtensionCount: extension_c_names.len() as u32,
        ppEnabledExtensionNames: extension_c_names.as_ptr(),
        pEnabledFeatures: &base_device_features,
    };

    let mut device_handle = VkDevice::none();
    unsafe { lib.vkCreateDevice(physical_device, &device_create_info, null(), &mut device_handle) };

    return Device { handle: device_handle, instance: instance.handle };
}