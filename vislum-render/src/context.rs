use std::{borrow::Cow, sync::Arc};

use ash::vk;
use vislum_render_rhi::{
    device::Device,
    memory::MemoryAllocator,
    queue::Queue,
    image::Image,
};

use crate::{graph::{ExecuteContext, FrameGraph, PrepareContext, pass::FrameGraphSubmitInfo, FrameNode}, resource::{ResourceManager, pool::ResourceId, texture::{Texture, TextureDimensions, TextureFormat}}};

pub struct RenderContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    allocator: Arc<MemoryAllocator>,
    resource_manager: ResourceManager,
    frame_graph: FrameGraph,
}

impl RenderContext {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self {
        let allocator = MemoryAllocator::new(device.clone());
        let resource_manager = ResourceManager::new(device.clone(), allocator.clone());
        let frame_graph = FrameGraph::new(device.clone(), queue.clone(), allocator.clone());

        Self {
            device,
            queue,
            allocator,
            resource_manager,
            frame_graph,
        }
    }

    /// Adds a new pass to the frame graph.
    pub fn add_pass<F>(&mut self, node: F)
    where
        F: FrameNode + 'static,
    {
        self.frame_graph.add_pass(node);
    }

    pub fn execute_and_submit(&mut self, submit_info: FrameGraphSubmitInfo) {
        self.frame_graph.execute(&self.resource_manager, submit_info);
    }

    /// Uploads data to a buffer.
    pub fn create_texture(&mut self, format: TextureFormat, dimensions: TextureDimensions, data: &[u8]) -> ResourceId<Texture> {
        self.create_texture_with_extent(format, dimensions, data, None)
    }

    /// Creates a texture with explicit extent dimensions.
    pub fn create_texture_with_extent(&mut self, format: TextureFormat, dimensions: TextureDimensions, data: &[u8], extent: Option<[u32; 3]>) -> ResourceId<Texture> {
        // Calculate extent before creating texture
        let extent = match extent {
            Some(ext) => ext,
            None => match dimensions {
                TextureDimensions::D2 => {
                    // Assume square texture - calculate from data size
                    let pixel_count = data.len() / 4; // 4 bytes per RGBA pixel
                    let side = (pixel_count as f32).sqrt() as u32;
                    let side = side.max(1);
                    [side, side, 1]
                }
                TextureDimensions::D3 => {
                    // For 3D, use a reasonable default
                    [1024, 1024, 1024]
                }
            }
        };

        let (id, staging_buffer) = self.resource_manager.create_texture_with_extent(format, dimensions, data, Some(extent));

        struct UploadTextureNode {
            texture_id: ResourceId<Texture>,
            staging_buffer: Arc<vislum_render_rhi::buffer::Buffer>,
            extent: vk::Extent3D,
        }

        impl FrameNode for UploadTextureNode {
            fn name(&self) -> Cow<'static, str> {
                "upload_texture".into()
            }

            fn prepare(&self, context: &mut PrepareContext) -> Box<dyn FnMut(&mut ExecuteContext<'_>) + 'static> {
                let destination = context.write_texture(self.texture_id).unwrap();
                let staging_buffer = self.staging_buffer.clone();
                let extent = self.extent;

                Box::new(move |execute_context| {
                    // Copy staging buffer to image (AutoCommandBuffer handles barriers)
                    let copy_region = vk::BufferImageCopy::default()
                        .buffer_offset(0)
                        .buffer_row_length(0) // 0 means tightly packed
                        .buffer_image_height(0) // 0 means tightly packed
                        .image_subresource(vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(0)
                            .base_array_layer(0)
                            .layer_count(1))
                        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                        .image_extent(extent);
                    
                    use vislum_render_rhi::command::types::ImageLayout;
                    execute_context.command_encoder.copy_buffer_to_image(
                        &staging_buffer,
                        &destination,
                        ImageLayout::TransferDstOptimal,
                        &[copy_region],
                    );
                    
                    // Transition image to shader read layout
                    use vislum_render_rhi::command::types::{AccessFlags2, PipelineStageFlags2};
                    execute_context.command_encoder.transition_image(
                        &destination,
                        ImageLayout::TransferDstOptimal,
                        ImageLayout::ShaderReadOnlyOptimal,
                        AccessFlags2::TRANSFER_WRITE,
                        AccessFlags2::SHADER_READ,
                        PipelineStageFlags2::TRANSFER,
                        PipelineStageFlags2::FRAGMENT_SHADER,
                    );
                })
            }
        }

        self.add_pass(UploadTextureNode {
            texture_id: id,
            staging_buffer,
            extent: vk::Extent3D {
                width: extent[0],
                height: extent[1],
                depth: extent[2],
            },
        });
        
        id
    }


    pub fn get_texture_image(&self, id: ResourceId<Texture>) -> Option<Arc<Image>> {
        self.resource_manager.resolve_texture_image(id)
    }
}
