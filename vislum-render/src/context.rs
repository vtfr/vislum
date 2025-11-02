use std::{borrow::Cow, sync::Arc};

use ash::vk;
use vislum_render_rhi::{
    device::Device,
    memory::MemoryAllocator,
    queue::Queue,
    image::Image,
};

use crate::{graph::{ExecuteContext, FrameGraph, PrepareContext, pass::FrameGraphSubmitInfo}, resource::{ResourceManager, pool::ResourceId, texture::{Texture, TextureDimensions, TextureFormat}}};

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
        allocator: Arc<MemoryAllocator>,
    ) -> Self {
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
    pub fn add_pass<S, E, P>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        prepare: P,
        execute: E,
    ) where
        S: 'static,
        P: Fn(&mut PrepareContext) -> S,
        E: for<'g, 's> Fn(&mut ExecuteContext<'g>, &'s mut S) + 'static,
    {
        self.frame_graph.add_pass(&self.resource_manager, name, prepare, execute);
    }

    pub fn execute_and_submit(&mut self, submit_info: FrameGraphSubmitInfo) {
        self.frame_graph.execute_and_submit(submit_info);
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

        struct UploadTextureState {
            staging_buffer: Arc<vislum_render_rhi::buffer::Buffer>,
            destination: Arc<Image>,
            extent: vk::Extent3D,
        }

        let extent_vk = vk::Extent3D {
            width: extent[0],
            height: extent[1],
            depth: extent[2],
        };

        // Temporary hack to keep the staging buffer alive until the upload pass is executed.
        //
        // We'll fix this later.
        std::mem::drop(staging_buffer.clone());

        self.frame_graph.add_pass(
            &self.resource_manager, 
            "upload_texture", 
            |prepare_context| {
                let destination = prepare_context.write_texture(id).unwrap();

                UploadTextureState {
                    staging_buffer: staging_buffer.clone(),
                    destination,
                    extent: extent_vk,
                }
            },
            |execute_context, state| {
                // Transition image to transfer destination layout
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
                    .image_extent(state.extent);
                
                execute_context.command_encoder.copy_buffer_to_image(
                    &state.staging_buffer,
                    &state.destination,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[copy_region],
                );
                
                // Transition image to shader read layout
                execute_context.command_encoder.transition_image(
                    &state.destination,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::AccessFlags2::SHADER_READ,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                );
            }
        );
        
        id
    }


    pub fn get_texture_image(&self, id: ResourceId<Texture>) -> Option<Arc<Image>> {
        self.resource_manager.resolve_texture_image(id)
    }
}
