use super::{Device, Gpu, Surface};
use ash::vk;
use std::sync::Arc;

const EMPTY_ATTACHMENTS: &[AttachmentDescriptor] = &[];
const EMPTY_DEPENDENCIES: &[RenderPassDependency] = &[];

pub struct RenderPass {
    pub(super) device: Arc<Device>,
    pub(super) vk_render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn setup() -> RenderPassDescriptor {
        RenderPassDescriptor::default()
    }
}

pub struct RenderPassDescriptor {
    attachments: Vec<vk::AttachmentDescription>,
    depth_stencil_attachment: Option<vk::AttachmentDescription>,
    dependencies: Vec<vk::SubpassDependency>,
}

impl Default for RenderPassDescriptor {
    fn default() -> Self {
        Self {
            attachments: Vec::with_capacity(2),
            depth_stencil_attachment: None,
            dependencies: Vec::with_capacity(1),
        }
    }
}

impl RenderPassDescriptor {
    pub fn color_attachments(
        mut self,
        attachments: impl IntoIterator<Item = AttachmentDescriptor>,
    ) -> Self {
        let initial_index = self.attachments.len();
        self.attachments.extend(
            attachments
                .into_iter()
                .map(|desc| desc.vk_attachment_description),
        );
        self
    }

    pub fn depth_stencil_attachment(mut self, attachment: AttachmentDescriptor) -> Self {
        self.depth_stencil_attachment = Some(attachment.vk_attachment_description);
        self
    }

    pub fn dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = RenderPassDependency>,
    ) -> Self {
        self.dependencies = dependencies
            .into_iter()
            .map(|dep| dep.vk_subpass_dependency)
            .collect::<Vec<_>>();
        self
    }

    pub fn create(self, gpu: &Gpu) -> RenderPass {
        let RenderPassDescriptor {
            mut attachments,
            depth_stencil_attachment,
            dependencies,
        } = self;
        let device = Arc::clone(&gpu.device);
        let color_attachments_count = attachments.len();

        let color_attachments_refs = attachments
            .iter()
            .enumerate()
            .map(|(index, attachment)| vk::AttachmentReference {
                attachment: index as u32,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect::<Vec<_>>();

        let depth_attachment_ref = depth_stencil_attachment.map(|attachment| {
            attachments.push(attachment);
            vk::AttachmentReference {
                attachment: color_attachments_count as u32,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            }
        });

        let subpass = depth_attachment_ref
            .as_ref()
            .map(|depth_attachment_ref| {
                vk::SubpassDescription::builder().depth_stencil_attachment(depth_attachment_ref)
            })
            .unwrap_or_else(|| vk::SubpassDescription::builder())
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments_refs)
            .build();

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments.as_slice())
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(dependencies.as_slice())
            .build();

        let vk_render_pass = unsafe {
            device
                .vk_device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass")
        };

        /*
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: vk_surface_format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            // vk::AttachmentDescription {
            //    format: vk::Format::D16_UNORM,
            //    samples: vk::SampleCountFlags::TYPE_1,
            //    load_op: vk::AttachmentLoadOp::CLEAR,
            //    initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            //    final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            //    ..Default::default()
            // },
        ];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        // let depth_attachment_ref = vk::AttachmentReference {
        //    attachment: 1,
        //    layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        // };
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            // .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build();

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            // .dependencies(&dependencies)
            .build();

        let renderpass = unsafe {
            vk_device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };

        let framebuffers =
            unsafe { Vulkan::create_framebuffers(&vk_device, &surface, &swapchain, &renderpass) };
        */
        RenderPass {
            device,
            vk_render_pass,
        }
    }
}

#[derive(Default, Clone)]
#[repr(C)]
pub struct AttachmentDescriptor {
    vk_attachment_description: vk::AttachmentDescription,
}

impl AttachmentDescriptor {
    pub fn surface(surface: &Surface) -> Self {
        Self {
            vk_attachment_description: vk::AttachmentDescription {
                format: surface.vk_surface_format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
        }
    }

    pub fn depth_stencil_default() -> Self {
        Self {
            vk_attachment_description: vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        }
    }

    // pub fn format(mut self, format: gpu::Format) -> Self {
    //    self.vk_attachment_description.format = format.into();
    //    self
    // }

    pub fn clear_on_load(mut self, value: bool) -> Self {
        self.vk_attachment_description.load_op = if value {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        };
        self
    }

    pub fn store(mut self) -> Self {
        self.vk_attachment_description.store_op = vk::AttachmentStoreOp::STORE;
        self
    }

    pub fn samples(mut self, samples_count: u32) -> Self {
        self.vk_attachment_description.samples = match samples_count {
            1 => vk::SampleCountFlags::TYPE_1,
            2 => vk::SampleCountFlags::TYPE_2,
            4 => vk::SampleCountFlags::TYPE_4,
            8 => vk::SampleCountFlags::TYPE_8,
            16 => vk::SampleCountFlags::TYPE_16,
            32 => vk::SampleCountFlags::TYPE_32,
            64 => vk::SampleCountFlags::TYPE_64,
            _ => panic!(
                "Unsupported smaples count {}, use one of: 1, 2, 4, 8, 16, 32, 64",
                samples_count
            ),
        };
        self
    }
}

#[derive(Clone)]
pub struct RenderPassDependency {
    vk_subpass_dependency: vk::SubpassDependency,
}

impl Default for RenderPassDependency {
    fn default() -> Self {
        Self {
            vk_subpass_dependency: vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            },
        }
    }
}
