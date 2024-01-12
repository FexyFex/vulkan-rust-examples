mod vulkan_debug;

use std::ffi::{c_char, c_void, CString};
use std::ptr::{null, null_mut};
use std::str::FromStr;
use nobs_vk as vk;
use crate::vulkan_core::vulkan_debug::debug_callback;


fn make_version(major: u32, minor: u32, patch: u32) -> u32 { return (major << 22) | (minor << 12) | (patch); }
fn make_api_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    return (variant << 29) | (major << 22) | (minor << 12) | patch;
}

pub unsafe fn create_instance() -> u64 {
    let validation_layer: CString = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
    let debug_utils_extension: CString = CString::new("VK_EXT_debug_utils").unwrap();
    let surface_extension: CString = CString::new("VK_KHR_surface").unwrap();
    let win32_surface_extension: CString = CString::new("VK_KHR_win32_surface").unwrap();

    let p_application_name = CString::new("vulkan-rust-example").unwrap();
    let p_engine_name = CString::new("FexEngine_Rust_Variant").unwrap();

    let app_info = vk::ApplicationInfo {
        sType: vk::STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: null(),
        pApplicationName: p_application_name.as_ptr(),
        applicationVersion: make_version(0, 0, 1),
        pEngineName: p_engine_name.as_ptr(),
        engineVersion: make_version(0, 0, 1),
        apiVersion: make_api_version(0, 1, 2, 0)
    };

    let arr: [i8; 256] = [0; 256];
    let mut layer_count: u32 = 0;
    vk::EnumerateInstanceLayerProperties(&mut layer_count, null_mut());
    let mut available_layers = vk::LayerProperties {
        layerName: arr,
        specVersion: 0,
        implementationVersion: 0,
        description: arr,
    };
    vk::EnumerateInstanceLayerProperties(&mut layer_count, &mut available_layers);
    let layers = vec![validation_layer]
        .iter().map(|e| e.as_ptr() as *const c_char).collect::<Vec<_>>();

    let extensions = vec![debug_utils_extension, surface_extension, win32_surface_extension]
        .iter().map(|e| e.as_ptr() as *const c_char).collect::<Vec<_>>();

    let mut debug_create_info: vk::DebugUtilsMessengerCreateInfoEXT = vk::DebugUtilsMessengerCreateInfoEXT {
        sType: vk::STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        pNext: null(),
        flags: 0,
        messageSeverity: vk::DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT | vk::DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT,
        messageType: vk::DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT | vk::DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT | vk::DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT,
        pfnUserCallback: debug_callback,
        pUserData: null_mut()
    };

    let instance_create_info = vk::InstanceCreateInfo {
        sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: &mut debug_create_info as *mut _ as *mut c_void,
        flags: 0,
        pApplicationInfo: &app_info,
        enabledLayerCount: layers.len() as u32,
        ppEnabledLayerNames: layers.as_ptr(),
        enabledExtensionCount: extensions.len() as u32,
        ppEnabledExtensionNames: extensions.as_ptr()
    };

    let instance_handle: &mut vk::Instance = &mut 0;
    vk::CreateInstance(&instance_create_info, null(), instance_handle);

    *instance_handle
}

