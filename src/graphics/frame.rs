use ash::vk;

use super::{Display, Extent2D, FramePresenter, Gpu, RenderSubmit};
use crate::log;
use crate::tasks::{All, Any, Mut, OutputChannel, Ref, Take, Task};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame Data
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    /// Current FPS
    pub fps: f32,
    /// Time passed since last frame
    pub delta: std::time::Duration,
    /// Timestamp of the frame
    pub timestamp: std::time::Instant,
    /// Frame resolution
    pub resolution: Extent2D,
    /// Absolute frame number
    pub number: u64,
    /// Frame's swapchain index
    pub swapchain_index: u32,
    /// Frame scale factor
    pub scale_factor: f32,
    /// Shows if frame was resized
    pub resized: bool,
}

/// Task, responsible for frame creation
pub struct CreateFrame {
    frame_counter: u64,
    fps_request: Option<f32>,
    log_fps_interval: Option<Duration>,
    log_fps_timestamp: Instant,
    last_frame: Option<Instant>,
    frames_duration: VecDeque<Duration>,
}

impl Default for CreateFrame {
    fn default() -> Self {
        Self {
            frame_counter: 0,
            fps_request: None,
            log_fps_interval: None,
            log_fps_timestamp: Instant::now(),
            last_frame: None,
            frames_duration: VecDeque::with_capacity(60),
        }
    }
}

impl CreateFrame {
    /// Sets interval for FPS logging, if None, it won't be logged
    pub fn log_fps_interval(mut self, interval: Option<Duration>) -> Self {
        self.log_fps_interval = interval;
        self
    }

    /// Sets FPS limit
    pub fn fps_request(mut self, fps_request: Option<f32>) -> Self {
        self.fps_request = fps_request;
        self
    }
}

impl Task for CreateFrame {
    type Context = (Mut<Display>,);
    type Output = Frame;

    fn run(&mut self, (mut display,): Self::Context) -> Self::Output {
        log::debug!("CreateFrame::run() -> begin");
        let frame_number = self.frame_counter + 1;
        let now = Instant::now();
        let delta = self
            .last_frame
            .replace(now)
            .map(|i| i.elapsed())
            .unwrap_or_else(|| Duration::from_secs_f32(1.0 / self.fps_request.unwrap_or(60.0)));

        log::debug!("CreateFrame::run() -> delta: {:?}", delta);
        // resize surface before aquiring of the new frame
        let mut resized = false;
        log::debug!("CreateFrame::run() -> display.surface_resize_request()");
        // NOTE: on iOS winit.inner_size() is not possible to use in a thread. We need a different
        // surface size / resize solution
        // if display.surface_resize_request() {
        if self.last_frame.is_none() {
            log::debug!("surface resized");
            display.resize_surface();
            resized = true;
        }
        log::debug!("CreateFrame::run() -> display.surface_resolution()");
        let surface_resolution = display.surface_resolution();

        log::debug!("CreateFrame::run() -> begin display.next_frame()");
        let swapchain_index = display.next_frame();
        log::debug!("CreateFrame::run() -> end display.next_frame()");
        // TODO: scale factor comes from a window, so shall it be taken from `Window` instance?
        let scale_factor = 1.0;

        // update frames durations buffer
        if self.frames_duration.len() == self.frames_duration.capacity() {
            self.frames_duration.pop_back();
        }

        self.frames_duration.push_front(delta);

        // calculate fps
        let frames = self.frames_duration.len() as f32;
        let duration: f32 = self.frames_duration.iter().map(|d| d.as_secs_f32()).sum();
        let fps = frames / duration;

        self.frame_counter = frame_number;

        if let Some(log_fps_interval) = self.log_fps_interval.as_ref().cloned() {
            if self.log_fps_timestamp.elapsed() > log_fps_interval {
                log::info!("FPS: {:.02}", fps);
                self.log_fps_timestamp = Instant::now();
            }
        }

        let frame = Frame {
            fps,
            delta,
            timestamp: now,
            resolution: surface_resolution,
            number: frame_number,
            swapchain_index,
            scale_factor,
            resized,
        };
        log::debug!("CreateFrame::run() -> {:?}", frame);
        frame
    }
}

/// Task, responsible for frame submition to be presented
pub struct SubmitFrame {
    gpu: Gpu,
    surface_version: u64,
    command_pool: vk::CommandPool,
    command_buffer_render: vk::CommandBuffer,
    command_buffer_setup: vk::CommandBuffer,
    setup_fence: vk::Fence,
    framebuffers: Vec<vk::Framebuffer>,
}

impl SubmitFrame {
    pub fn new(display: &Display) -> Self {
        let gpu = display.gpu();
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(gpu.queue_family_index());
        let command_pool = unsafe {
            gpu.create_command_pool(&pool_create_info)
                .expect("Failed to create a command pool")
        };

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let setup_fence = unsafe { gpu.create_fence(&fence_create_info) };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(2)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let (command_buffer_render, command_buffer_setup) = unsafe {
            gpu.allocate_command_buffers(&command_buffer_allocate_info)
                .into()
        };

        Self {
            gpu,
            command_pool,
            command_buffer_render,
            command_buffer_setup,
            setup_fence,
            framebuffers: vec![],
            surface_version: 0,
        }
    }

    unsafe fn create_framebuffers(&mut self, display: &Display) {
        let resolution = display.surface_resolution();
        self.framebuffers = display
            .swapchain_image_views()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, display.depth_image_view()];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(display.render_pass())
                    .attachments(&framebuffer_attachments)
                    .width(resolution.width)
                    .height(resolution.height)
                    .layers(1);

