use super::RenderPass;
use crate::log;
use crate::render::vk;
use crate::render::{CommandRecorder, Framebuffers, Semaphore};
use crate::{Any, Display, Frame, Gpu, Ref, Task};

pub struct Renderer {
    gpu: Gpu,
    wait_semaphores: Vec<Semaphore>,
    signal_semaphore: Semaphore,
    framebuffers: Framebuffers,
    command_pool: vk::CommandPool,
    setup_command_buffer: vk::CommandBuffer,
    draw_command_buffer: vk::CommandBuffer,
    setup_command_buffer_reuse_fence: vk::Fence,
    draw_command_buffer_reuse_fence: vk::Fence,
    render_pass: vk::RenderPass,
    surface_version: u64,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gpu.destroy_command_pool(self.command_pool);
            self.gpu.destroy_render_pass(self.render_pass);
            self.gpu
                .destroy_fence(self.setup_command_buffer_reuse_fence);
            self.gpu.destroy_fence(self.draw_command_buffer_reuse_fence);
            self.framebuffers.destroy(&self.gpu);
        }
    }
}

impl Task for Renderer {
    type Context = (Any<Frame>, Ref<Display>);
    type Output = RenderPass;

    fn run(&mut self, (frame, display): Self::Context) -> Self::Output {
        log::debug!("pbr: begin");

        if let Some(surface_version) = display.surface_changed(self.surface_version) {
            unsafe {
                self.framebuffers.rebuild(&display, self.render_pass);
            };
            self.surface_version = surface_version;
        }

        unsafe {
            self.execute_render_pass(&frame);
            self.submit_draw_commands();
        }

        log::debug!("pbr: submit_command_buffer");
        RenderPass {}
    }
}

impl Renderer {
    pub fn setup() -> RendererSetup {
        RendererSetup::default()
    }

    pub fn new(gpu: Gpu, setup: RendererSetup) -> Self {
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(gpu.queue_family_index())
            .build();
        let command_pool = unsafe { gpu.create_command_pool(&pool_create_info) };
        let framebuffers = Framebuffers::new();

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(2)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        let (setup_command_buffer, draw_command_buffer) = unsafe {
            gpu.allocate_command_buffers(&command_buffer_allocate_info)
                .into()
        };

        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        let setup_command_buffer_reuse_fence = unsafe { gpu.create_fence(&fence_create_info) };
        let draw_command_buffer_reuse_fence = unsafe { gpu.create_fence(&fence_create_info) };

        let signal_semaphore = gpu.create_semaphore();
        let wait_semaphores = setup.wait_semaphores;

        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: setup.surface_format,
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
            .dependencies(&dependencies)
            .build();

        let render_pass = unsafe { gpu.create_render_pass(&renderpass_create_info) };

        Self {
            gpu,
            command_pool,
            wait_semaphores,
            signal_semaphore,
            setup_command_buffer,
            draw_command_buffer,
            setup_command_buffer_reuse_fence,
            draw_command_buffer_reuse_fence,
            render_pass,
            framebuffers,
            surface_version: 0,
        }
    }

    pub fn complete_semaphore(&self) -> &Semaphore {
        &self.signal_semaphore
    }

    unsafe fn execute_render_pass(&self, frame: &Frame) {
        let recorder = CommandRecorder::setup()
            .command_buffer(self.draw_command_buffer)
            .reuse_fence(Some(self.draw_command_buffer_reuse_fence))
            .one_time_submit(true)
            .create(&self.gpu);

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.1, 0.0],
                },
            },
            //    vk::ClearValue {
            //        depth_stencil: vk::ClearDepthStencilValue {
            //            depth: 1.0,
            //            stencil: 0,
            //        },
            //    },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers.get(frame.swapchain_index))
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: frame.resolution.width,
                    height: frame.resolution.height,
                },
            })
            .clear_values(&clear_values)
            .build();

        recorder.begin_render_pass(&render_pass_begin_info, vk::SubpassContents::INLINE);

        recorder.end_render_pass();
    }

    unsafe fn submit_draw_commands(&self) {
        let (wait_semaphores, wait_dst_stage_mask): (Vec<_>, Vec<_>) = self
            .wait_semaphores
            .iter()
            .map(|s| {
                (
                    *s.vk_semaphore(),
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                )
            })
            .unzip();
        let signal_semaphores = [*self.signal_semaphore.vk_semaphore()];
        let command_buffers = [self.draw_command_buffer];
        let submits = [vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores.as_slice())
            .wait_dst_stage_mask(wait_dst_stage_mask.as_slice())
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build()];

        log::debug!(
            "buffers: {}, wait: {}, signal: {}, deps: {}",
            command_buffers.len(),
            wait_semaphores.len(),
            signal_semaphores.len(),
            wait_dst_stage_mask.len(),
        );
        self.gpu
            .submit_queue(&submits, self.draw_command_buffer_reuse_fence);
    }
}

#[derive(Default)]
pub struct RendererSetup {
    wait_semaphores: Vec<Semaphore>,
    surface_format: vk::Format,
}

impl RendererSetup {
    pub fn wait_semaphores(mut self, semaphores: impl IntoIterator<Item = Semaphore>) -> Self {
        self.wait_semaphores.extend(semaphores);
        self
    }

    pub fn surface_format(mut self, surface_format: vk::Format) -> Self {
        self.surface_format = surface_format;
        self
    }

    pub fn create(self, gpu: Gpu) -> Renderer {
        Renderer::new(gpu, self)
    }
}
