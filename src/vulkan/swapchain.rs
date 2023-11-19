use super::{Device, Surface};
use ash::vk;

pub(super) struct Swapchain {
    pub(super) loader: ash::extensions::khr::Swapchain,
    pub(super) vk_swapchain: vk::SwapchainKHR,
    pub(super) vk_present_images: Vec<vk::Image>,
    pub(super) vk_present_image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub(super) unsafe fn create(device: &Device, surface: &Surface) -> Swapchain {
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.vk_surface)
            .min_image_count(surface.images_count)
            .image_color_space(surface.vk_surface_format.color_space)
            .image_format(surface.vk_surface_format.format)
            .image_extent(surface.vk_surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface.vk_surface_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(surface.vk_present_mode)
            .clipped(true)
            .image_array_layers(1)
            .build();

        let loader = ash::extensions::khr::Swapchain::new(&device.vk_instance, &device.vk_device);

        let vk_swapchain = loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create a Vulkan swapchain");

        let vk_present_images = unsafe { loader.get_swapchain_images(vk_swapchain).unwrap() };

        let vk_present_image_views: Vec<vk::ImageView> = vk_present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface.vk_surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);

                unsafe {
                    device
                        .vk_device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                }
            })
            .collect();

        Swapchain {
            loader,
            vk_swapchain,
            vk_present_images,
            vk_present_image_views,
        }
    }

    pub(super) unsafe fn destroy(&self, device: &Device) {
        for &image_view in self.vk_present_image_views.iter() {
            device.vk_device.destroy_image_view(image_view, None);
        }
        self.loader.destroy_swapchain(self.vk_swapchain, None);
    }
}
