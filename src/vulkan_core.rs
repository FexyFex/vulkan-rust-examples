#![allow(dead_code)]

pub(crate) mod vulkan_surface;
mod vulkan_debug;

use std::ffi::{c_void, CStr, CString};
use std::ptr::{null, null_mut};
use vulkan_raw::*;
use crate::vulkan_core::vulkan_debug::debug_callback;


const REQUIRED_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
const REQUIRED_EXTENSIONS: [&str; 3] = ["VK_EXT_debug_utils", "VK_KHR_surface", "VK_KHR_win32_surface"];


fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    return (major << 22) | (minor << 12) | (patch);
}
fn make_api_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    return (variant << 29) | (major << 22) | (minor << 12) | patch;
}


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

    let mut layer_count: u32 = 0;
    unsafe { vkEnumerateInstanceLayerProperties(&mut layer_count, null_mut()) };
    let mut layers: Vec<VkLayerProperties> = Vec::with_capacity(layer_count as usize);
    unsafe { vkEnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr()) };
    unsafe { layers.set_len(layer_count as usize) };
    let layer_readable_names = layers.iter()
        .map(|e| unsafe { CStr::from_ptr(e.layerName.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for layer_name in REQUIRED_LAYERS {
        if !layer_readable_names.contains(&layer_name) { println!("MISSING LAYER: {}", layer_name) }
    }
    let layer_c_names = layers.iter()
        .filter(|e| REQUIRED_LAYERS.contains(&unsafe { CStr::from_ptr(e.layerName.as_ptr()).to_str().unwrap() }))
        .map(|e| e.layerName.as_ptr())
        .collect::<Vec<_>>();

    let mut extension_count: u32 = 0;
    unsafe { vkEnumerateInstanceExtensionProperties(null(), &mut extension_count, null_mut()) };
    let mut extensions: Vec<VkExtensionProperties> = Vec::with_capacity(extension_count as usize);
    unsafe { vkEnumerateInstanceExtensionProperties(null(), &mut extension_count, extensions.as_mut_ptr()) };
    let extension_readable_names = extensions.iter()
        .map(|e| unsafe { CStr::from_ptr(e.extensionName.as_ptr()).to_str().unwrap() })
        .collect::<Vec<_>>();
    for extension_name in REQUIRED_EXTENSIONS {
        if !extension_readable_names.contains(&extension_name) { println!("MISSING EXTENSION: {}", extension_name) }
    }
    let extension_c_names = extensions.iter()
        .filter(|e| REQUIRED_EXTENSIONS.contains(&unsafe { CStr::from_ptr(e.extensionName.as_ptr()).to_str().unwrap() }))
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

    let layer_count = layers.len() as u32;
    let extension_count = extensions.len() as u32;
    let instance_create_info = VkInstanceCreateInfo {
        sType: VkStructureType::INSTANCE_CREATE_INFO,
        pNext: &mut debug_create_info as *mut _ as *mut c_void,
        flags: VkInstanceCreateFlagBits::all(),
        pApplicationInfo: &app_info,
        enabledLayerCount: layer_count,
        ppEnabledLayerNames: layer_c_names.as_ptr(),
        enabledExtensionCount: extension_count,
        ppEnabledExtensionNames: extension_c_names.as_ptr(),
    };

    let mut instance_handle: VkInstance = VkInstance::none();
    unsafe { vkCreateInstance(&instance_create_info, null(), &mut instance_handle) };

    return Instance { handle: instance_handle };
}


pub fn create_physical_device(instance: *const Instance) -> VkPhysicalDevice {
    let lib = unsafe { InstanceLevelFunctions::load_from_instance((*instance).handle) };

    let mut physical_devices_count: u32 = 0;
    unsafe { lib.vkEnumeratePhysicalDevices((*instance).handle, &mut physical_devices_count, null_mut()) };
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::with_capacity(physical_devices_count as usize);
    unsafe { lib.vkEnumeratePhysicalDevices((*instance).handle, &mut physical_devices_count, physical_devices.as_mut_ptr()) };
    unsafe { physical_devices.set_len(physical_devices_count as usize) }; // not sure why this is needed but it is...

    // TODO: Choose a VkPhysicalDevice based on their properties and available features
    let physical_device = physical_devices[0];

    let mut physical_device_properties: VkPhysicalDeviceProperties2 = Default::default();
    unsafe { lib.vkGetPhysicalDeviceProperties2(physical_device, &mut physical_device_properties) };
    let device_name = unsafe { CStr::from_ptr(physical_device_properties.properties.deviceName.as_ptr()) }.to_str().unwrap();
    println!("Physical Device Chosen: {}", device_name);

    return physical_device;
}


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

pub fn create_device(instance: *const Instance, physical_device: VkPhysicalDevice) -> Device {
    let lib = unsafe { InstanceLevelFunctions::load_from_instance((*instance).handle) };

    let queue_count = 1;
    let queue_priority: f32 = 1.0;
    let mut queue_create_infos: Vec<VkDeviceQueueCreateInfo> = Vec::with_capacity(queue_count);
    for i in 0..queue_count {
        let create_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType::DEVICE_QUEUE_CREATE_INFO,
            pNext: null(),
            flags: VkDeviceQueueCreateFlagBits::empty(),
            queueFamilyIndex: 0,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        };

        queue_create_infos[i] = create_info
    }
    unsafe { queue_create_infos.set_len(queue_count) };

    let device_create_info = VkDeviceCreateInfo {
        sType: VkStructureType::DEVICE_CREATE_INFO,
        pNext: null(),
        flags: VkDeviceCreateFlags::empty(),
        queueCreateInfoCount: queue_count as u32,
        pQueueCreateInfos: queue_create_infos.as_ptr(),
        enabledLayerCount: 0,
        ppEnabledLayerNames: null(),
        enabledExtensionCount: 0,
        ppEnabledExtensionNames: null(),
        pEnabledFeatures: null(),
    };

    let mut device_handle = VkDevice::none();
    unsafe { lib.vkCreateDevice(physical_device, &device_create_info, null(), &mut device_handle) };

    return Device { handle: device_handle, instance: unsafe { (*instance).handle } };
}

