pub mod vulkan_surface {
    use std::ffi::{CStr, CString};
    use std::ptr::null;
    use vulkan_raw::*;
    use crate::render_app::Window;
    use crate::vulkan_core::Instance;

    pub struct Surface {
        pub handle: VkSurfaceKHR,
        instance: VkInstance
    }

    impl Surface {
        pub fn destroy(&self) {
            let lib = InstanceLevelFunctions::load_from_instance(self.instance);
            unsafe { lib.vkDestroyInstance(self.instance, null()) };
        }
    }

    pub fn create_surface(instance: *const Instance, window: Window) -> Surface {
        let lib = unsafe { InstanceLevelFunctions::load_from_instance((*instance).handle) };

        let create_info = VkWin32SurfaceCreateInfoKHR {
            sType: VkStructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            pNext: null(),
            flags: VkWin32SurfaceCreateFlagBitsKHR::empty(),
            hinstance: window.hinstance as usize,
            hwnd: window.hwnd as usize,
        };

        let mut surface_handle = VkSurfaceKHR::none();
        unsafe { lib.vkCreateWin32SurfaceKHR((*instance).handle, &create_info, null(), &mut surface_handle) };

        return Surface { handle: surface_handle, instance: unsafe { (*instance).handle } };
    }
}