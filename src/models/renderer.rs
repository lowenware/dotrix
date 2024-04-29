use crate::graphics::vk;
use crate::graphics::vulkan::Buffer;
use crate::graphics::{CommandRecorder, Framebuffers, RenderPass, Semaphore};
use crate::loaders::Assets;
use crate::utils::{BufferLayout, Id, LayoutInBuffer, MeshLayout, MeshVerticesLayout};
use crate::world::{Entity, World};
use crate::{log, Asset};
use crate::{Any, Display, Frame, Gpu, Ref, Task};

use super::{Armature, Material, Mesh, Transform, VertexNormal, VertexPosition, VertexTexture};

pub struct RenderModels {
    /// GPU instance
    gpu: Gpu,
    /// Wait for these semaphores before executing command buffers
    wait_semaphores: Vec<Semaphore>,
    /// Signal these semaphores after rendering is done
    signal_semaphore: Semaphore,
    /// Command Pool
    command_pool: vk::CommandPool,
    /// Setup command buffer
    setup_command_buffer: vk::CommandBuffer,
    /// Setup command buffer reuse fence
    setup_command_buffer_reuse_fence: vk::Fence,
    /// Draw command buffer
    draw_command_buffer: vk::CommandBuffer,
    /// Draw command buffer reuse fence
    draw_command_buffer_reuse_fence: vk::Fence,
    /// Framebuffers
    framebuffers: Framebuffers,
    /// Render pass
    render_pass: vk::RenderPass,
    /// Version of surface to track changes and update framebuffers and fender pass
    surface_version: u64,
    /// Vertex buffer (non-rigged meshes)
    vertex_buffer_non_rigged: Buffer,
    /// Mesh layout snapshot, defining the order of meshes in the buffer
    mesh_layout_snapshot: Vec<Id<Mesh>>,
    /// Mesh layouts of non-rigged models
    mesh_layout_non_rigged: BufferLayout<Id<Mesh>, MeshLayout>,
    /// Config mesh buffer size
    cfg_mesh_buffer_size: u64,
}

pub type VertexBufferLayoutNonRigged = (VertexPosition, VertexNormal, VertexTexture);

impl Drop for RenderModels {
    fn drop(&mut self) {
        unsafe {
            // TODO: destroy vertex buffer
            self.gpu.destroy_command_pool(self.command_pool);
            self.gpu.destroy_render_pass(self.render_pass);
            self.gpu
                .destroy_fence(self.setup_command_buffer_reuse_fence);
            self.gpu.destroy_fence(self.draw_command_buffer_reuse_fence);
            self.framebuffers.destroy(&self.gpu);
        }
    }
}

impl Task for RenderModels {
    type Context = (Any<Frame>, Ref<Assets>, Ref<Display>, Ref<World>);
    type Output = RenderPass;

    fn run(&mut self, (frame, assets, display, world): Self::Context) -> Self::Output {
        log::debug!("pbr: begin");

        if let Some(surface_version) = display.surface_changed(self.surface_version) {
            unsafe {
                self.framebuffers.rebuild(&display, self.render_pass);
            };
            self.surface_version = surface_version;
        }

        self.update_buffers(&assets, &world);

        unsafe {
            self.execute_render_pass(&frame);
            self.submit_draw_commands();
        }

        log::debug!("pbr: submit_command_buffer");
        RenderPass {}
    }
}

impl RenderModels {
    pub fn setup() -> RenderModelsSetup {
        RenderModelsSetup::default()
    }

