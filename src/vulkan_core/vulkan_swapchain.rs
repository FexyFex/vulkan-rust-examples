use std::ptr::{null, null_mut};
use vulkan_raw::*;
use crate::util::image_extent::ImageExtent;
use crate::vulkan_core::{Device, Instance, QueueFamily};
use crate::vulkan_core::vulkan_surface::vulkan_surface::Surface;


pub struct Swapchain {
    pub handle: VkSwapchainKHR,
    pub extent: ImageExtent,
    pub images: Vec<VkImage>,
    pub image_views: Vec<VkImageView>,
    instance: Instance,
    device: Device
}
impl Swapchain {
    pub fn destroy(&self) {
        let lib1 = InstanceLevelFunctions::load_from_instance(self.instance.handle);
        let lib2 = DeviceLevelFunctions::load_from_device(&lib1, self.device.handle);
        unsafe { lib2.vkDestroySwapchainKHR(self.device.handle, self.handle, null()) };
    }
}


const PREFERRED_COLOR_SPACE: VkColorSpaceKHR = VkColorSpaceKHR::SRGB_NONLINEAR_KHR;
const PREFERRED_IMAGE_FORMAT: VkFormat = VkFormat::B8G8R8A8_SRGB;
const PREFERRED_PRESENT_MODE: VkPresentModeKHR = VkPresentModeKHR::MAILBOX_KHR;


pub fn create_swapchain(
    instance: Instance,
    surface: Surface,
    physical_device: VkPhysicalDevice,
    device: Device,
    images_total: u32,
    extent: ImageExtent,
    graphics_queue_family: QueueFamily,
    present_queue_family: QueueFamily
) -> Swapchain {
    let lib_instance = InstanceLevelFunctions::load_from_instance(instance.handle);
    let lib_device = DeviceLevelFunctions::load_from_device(&lib_instance, device.handle);

    let mut capabilities = VkSurfaceCapabilitiesKHR::default();
    unsafe { lib_instance.vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface.handle, &mut capabilities) };

    let mut format_count: u32 = 0;
    unsafe {
        lib_instance.vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface.handle,
            &mut format_count,
            null_mut()
        )
    };
    let mut formats = Vec::with_capacity(format_count as usize);
    unsafe {
        lib_instance.vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface.handle,
            &mut format_count,
            formats.as_mut_ptr()
        )
    };

    let mut present_mode_count: u32 = 0;
    unsafe {
        lib_instance.vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface.handle,
            &mut present_mode_count,
            null_mut()
        )
    };
    let mut present_modes = Vec::with_capacity(present_mode_count as usize);
    unsafe {
        lib_instance.vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface.handle,
            &mut present_mode_count,
            present_modes.as_mut_ptr()
        )
    };

    let mut queue_family_indices: Vec<u32> = Vec::new();
    let mut image_sharing_mode = VkSharingMode::EXCLUSIVE;
    if graphics_queue_family.index != present_queue_family.index {
        queue_family_indices.push(present_queue_family.index);
        queue_family_indices.push(graphics_queue_family.index);
    }

    let mut pre_transform = capabilities.currentTransform;
    if capabilities.supportedTransforms.contains(VkSurfaceTransformFlagBitsKHR::IDENTITY_BIT_KHR) {
        pre_transform = VkSurfaceTransformFlagBitsKHR::IDENTITY_BIT_KHR;
    }

    let surface_format = choose_surface_format(formats);
    let present_mode = choose_present_mode(present_modes);

    let swapchain_create_info = VkSwapchainCreateInfoKHR {
        sType: VkStructureType::SWAPCHAIN_CREATE_INFO_KHR,
        pNext: null(),
        flags: VkSwapchainCreateFlagsKHR::empty(),
        surface: surface.handle,
        minImageCount: images_total,
        imageFormat: surface_format.format,
        imageColorSpace: surface_format.colorSpace,
        imageExtent: VkExtent2D { width: extent.width, height: extent.height },
        imageArrayLayers: 1,
        imageUsage: VkImageUsageFlagBits::COLOR_ATTACHMENT_BIT | VkImageUsageFlagBits::TRANSFER_DST_BIT,
        imageSharingMode: image_sharing_mode,
        queueFamilyIndexCount: queue_family_indices.len() as u32,
        pQueueFamilyIndices: queue_family_indices.as_ptr(),
        preTransform: pre_transform,
        compositeAlpha: VkCompositeAlphaFlagBitsKHR::OPAQUE_BIT_KHR,
        presentMode: present_mode,
        clipped: VkBool32::from(true),
        oldSwapchain: VkSwapchainKHR::none()
    };

    let mut swapchain_handle = VkSwapchainKHR::none();
    unsafe { lib_device.vkCreateSwapchainKHR(device.handle, &swapchain_create_info, null(), &mut swapchain_handle) };

    let images = create_images(device, swapchain_handle, &lib_device);
    let image_views = create_image_views(device, images, surface_format.format, &lib_device);

    return Swapchain {
        handle: swapchain_handle,
        extent,
        images: images.clone(),
        image_views: image_views.clone(),
        instance,
        device
    }
}


