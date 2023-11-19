use crate::log;
use ash::vk;
use std::sync::Arc;

use super::{Device, Framebuffers, RenderPass};

pub struct CommandPool {
    pub(super) device: Arc<Device>,
    pub(super) vk_command_pool: vk::CommandPool,
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        log::debug!("!!!!!!!!!!! CommandPool::drop()");
        unsafe {
            self.device
                .vk_device
                .destroy_command_pool(self.vk_command_pool, None);
        }
    }
}

impl CommandPool {
    pub fn allocate_buffers(&self, buffers_to_allocate: u32) -> CommandBufferIter {
        // TODO: return iterator instead to avoid vector allocation
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(buffers_to_allocate)
            .command_pool(self.vk_command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        CommandBufferIter {
            device: Arc::clone(&self.device),
            inner: unsafe {
                self.device
                    .vk_device
                    .allocate_command_buffers(&command_buffer_allocate_info)
                    .expect("Failed to allocate command buffers")
                    .into_iter()
            },
        }
    }
}

pub struct CommandBufferIter {
    device: Arc<Device>,
    inner: std::vec::IntoIter<vk::CommandBuffer>,
}

impl CommandBufferIter {
    pub fn unpack<T: CommandBuffersTuple>(self) -> T {
        T::unpack(self)
    }
}

impl Iterator for CommandBufferIter {
    type Item = CommandBuffer;

    fn next(&mut self) -> Option<Self::Item> {
        let device = Arc::clone(&self.device);
        self.inner
            .next()
            .map(|vk_command_buffer| CommandBuffer::new(device, vk_command_buffer))
    }
}

pub trait CommandBuffersTuple {
    fn unpack(iter: CommandBufferIter) -> Self;
}

impl CommandBuffersTuple for CommandBuffer {
    fn unpack(mut iter: CommandBufferIter) -> Self {
        iter.next()
            .expect("Command buffer was not allocated: 1 of 1")
    }
}

impl CommandBuffersTuple for (CommandBuffer, CommandBuffer) {
    fn unpack(mut iter: CommandBufferIter) -> Self {
        (
            iter.next()
                .expect("Command buffer was not allocated: 2 of 2"),
            iter.next()
                .expect("Command buffer was not allocated: 1 of 2"),
        )
    }
}

impl CommandBuffersTuple for (CommandBuffer, CommandBuffer, CommandBuffer) {
    fn unpack(mut iter: CommandBufferIter) -> Self {
        (
            iter.next()
                .expect("Command buffer was not allocated: 3 of 3"),
            iter.next()
                .expect("Command buffer was not allocated: 2 of 3"),
            iter.next()
                .expect("Command buffer was not allocated: 1 of 3"),
        )
    }
}

pub struct CommandBuffer {
    pub(super) device: Arc<Device>,
    pub(super) vk_command_buffer: vk::CommandBuffer,
    pub(super) vk_reuse_fence: vk::Fence,
}

impl CommandBuffer {
    fn new(device: Arc<Device>, vk_command_buffer: vk::CommandBuffer) -> Self {
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        let vk_reuse_fence = unsafe {
            device
                .vk_device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.")
        };

        Self {
            device,
            vk_command_buffer,
            vk_reuse_fence,
        }
    }
}

impl CommandBuffer {
    pub fn recorder<'a>(&'a mut self) -> CommandRecorder<'a> {
        // begin buffer
        let reset_buffer = true;
        unsafe {
            self.device
                .vk_device
                .wait_for_fences(&[self.vk_reuse_fence], true, u64::MAX)
                .expect("Wait for fence failed.");

            self.device
                .vk_device
                .reset_fences(&[self.vk_reuse_fence])
                .expect("Reset fences failed");

            if reset_buffer {
                self.device
                    .vk_device
                    .reset_command_buffer(
                        self.vk_command_buffer,
                        vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                    )
                    .expect("Reset command buffer failed.");
            }

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            self.device
                .vk_device
                .begin_command_buffer(self.vk_command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer");
        }

        // record buffer
        CommandRecorder {
            command_buffer: self,
        }

        // end command buffer call is implemented on Drop
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .destroy_fence(self.vk_reuse_fence, None);
        }
    }
}

pub struct CommandRecorder<'a> {
    command_buffer: &'a mut CommandBuffer,
}

impl<'a> Drop for CommandRecorder<'a> {
    fn drop(&mut self) {
        unsafe {
            self.command_buffer
                .device
                .vk_device
                .end_command_buffer(self.command_buffer.vk_command_buffer)
                .expect("End commandbuffer");
        }
    }
}

impl<'a> CommandRecorder<'a> {
    pub fn begin_render_pass(
        &self,
        render_pass: &RenderPass,
        framebuffers: &Framebuffers,
        present_index: u32,
    ) {
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
            .render_pass(render_pass.vk_render_pass)
            .framebuffer(framebuffers.vk_framebuffers[present_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: framebuffers.resolution.width,
                    height: framebuffers.resolution.height,
                },
            })
            .clear_values(&clear_values)
            .build();

        unsafe {
            self.command_buffer.device.vk_device.cmd_begin_render_pass(
                self.command_buffer.vk_command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.command_buffer
                .device
                .vk_device
                .cmd_end_render_pass(self.command_buffer.vk_command_buffer);
        }
    }

    pub fn bind_pipeline(&self) {
        todo!("implement bind_pipeline");
    }

    pub fn set_viewport(&self) {
        todo!("implement set_viewport");
    }

    pub fn set_scissors(&self) {
        todo!("implement set_scissors");
    }

    pub fn bind_vertex_buffers(&self) {
        todo!("implement bind_vertex_buffers");
    }

    pub fn bind_index_buffer(&self) {
        todo!("implement bind_index_buffer");
    }

    pub fn draw_indexed(&self) {
        todo!("implement draw_indexed");
    }
}
