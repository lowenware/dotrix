use std::sync::Arc;
use super::Device;

pub PipelineLayout {
    pub(super) device: Arc<Device>,
    pub(super) vk_pipeline_layout;
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self
                .device
                .vk_device.destroy_pipeline_layout(self.vk_pipeline_layout, None);
        }
    }
}

#[derive(Default)]
pub struct PipelineLayoutSetup {
}

impl PipelineLayoutSetup<'a> {
    pub fn set_bindings(mut self, bindings: &[Binding]) -> Self {

    };
}

pub struct GraphicsPipeline {
    pub(super) device: Arc<Device>,
    pub(super) vk_graphics_pipeline;
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self
                .device
                .vk_device.destroy_pipeline(self.vk_graphics_pipeline, None);
        }
    }
}

impl GraphicsPipeline {
    pub fn setup() -> GraphicsPipelineSetup {
        Constructor::default()
    }
}

#[derive(Default)]
pub struct GraphicsPipelineConstructor {

}

impl GraphicsPipelineConstructor {

}

