pub mod vulkan_surface {
    use std::ptr::null;
    use vulkan_raw::*;
    use crate::render_app::Window;
    use crate::vulkan_core::Instance;

    #[derive(Clone)]
    pub struct Surface {
        pub handle: VkSurfaceKHR,
        instance: Instance
    }

    impl Surface {
        pub fn destroy(&self) {
            unsafe { self.instance.lib.vkDestroyInstance(self.instance.handle, null()) };
        }
    }

    pub fn create_surface(instance: Instance, window: Window) -> Surface {
        let create_info = VkWin32SurfaceCreateInfoKHR {
            sType: VkStructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            pNext: null(),
            flags: VkWin32SurfaceCreateFlagBitsKHR::empty(),
            hinstance: window.hinstance as usize,
            hwnd: window.hwnd as usize,
        };

        let mut surface_handle = VkSurfaceKHR::none();
        unsafe { instance.lib.vkCreateWin32SurfaceKHR(instance.handle, &create_info, null(), &mut surface_handle) };

        return Surface { handle: surface_handle, instance };
    }
}