mod vulkan_debug;

use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::{null, null_mut};
use std::str::FromStr;
use vulkan_raw::*;
use crate::vulkan_core::vulkan_debug::debug_callback;


pub struct VulkanCore {
    pub instance: VkInstance,
    pub physical_device: VkPhysicalDevice,
    pub device: VkDevice
}


pub fn initialize() -> VulkanCore {
    let instance = create_instance();
    let physical_device = create_physical_device(instance);
    let device = create_device(instance);

    return VulkanCore { instance, physical_device, device }
}


fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    return (major << 22) | (minor << 12) | (patch);
}
fn make_api_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    return (variant << 29) | (major << 22) | (minor << 12) | patch;
}

fn create_instance() -> VkInstance {
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

    let layers = vec!["VK_LAYER_KHRONOS_validation"]
        .iter().map(|e| e.as_ptr() as *const c_char).collect::<Vec<_>>();

    let extensions = vec!["VK_EXT_debug_utils"]
        .iter().map(|e| e.as_ptr() as *const c_char).collect::<Vec<_>>();

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
        enabledLayerCount: layers.len() as u32,
        ppEnabledLayerNames: layers.as_ptr(),
        enabledExtensionCount: extensions.len() as u32,
        ppEnabledExtensionNames: extensions.as_ptr(),
    };

    let mut instance: VkInstance = VkInstance::none();
    unsafe { vkCreateInstance(&instance_create_info, null(), &mut instance) };

    return instance;
}


fn create_physical_device(instance: VkInstance) -> VkPhysicalDevice {
    let lib = InstanceLevelFunctions::load_from_instance(instance);

    let mut physical_devices_count: u32 = 0;
    unsafe { lib.vkEnumeratePhysicalDevices(instance, &mut physical_devices_count, null_mut()) };
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::with_capacity(physical_devices_count as usize);
    unsafe { lib.vkEnumeratePhysicalDevices(instance, &mut physical_devices_count, physical_devices.as_mut_ptr()) };
    unsafe { physical_devices.set_len(physical_devices_count as usize) }; // not sure why this is needed but it is...

    // TODO: Choose a VkPhysicalDevice based on their properties and available features
    let physical_device = physical_devices[0];

    let mut physical_device_properties: VkPhysicalDeviceProperties2 = Default::default();
    unsafe { lib.vkGetPhysicalDeviceProperties2(physical_device, &mut physical_device_properties) };
    let device_name = unsafe { CStr::from_ptr(physical_device_properties.properties.deviceName.as_ptr()) }.to_str().unwrap();
    println!("Physical Device Chosen: {}", device_name);

    return physical_device;
}


fn create_device(instance: VkInstance) -> VkDevice {
    let mut device = VkDevice::none();

    return device;
}

