#![allow(unused_variables)]

use std::ffi::{c_void, CStr};
use nobs_vk as vk;
use nobs_vk::{Bool32, DebugUtilsMessageSeverityFlagBitsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT};


pub extern "system" fn debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagBitsEXT,
    message_types: DebugUtilsMessageTypeFlagsEXT,
    p_data: *const DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void
) -> Bool32 {
    let data = unsafe { *p_data };
    let message = unsafe { CStr::from_ptr(data.pMessage) }.to_string_lossy();

    let severity = match message_severity {
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT => "INFO",
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT => "WARNING",
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT => "ERROR",
        _ => ""
    };

    println!("{}: {}", severity, message);
    return Bool32::from(false);
}