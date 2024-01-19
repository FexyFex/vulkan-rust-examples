use std::ptr::{null, null_mut};
use vulkan_raw::*;
use crate::util::image_extent::ImageExtent;
use crate::vulkan_core::{Device, Instance, QueueFamily};
use crate::vulkan_core::surface::vulkan_surface::Surface;

#[derive(Clone)]
pub struct Swapchain {
    pub handle: VkSwapchainKHR,
    pub extent: ImageExtent,
    pub images: Vec<VkImage>,
    pub image_views: Vec<VkImageView>,
    device: Device
}
impl Swapchain {
    pub fn destroy(&self) {
        unsafe { self.device.lib.vkDestroySwapchainKHR(self.device.handle, self.handle, null()) };
    }
}


const PREFERRED_COLOR_SPACE: VkColorSpaceKHR = VkColorSpaceKHR::SRGB_NONLINEAR_KHR;
const PREFERRED_IMAGE_FORMATS: [VkFormat; 2] = [VkFormat::B8G8R8A8_SRGB, VkFormat::R8G8B8A8_SRGB];
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
    let mut capabilities = VkSurfaceCapabilitiesKHR::default();
    unsafe { instance.lib.vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface.handle, &mut capabilities) };

    let mut format_count: u32 = 0;
    unsafe {
        instance.lib.vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface.handle,
            &mut format_count,
            null_mut()
        )
    };
    let mut formats = Vec::with_capacity(format_count as usize);
    unsafe {
        instance.lib.vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface.handle,
            &mut format_count,
            formats.as_mut_ptr()
        )
    };
    unsafe {formats.set_len(format_count as usize) };

    let mut present_mode_count: u32 = 0;
    unsafe {
        instance.lib.vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface.handle,
            &mut present_mode_count,
            null_mut()
        )
    };
    let mut present_modes = Vec::with_capacity(present_mode_count as usize);
    unsafe {
        instance.lib.vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface.handle,
            &mut present_mode_count,
            present_modes.as_mut_ptr()
        )
    };
    unsafe { present_modes.set_len(present_mode_count as usize) };

    let mut queue_family_indices: Vec<u32> = Vec::new();
    let mut image_sharing_mode = VkSharingMode::EXCLUSIVE;
    if graphics_queue_family.index != present_queue_family.index {
        queue_family_indices.push(present_queue_family.index);
        queue_family_indices.push(graphics_queue_family.index);
        image_sharing_mode = VkSharingMode::CONCURRENT;
    }

    let mut pre_transform = capabilities.currentTransform;
    if capabilities.supportedTransforms.contains(VkSurfaceTransformFlagBitsKHR::IDENTITY_BIT_KHR) {
        pre_transform = VkSurfaceTransformFlagBitsKHR::IDENTITY_BIT_KHR;
    }

    let surface_format = choose_surface_format(&formats);
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
    unsafe { device.lib.vkCreateSwapchainKHR(device.handle, &swapchain_create_info, null(), &mut swapchain_handle) };

    let images = create_images(device.clone(), swapchain_handle);
    let image_views = create_image_views(device.clone(), &images, surface_format.format);

    return Swapchain {
        handle: swapchain_handle,
        extent,
        images,
        image_views,
        device
    }
}


fn choose_surface_format(formats: &Vec<VkSurfaceFormatKHR>) -> &VkSurfaceFormatKHR {
    for format in formats {
        if format.colorSpace == PREFERRED_COLOR_SPACE && PREFERRED_IMAGE_FORMATS.contains(&format.format) {
            return format;
        }
    }
    unimplemented!()
}

fn choose_present_mode(all_modes: Vec<VkPresentModeKHR>) -> VkPresentModeKHR {
    let mut current_best_mode = VkPresentModeKHR::FIFO_KHR;
    for present_mode in all_modes {
        if present_mode == PREFERRED_PRESENT_MODE { return present_mode; }
        if present_mode == VkPresentModeKHR::FIFO_RELAXED_KHR { current_best_mode = present_mode; }
    }
    return current_best_mode;
}

fn create_images(device: Device, swapchain_handle: VkSwapchainKHR) -> Vec<VkImage> {
    let mut image_count: u32 = 0;
    unsafe { device.lib.vkGetSwapchainImagesKHR(device.handle, swapchain_handle, &mut image_count, null_mut()) };
    let mut images = Vec::with_capacity(image_count as usize);
    unsafe { device.lib.vkGetSwapchainImagesKHR(device.handle, swapchain_handle, &mut image_count, images.as_mut_ptr()) };
    unsafe { images.set_len(image_count as usize) };

    return images;
}

fn create_image_views(device: Device, images: &Vec<VkImage>, color_format: VkFormat) -> Vec<VkImageView> {
    let mut image_views: Vec<VkImageView> = Vec::new();

    for current_image in images {
        let image_view_info = VkImageViewCreateInfo {
            sType: VkStructureType::IMAGE_VIEW_CREATE_INFO,
            pNext: null(),
            flags: VkImageViewCreateFlagBits::empty(),
            image: *current_image,
            viewType: VkImageViewType::IVT_2D,
            format: color_format,
            components: VkComponentMapping {
                r: VkComponentSwizzle::IDENTITY,
                g: VkComponentSwizzle::IDENTITY,
                b: VkComponentSwizzle::IDENTITY,
                a: VkComponentSwizzle::IDENTITY,
            },
            subresourceRange: VkImageSubresourceRange {
                aspectMask: VkImageAspectFlagBits::COLOR_BIT,
                baseMipLevel: 0,
                levelCount: 1,
                baseArrayLayer: 0,
                layerCount: 1
            }
        };

        let mut image_view_handle = VkImageView::none();
        unsafe { device.lib.vkCreateImageView(device.handle, &image_view_info, null(), &mut image_view_handle) };
        image_views.push(image_view_handle);
    }

    unsafe { image_views.set_len(images.len()) };

    return image_views;
}