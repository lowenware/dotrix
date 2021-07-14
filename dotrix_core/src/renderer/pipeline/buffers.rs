use wgpu;

#[derive(Default)]
pub struct VertexBuffer {
    /// Packed vertex attributes
    attributes: Option<wgpu::Buffer>,
    /// Optional Indices buffer
    indices: Option<wgpu::Buffer>,
}

impl VertexBuffer {
    pub(crate) fn is_empty(&self) -> bool {
        self.attributes.is_none()
    }

    pub(crate) fn init<'a>(
        &mut self,
        device: wgpu::Device,
        attributes: &'a [u8],
        indices: Option<&'a [u8]>,
    ) -> Self {
        self.attributes = Some(
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("VertexBuffer"),
                    contents: attributes,
                    usage: wgpu::BufferUsage::VERTEX,
                }
            )
        );

        self.indices = indices.map(|contents| device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("IndexBuffer"),
                contents,
                usage: wgpu::BufferUsage::INDEX,
            }
        ));
    }

    pub(crate) fn reload<'a>(
        &self,
        queue: wgpu::Queue,
        attributes: &'a [u8],
        indices: Option<&'a [u8]>,
    ) -> Self {

        if let Some(buffer) = self.attributes.as_ref() {
            queue.write_buffer(buffer, 0, attributes);
        }

        if let Some(buffer) = self.indices.as_ref() {
            let indices = indices.expect("Indexed meshed can't be reloaded without indices");
            queue.write_buffer(buffer, 0, indices);
        }
    }
}

pub struct TextureBuffer {
    view: Option<wgpu::TextureView>,
}

impl TextureBuffer {
    pub fn new<'a>(
        device: wgpu::Device,
        attributes: &'a [u8],
    ) -> Self {
        Self {
            view: 
        }
    }
}