                self.gpu
                    .create_framebuffer(&frame_buffer_create_info)
                    .expect("Could not create a framebuffer")
            })
            .collect::<Vec<_>>()
    }

    unsafe fn destroy_framebuffers(&mut self) {
        for framebuffer in self.framebuffers.drain(..) {
            self.gpu.destroy_framebuffer(framebuffer);
        }
    }

    unsafe fn setup_depth_image(&self, display: &Display) {
        let depth_image = display.depth_image();

        // begin: prepare

        self.gpu
            .wait_for_fences(&[self.setup_fence], true, u64::MAX)
            .expect("Wait for fence failed.");

        self.gpu
            .reset_fences(&[self.setup_fence])
            .expect("Reset fences failed.");

        self.gpu
            .reset_command_buffer(
                self.command_buffer_setup,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed.");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.gpu
            .begin_command_buffer(self.command_buffer_setup, &command_buffer_begin_info)
            .expect("Begin commandbuffer");

        // end: prepare

        let layout_transition_barriers = vk::ImageMemoryBarrier::default()
            .image(depth_image)
            .dst_access_mask(
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .layer_count(1)
                    .level_count(1),
            );

        self.gpu.cmd_pipeline_barrier(
            self.command_buffer_setup,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[layout_transition_barriers],
        );

        // submit
        self.gpu
            .end_command_buffer(self.command_buffer_setup)
            .expect("End commandbuffer");

        let command_buffers = [self.command_buffer_setup];
        let wait_mask = [];
        let wait_semaphores = [];
        let signal_semaphores = [];

        let submits = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];

        self.gpu
            .submit_queue(&submits, self.setup_fence)
            .expect("queue submit failed.")
    }
}

impl Drop for SubmitFrame {
    fn drop(&mut self) {
        unsafe {
            self.gpu.device_wait_idle().unwrap();
            // framebuffers
            self.destroy_framebuffers();
            // command buffers
            self.gpu.destroy_command_pool(self.command_pool);
            // destory fence
            self.gpu.destroy_fence(self.setup_fence)
        };
    }
}

impl Task for SubmitFrame {
    type Context = (Ref<Display>, Any<Frame>, Take<All<RenderSubmit>>);
    type Output = FramePresenter;

    fn output_channel(&self) -> OutputChannel {
        OutputChannel::Scheduler
    }

    fn run(&mut self, (display, frame, submits): Self::Context) -> Self::Output {
        log::info!("get presenter");

        if let Some(surface_version) = display.surface_changed(self.surface_version) {
            unsafe {
                log::debug!("resize: Surface changed");
                // self.gpu.device_wait_idle().unwrap();

                // rebuild framebuffers
                log::debug!("resize: destroy_framebuffers");
                self.destroy_framebuffers();

                log::debug!("resize: create_framebuffers");
                self.create_framebuffers(&display);

                log::debug!("resize: setup_depth_image");
                self.setup_depth_image(&display);
            }
            self.surface_version = surface_version;
        }

        let mut submits = submits.take();

        submits.sort_by(|a, b| {
            let a_deps = a.wait_for();
            let b_deps = b.wait_for();
            let a_depends_on_b = a_deps.iter().any(|i| *i == b.id());
            let b_depends_on_a = b_deps.iter().any(|i| *i == a.id());

            if a_depends_on_b && b_depends_on_a {
                panic!("Circular rendering dependencies");
            }

            if a_depends_on_b && !b_depends_on_a {
                return std::cmp::Ordering::Greater;
            }

            if !a_depends_on_b && b_depends_on_a {
                return std::cmp::Ordering::Less;
            }

            a_deps.len().cmp(&b_deps.len())
        });
        /*
        .iter()
        .map(|i| {
            vk::SubmitInfo::default()
                .wait_semaphores(i.wait_semaphores.as_slice())
                .wait_dst_stage_mask(i.wait_dst_stage_mask.as_slice())
                .command_buffers(i.command_buffers.as_slice())
                .signal_semaphores(i.signal_semaphores.as_slice())
        })
        .collect::<Vec<_>>();
        */

        let draw_fence = display.draw_fence();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.1, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass = unsafe { display.render_pass() };
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(render_pass)
            .framebuffer(self.framebuffers[frame.swapchain_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: frame.resolution.width,
                    height: frame.resolution.height,
                },
            })
            .clear_values(&clear_values);

        unsafe {
            self.gpu
                .reset_command_buffer(
                    self.command_buffer_render,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Failed to reset Vulkan command buffer");

            self.gpu
                .begin_command_buffer(self.command_buffer_render, &command_buffer_begin_info)
                .expect("Failed to begin draw command buffer");

            self.gpu.cmd_begin_render_pass(
                self.command_buffer_render,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            for submit in submits.into_iter() {
                submit.record_command_buffer(&self.gpu, self.command_buffer_render);
            }

            self.gpu.cmd_end_render_pass(self.command_buffer_render);

            self.gpu
                .end_command_buffer(self.command_buffer_render)
                .expect("End commandbuffer");

            let present_complete_semaphore = display.present_complete_semaphore();
            let render_complete_semaphore = display.render_complete_semaphore();
            let wait_semaphores = [present_complete_semaphore];
            let wait_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.command_buffer_render];
            let signal_semaphores = [render_complete_semaphore];

            let submit_info = [vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_mask)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores)];

            self.gpu
                .submit_queue(&submit_info, draw_fence)
                .expect("Failed to submit draw buffer to queue");
        }
        display.presenter(frame.swapchain_index)
    }
}
