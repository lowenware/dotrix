use super::Device;
use crate::render::Extent2D;
use crate::window::Window;
use ash::vk;

pub struct Surface {
    pub(super) loader: ash::extensions::khr::Surface,
    pub(super) vk_surface: vk::SurfaceKHR,
    pub(super) vk_surface_format: vk::SurfaceFormatKHR,
    pub(super) vk_surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub(super) vk_surface_transform: vk::SurfaceTransformFlagsKHR,
    pub(super) vk_present_mode: vk::PresentModeKHR,
    pub(super) vk_surface_resolution: vk::Extent2D,
    pub(super) images_count: u32,
    pub(super) version: u64,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.vk_surface, None);
        }
    }
}

impl Surface {
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn resolution(&self) -> Extent2D {
        Extent2D {
            width: self.vk_surface_resolution.width,
            height: self.vk_surface_resolution.height,
        }
    }

    pub(super) unsafe fn new(
        window: &Window,
        vk_instance: &ash::Instance,
        vk_entry: &ash::Entry,
    ) -> Self {
        use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

        let raw_display_handle = window.handle().raw_display_handle();
        let raw_window_handle = window.handle().raw_window_handle();
        let window_resolution = window.resolution();
        let vk_surface_resolution = vk::Extent2D {
            width: window_resolution.width,
            height: window_resolution.height,
        };

        let vk_surface = ash_window::create_surface(
            vk_entry,
            vk_instance,
            raw_display_handle,
            raw_window_handle,
            None,
        )
        .expect("Failed to create a Vulkan surface");

        let loader = ash::extensions::khr::Surface::new(vk_entry, vk_instance);

        Self {
            loader,
            vk_surface,
            vk_surface_format: vk::SurfaceFormatKHR::default(),
            vk_surface_capabilities: vk::SurfaceCapabilitiesKHR::default(),
            vk_surface_transform: vk::SurfaceTransformFlagsKHR::default(),
            vk_present_mode: vk::PresentModeKHR::default(),
            vk_surface_resolution,
            images_count: 2,
            version: 1,
        }
    }

    pub(super) fn configure(&mut self, p_device: vk::PhysicalDevice) {
        let vk_surface_format = unsafe {
            self.loader
                .get_physical_device_surface_formats(p_device, self.vk_surface)
                .unwrap()[0]
        };

        let vk_surface_capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(p_device, self.vk_surface)
                .unwrap()
        };

        let images_count = if vk_surface_capabilities.max_image_count > 0 {
            (vk_surface_capabilities.min_image_count + 1)
                .min(vk_surface_capabilities.max_image_count)
        } else {
            vk_surface_capabilities.min_image_count + 1
        };

        let vk_surface_resolution = match vk_surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D {
                // NOTE: vk_surface_resolution comes from window size
                width: self.vk_surface_resolution.width,
                height: self.vk_surface_resolution.height,
            },
            _ => vk_surface_capabilities.current_extent,
        };

        let vk_surface_transform = if vk_surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            vk_surface_capabilities.current_transform
        };

        let present_modes = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(p_device, self.vk_surface)
                .unwrap()
        };

        let vk_present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        self.vk_surface_format = vk_surface_format;
        self.vk_surface_capabilities = vk_surface_capabilities;
        self.vk_surface_transform = vk_surface_transform;
        self.vk_present_mode = vk_present_mode;
        self.images_count = images_count;
        self.vk_surface_resolution = vk_surface_resolution;
        self.images_count = images_count;
    }

    pub(super) unsafe fn get_physical_device_support(
        &self,
        p_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> bool {
        self.loader
            .get_physical_device_surface_support(p_device, queue_family_index, self.vk_surface)
            .unwrap_or(false)
    }
}