    pub fn new(gpu: Gpu, setup: RenderModelsSetup) -> Self {
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

        let cfg_mesh_buffer_size = setup.mesh_buffer_size;

        let vertex_buffer_non_rigged = unsafe {
            Buffer::setup()
                .size(cfg_mesh_buffer_size)
                .use_as_target()
                .use_as_vertex()
                .create(&gpu)
        };

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
            vertex_buffer_non_rigged,
            mesh_layout_snapshot: Vec::new(),
            mesh_layout_non_rigged: BufferLayout::new(cfg_mesh_buffer_size),
            cfg_mesh_buffer_size,
        }
    }

    pub fn complete_semaphore(&self) -> &Semaphore {
        &self.signal_semaphore
    }

    fn update_buffers(&mut self, assets: &Assets, world: &World) {
        // [x] Step 0: Model registry
        // [x] Step 1: Vertex buffer
        // [ ] Step 2: Transform buffer
        // [ ] Step 3: Material buffer
        // [ ] Step 4: Indirect buffer
        // [ ] Step 5: Shader, pipeline
        // [ ] Step 6: Debug

        for (entity_id, mesh_id, material_id, armature_id, transform) in world.query::<(
            &Id<Entity>,
            &Id<Mesh>,
            &Id<Material>,
            &Id<Armature>,
            &Transform,
        )>() {
            log::debug!("Update buffers: {:?}", entity_id);
            let mesh_vertex_layout =
                if let Some(mesh_vertex_layout) = self.register_mesh(*mesh_id, assets) {
                    mesh_vertex_layout
                } else {
                    continue;
                };
            let material_layout =
                if let Some(material_layout) = self.register_material(*material_id, assets) {
                    material_layout
                } else {
                    continue;
                };
        }
    }

    fn register_mesh(&mut self, mesh_id: Id<Mesh>, assets: &Assets) -> Option<MeshVerticesLayout> {
        // check if the mesh is already in buffer
        if let Some(mesh_layout) = self.mesh_layout_non_rigged.get(mesh_id) {
            return Some(mesh_layout.vertices.clone());
        }
        // try to get mesh to store it in buffer
        if let Some(mesh) = assets.get(mesh_id) {
            if mesh.indices::<u8>().is_some() {
                panic!("Index buffer is not implemented yet");
            }
            if let Some(vertex_data) = mesh.buffer::<VertexBufferLayoutNonRigged>() {
                use crate::models::meshes::VertexBufferLayout;
                let vertex_size = VertexBufferLayoutNonRigged::vertex_size() as u64;
                let data_size = vertex_data.len() as u64;

                let offset = self.mesh_layout_non_rigged.used_size();

                let vertex_layout = MeshVerticesLayout {
                    base_vertex: (offset / vertex_size) as u32,
                    vertex_count: mesh.count_vertices() as u32,
                };

                self.mesh_layout_non_rigged
                    .store(
                        mesh_id,
                        MeshLayout {
                            in_vertex_buffer: LayoutInBuffer {
                                offset,
                                size: data_size,
                            },
                            in_index_buffer: None,
                            vertices: vertex_layout.clone(),
                        },
                        data_size,
                    )
                    .ok();

                self.mesh_layout_snapshot.push(mesh_id);
                return Some(vertex_layout);
            } else {
                log::error!("Could not store the mesh named `{}`", mesh.name());
            }
        }
        None
    }
    /*
    let index_buffer_data = [0u32, 1, 2];
    let index_buffer_info = vk::BufferCreateInfo::default()
        .size(std::mem::size_of_val(&index_buffer_data) as u64)
        .usage(vk::BufferUsageFlags::INDEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let index_buffer = base.device.create_buffer(&index_buffer_info, None).unwrap();
    let index_buffer_memory_req = base.device.get_buffer_memory_requirements(index_buffer);
    let index_buffer_memory_index = find_memorytype_index(
        &index_buffer_memory_req,
        &base.device_memory_properties,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .expect("Unable to find suitable memorytype for the index buffer.");

    let index_allocate_info = vk::MemoryAllocateInfo {
        allocation_size: index_buffer_memory_req.size,
        memory_type_index: index_buffer_memory_index,
        ..Default::default()
    };
    let index_buffer_memory = base
        .device
        .allocate_memory(&index_allocate_info, None)
        .unwrap();
    let index_ptr = base
        .device
        .map_memory(
            index_buffer_memory,
            0,
            index_buffer_memory_req.size,
            vk::MemoryMapFlags::empty(),
        )
        .unwrap();
    let mut index_slice = Align::new(
        index_ptr,
        align_of::<u32>() as u64,
        index_buffer_memory_req.size,
    );
    index_slice.copy_from_slice(&index_buffer_data);
    base.device.unmap_memory(index_buffer_memory);
    base.device
        .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
        .unwrap();

    let vertex_input_buffer_info = vk::BufferCreateInfo {
        size: 3 * std::mem::size_of::<Vertex>() as u64,
        usage: vk::BufferUsageFlags::VERTEX_BUFFER,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let vertex_input_buffer = base
        .device
        .create_buffer(&vertex_input_buffer_info, None)
        .unwrap();

    let vertex_input_buffer_memory_req = base
        .device
        .get_buffer_memory_requirements(vertex_input_buffer);

    let vertex_input_buffer_memory_index = find_memorytype_index(
        &vertex_input_buffer_memory_req,
        &base.device_memory_properties,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .expect("Unable to find suitable memorytype for the vertex buffer.");

    let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
        allocation_size: vertex_input_buffer_memory_req.size,
        memory_type_index: vertex_input_buffer_memory_index,
        ..Default::default()
    };

    let vertex_input_buffer_memory = base
        .device
        .allocate_memory(&vertex_buffer_allocate_info, None)
        .unwrap();

    let vertices = [
        Vertex {
            pos: [-1.0, 1.0, 0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [1.0, 1.0, 0.0, 1.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex {
            pos: [0.0, -1.0, 0.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
    ];

    let vert_ptr = base
        .device
        .map_memory(
            vertex_input_buffer_memory,
            0,
            vertex_input_buffer_memory_req.size,
            vk::MemoryMapFlags::empty(),
        )
        .unwrap();

    let mut vert_align = Align::new(
        vert_ptr,
        align_of::<Vertex>() as u64,
        vertex_input_buffer_memory_req.size,
    );
    vert_align.copy_from_slice(&vertices);
    base.device.unmap_memory(vertex_input_buffer_memory);
    base.device
        .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)
        .unwrap();
    */

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

pub struct RenderModelsSetup {
    wait_semaphores: Vec<Semaphore>,
    surface_format: vk::Format,
    mesh_buffer_size: u64,
}

impl Default for RenderModelsSetup {
    fn default() -> Self {
        Self {
            wait_semaphores: Vec::new(),
            surface_format: vk::Format::default(),
            mesh_buffer_size: 8 * 1024 * 1024,
        }
    }
}

impl RenderModelsSetup {
    pub fn wait_semaphores(mut self, semaphores: impl IntoIterator<Item = Semaphore>) -> Self {
        self.wait_semaphores.extend(semaphores);
        self
    }

    pub fn surface_format(mut self, surface_format: vk::Format) -> Self {
        self.surface_format = surface_format;
        self
    }

    pub fn create(self, gpu: Gpu) -> RenderModels {
        RenderModels::new(gpu, self)
    }
}
