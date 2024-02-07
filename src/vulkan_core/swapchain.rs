use std::ptr::{null};
use ash::vk;
use ash::vk::Bool32;
use crate::vulkan_core::{QueueFamily, SurfaceInfo};


pub struct SwapchainInfo {
    pub handle: vk::SwapchainKHR,
    pub loader: ash::extensions::khr::Swapchain,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub color_format: vk::Format,
}


const PREFERRED_COLOR_SPACE: vk::ColorSpaceKHR = vk::ColorSpaceKHR::SRGB_NONLINEAR;
const PREFERRED_IMAGE_FORMATS: [vk::Format; 2] = [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB];
const PREFERRED_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::MAILBOX;


pub fn create_swapchain(
    instance: &ash::Instance,
    surface: &SurfaceInfo,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    images_total: u32,
    graphics_queue_family: &QueueFamily,
    present_queue_family: &QueueFamily
) -> SwapchainInfo {
    let capabilities = unsafe {
        surface.loader.get_physical_device_surface_capabilities(physical_device, surface.handle)
    }.unwrap();

    let formats = unsafe {
        surface.loader
            .get_physical_device_surface_formats(physical_device, surface.handle)
            .expect("MEH")
    };

    let present_modes = unsafe {
        surface.loader
            .get_physical_device_surface_present_modes(physical_device, surface.handle)
            .expect("MEH")
    };

    let mut queue_family_indices: Vec<u32> = Vec::new();
    let mut image_sharing_mode = vk::SharingMode::EXCLUSIVE;
    if graphics_queue_family.index != present_queue_family.index {
        queue_family_indices.push(present_queue_family.index);
        queue_family_indices.push(graphics_queue_family.index);
        image_sharing_mode = vk::SharingMode::CONCURRENT;
    }

    let mut pre_transform = capabilities.current_transform;
    if capabilities.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
        pre_transform = vk::SurfaceTransformFlagsKHR::IDENTITY;
    }

    let surface_format = choose_surface_format(&formats);
    let present_mode = choose_present_mode(present_modes);

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface: surface.handle,
        min_image_count: images_total,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: vk::Extent2D { width: capabilities.min_image_extent.width, height: capabilities.min_image_extent.height },
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
        image_sharing_mode,
        queue_family_index_count: queue_family_indices.len() as u32,
        p_queue_family_indices: queue_family_indices.as_ptr(),
        pre_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: Bool32::from(true),
        old_swapchain: vk::SwapchainKHR::null()
    };

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain_handle = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("MEH")
    };

    let images = create_images(swapchain_handle, &swapchain_loader);
    let image_views = create_image_views(device, &images, surface_format.format);

    return SwapchainInfo {
        handle: swapchain_handle,
        loader: swapchain_loader,
        extent: capabilities.min_image_extent,
        images,
        image_views,
        color_format: surface_format.format
    }
}


fn choose_surface_format(formats: &Vec<vk::SurfaceFormatKHR>) -> &vk::SurfaceFormatKHR {
    for format in formats {
        if format.color_space == PREFERRED_COLOR_SPACE && PREFERRED_IMAGE_FORMATS.contains(&format.format) {
            return format;
        }
    }
    unimplemented!()
}

fn choose_present_mode(all_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    let mut current_best_mode = vk::PresentModeKHR::FIFO;
    for present_mode in all_modes {
        if present_mode == PREFERRED_PRESENT_MODE { return present_mode; }
        if present_mode == vk::PresentModeKHR::FIFO_RELAXED { current_best_mode = present_mode; }
    }
    return current_best_mode;
}

fn create_images(swapchain_handle: vk::SwapchainKHR, swapchain_loader: &ash::extensions::khr::Swapchain) -> Vec<vk::Image> {
    let images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain_handle)
            .expect("MEH")
    };

    return images;
}

fn create_image_views(device: &ash::Device, images: &Vec<vk::Image>, color_format: vk::Format) -> Vec<vk::ImageView> {
    let mut image_views: Vec<vk::ImageView> = Vec::new();

    for current_image in images {
        let image_view_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: null(),
            flags: vk::ImageViewCreateFlags::empty(),
            image: *current_image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: color_format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            }
        };

        let image_view_handle = unsafe {
            device
                .create_image_view(&image_view_info, None)
                .expect("MEH")
        };
        image_views.push(image_view_handle);
    }

    unsafe { image_views.set_len(images.len()) };

    return image_views;
}