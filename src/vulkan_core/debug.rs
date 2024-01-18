#![allow(unused_variables)]

use std::ffi::{c_void, CStr};
use colored::Colorize;
use vulkan_raw::*;


pub extern "C" fn debug_callback(
    message_severity: VkDebugUtilsMessageSeverityFlagBitsEXT,
    message_types: VkDebugUtilsMessageTypeFlagsEXT,
    p_data: *const VkDebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void
) -> VkBool32 {
    let p_message = unsafe { (*p_data).pMessage };
    let message = unsafe { CStr::from_ptr(p_message) }.to_string_lossy();

    let severity = match message_severity {
        VkDebugUtilsMessageSeverityFlagBitsEXT::INFO_BIT_EXT => "INFO".white(),
        VkDebugUtilsMessageSeverityFlagBitsEXT::WARNING_BIT_EXT => "WARNING".yellow(),
        VkDebugUtilsMessageSeverityFlagBitsEXT::ERROR_BIT_EXT => "ERROR".red(),
        _ => "".white()
    };

    println!("{}: {}", severity, message);
    return VkBool32::FALSE;
}