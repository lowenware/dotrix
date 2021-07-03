pub mod solid;

// impl Model {

    // Returns loaded assets if they are all ready
    /*
    fn get_assets<'a>(
        &self,
        assets: &'a mut Assets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(&'a Mesh, &'a Texture, Option<&'a Skin>), ()> {

        if !self.skin.is_null() && assets.get(self.skin).is_none() {
            return Err(());
        }

        if let Some(mesh) = assets.get_mut(self.mesh) {
            if self.skin.is_null() {
                mesh.load_as_static(device);
            } else {
                mesh.load_as_skinned(device);
            }
        }

        if let Some(texture) = assets.get_mut(self.texture) {
            texture.load(device, queue);
        }

        if let Some(mesh) = assets.get(self.mesh) {
            if let Some(texture) = assets.get(self.texture) {
                let skin = assets.get(self.skin);
                return Ok((mesh, texture, skin));
            }
        }
        Err(())
    }
    *   */
    // Loads the [`Model`] buffers
    /*
    pub(crate) fn load(
        &mut self,
        renderer: &Renderer,
        assets: &mut Assets,
        pipeline: &Pipeline,
        sampler: &wgpu::Sampler,
        proj_view: &wgpu::Buffer,
        lights_buffer: &wgpu::Buffer,
    ) {
        use wgpu::util::DeviceExt;

        let device = &renderer.device;
        let queue = &renderer.queue;

        let transform_matrix = self.transform.matrix();
        let model_transform = AsRef::<[f32; 16]>::as_ref(&transform_matrix);

        if let Ok((_, texture, skin)) = self.get_assets(assets, device, queue) {
            if let Some(buffers) = self.buffers.as_ref() {
                queue.write_buffer(&buffers.transform, 0, bytemuck::cast_slice(model_transform));

                if let Some(pose) = self.pose.as_ref() {
                    if let Some(skin) = assets.get(self.skin) {
                        pose.load(&skin.index, queue);
                    }
                }
            } else {
                let transform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Model Transform"),
                    contents: bytemuck::cast_slice(model_transform),
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                });

                let mut bind_group_entries = Vec::with_capacity(6);
                let pose = skin.map(|_| Pose::new(device));
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: proj_view.as_entire_binding(),
                });
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: transform.as_entire_binding(),
                });
                if let Some(pose) = pose.as_ref() {
                    bind_group_entries.push(wgpu::BindGroupEntry {
                        binding: 2,
                        resource: pose.buffer.as_entire_binding(),
                    });
                }
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                });
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(sampler),
                });
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 5,
                    resource: lights_buffer.as_entire_binding(),
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline.bind_group_layout,
                    entries: &bind_group_entries,
                    label: None,
                });

                self.pose = pose;

                self.buffers = Some(
                    Buffers {
                        bind_group,
                        transform,
                    }
                )
            }
        }
    }
    */

    // Renders the [`Model`]
    /*
    pub(crate) fn draw(
        &self,
        assets: &Assets,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &Pipeline,
        frame: &wgpu::SwapChainTexture,
        depth_buffer: &wgpu::TextureView,
    ) {
        if let Some(buffers) = self.buffers.as_ref() {
            let mesh = assets
                .get(self.mesh)
                .expect("Static model must have a mesh");

            let vertices_buffer = mesh
                .vertices_buffer
                .as_ref()
                .expect("Static model mesh must have initialized buffers at this stage");

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, 
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: depth_buffer,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                ),
            });

            rpass.push_debug_group("Prepare to draw a Model");
            rpass.set_pipeline(&pipeline.wgpu_pipeline);
            rpass.set_bind_group(0, &buffers.bind_group, &[]);
            rpass.set_vertex_buffer(0, vertices_buffer.slice(..));
            rpass.pop_debug_group();

            if let Some(indices_buffer) = mesh.indices_buffer.as_ref() {
                rpass.insert_debug_marker("Draw indexed model");
                rpass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh.indices_count(), 0, 0..1);
            } else {
                rpass.insert_debug_marker("Draw a model");
                rpass.draw(0..mesh.indices_count(), 0..1);
            }
        }
    }
    */

    /*
    pub fn load(&mut self, renderer: &Renderer, assets: &mut Assets) {
        if let Some(mesh) = assets.get_mut(self.mesh) {
            mesh.load(renderer);
        }

        if let Some(texture) = assets.get_mut(self.texture) {
            texture.load(renderer);
        }

        if let Some(skin) = assets.get_mut(self.skin) {
            if let Some(pose) = self.pose.as_mut() {
                pose.load(renderer, &skin.index);
            }
        }

        let transform_matrix = self.transform.matrix();
        let transform_raw = AsRef::<[f32; 16]>::as_ref(&transform_matrix);

        renderer.load_uniform_buffer(
            &mut self.transform_buffer,
            bytemuck::cast_slice(transform_raw)
        );
    }
    *
*/

// }
/*
        if model.pipeline.is_null() {
            let pipelines = ctx.pipelines.as_ref().unwrap();
            model.pipeline = if !model.skin.is_null() {
                pipelines.skinned_model
            } else {
                pipelines.static_model
            };
        }
        let pipeline = renderer.pipeline(model.pipeline);
        let proj_view_buffer = ctx.proj_view_buffer.as_ref().unwrap();
        let lights_buffer = ctx.lights_buffer.as_ref().unwrap();

        model.load(&renderer, &mut assets, pipeline, sampler, proj_view_buffer, lights_buffer);
        model.draw(&assets, &mut encoder, pipeline, frame, depth_buffer);
    }
}
*/
