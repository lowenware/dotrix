use super::{Device, Gpu};
use ash::vk;
use std::sync::Arc;

pub struct ShaderModule {
    pub(super) device: Arc<Device>,
    pub(super) vk_shader_module: vk::ShaderModule,
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .destroy_shader_module(self.vk_shader_module, None);
        }
    }
}

impl ShaderModule {
    pub fn setup<'a>() -> Constructor<'a> {
        Constructor::default()
    }
}

#[derive(Default)]
pub struct Constructor<'a> {
    vk_shader_module_create_info: vk::ShaderModuleCreateInfo,
    phantom_data: std::marker::PhantomData<&'a ()>,
}

impl<'a> Constructor<'a> {
    pub fn create(self, gpu: &Gpu) -> ShaderModule {
        let device = Arc::clone(&gpu.device);
        let vk_shader_module = unsafe {
            device
                .vk_device
                .create_shader_module(&self.vk_shader_module_create_info, None)
                .expect("Failed to create shader module")
        };
        ShaderModule {
            device,
            vk_shader_module,
        }
    }

    pub fn spv_code(mut self, spv_code: &'a [u32]) -> Self {
        self.vk_shader_module_create_info.code_size = spv_code.len() * std::mem::size_of::<u32>();
        self.vk_shader_module_create_info.p_code = spv_code.as_ptr();
        self
    }
}
