use std::ffi::c_void;
use nobs_vk::{Bool32, DebugUtilsMessageSeverityFlagBitsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT};


pub extern "system" fn debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagBitsEXT,
    message_types: DebugUtilsMessageTypeFlagsEXT,
    p_data: *const DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void
) -> Bool32 {
    println!("meep");
    return Bool32::from(false);
}