fn choose_surface_format(formats: Vec<VkSurfaceFormatKHR>) -> VkSurfaceFormatKHR {
    for format in formats {
        if format.colorSpace == PREFERRED_COLOR_SPACE && format.format == PREFERRED_IMAGE_FORMAT {
            return format;
        }
    }
    unimplemented!()
}

fn choose_present_mode(all_modes: Vec<VkPresentModeKHR>) -> VkPresentModeKHR {
    let mut current_best_mode = VkPresentModeKHR::FIFO_KHR;
    for present_mode in all_modes {
        if (present_mode == PREFERRED_PRESENT_MODE) { return present_mode; }
        if (present_mode == VkPresentModeKHR::FIFO_RELAXED_KHR) { current_best_mode = present_mode; }
    }
    return current_best_mode;
}

fn create_images(device: Device, swapchain_handle: VkSwapchainKHR, device_lib: *const DeviceLevelFunctions) -> Vec<VkImage> {
    let mut image_count: u32 = 0;
    unsafe { (*device_lib).vkGetSwapchainImagesKHR(device.handle, swapchain_handle, &mut image_count, null_mut()) };
    let mut images = Vec::with_capacity(image_count as usize);
    unsafe { (*device_lib).vkGetSwapchainImagesKHR(device.handle, swapchain_handle, &mut image_count, images.as_mut_ptr()) };
    unsafe { images.set_len(image_count as usize) };

    return images;
}

fn create_image_views(device: Device, images: Vec<VkImage>, color_format: VkFormat, device_lib: *const DeviceLevelFunctions) -> Vec<VkImageView> {
    let mut image_views: Vec<VkImageView> = Vec::new();

    let component_mapping = VkComponentMapping {
        r: VkComponentSwizzle::IDENTITY,
        g: VkComponentSwizzle::IDENTITY,
        b: VkComponentSwizzle::IDENTITY,
        a: VkComponentSwizzle::IDENTITY,
    };

    let subresource_range = VkImageSubresourceRange {
        aspectMask: VkImageAspectFlagBits::COLOR_BIT,
        baseMipLevel: 0,
        levelCount: 1,
        baseArrayLayer: 0,
        layerCount: 1
    };

    for current_image in images {
        let image_view_info = VkImageViewCreateInfo {
            sType: VkStructureType::IMAGE_VIEW_CREATE_INFO,
            pNext: null(),
            flags: VkImageViewCreateFlagBits::empty(),
            image: current_image,
            viewType: VkImageViewType::IVT_2D,
            format: color_format,
            components: component_mapping.clone(),
            subresourceRange: subresource_range.clone()
        };

        let mut image_view_handle = VkImageView::none();
        unsafe { (*device_lib).vkCreateImageView(device.handle, &image_view_info, null(), &mut image_view_handle) };
        image_views.push(image_view_handle);
    }

    unsafe { image_views.set_len(images.len()) };

    return image_views;
}