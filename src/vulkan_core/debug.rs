#![allow(unused_variables)]

use std::ffi::{c_void, CStr};
use colored::Colorize;
use ash::*;
use ash::vk::{Bool32, DebugUtilsMessengerCallbackDataEXT};


pub unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void,
) -> Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);

    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "INFO".white(),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "WARNING".yellow(),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "ERROR".red(),
        _ => "".white()
    };

    println!("{}: {}", severity, message.to_str().unwrap());
    return vk::FALSE;
}