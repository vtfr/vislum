use std::{borrow::Cow, sync::Arc};

use crate::graph::{ExecuteContext, FrameNode, PrepareContext};
use ash::vk;
use vislum_render_rhi::{
    buffer::Buffer,
    image::{Extent3D, Image, ImageCreateInfo, ImageFormat, ImageType, ImageUsage},
    memory::MemoryAllocator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8Srgb,
    Rgb8Unorm,
    Rgb8Srgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDimensions {
    D2,
    D3,
}

impl Into<vk::ImageType> for TextureDimensions {
    fn into(self) -> vk::ImageType {
        match self {
            TextureDimensions::D2 => vk::ImageType::TYPE_2D,
            TextureDimensions::D3 => vk::ImageType::TYPE_3D,
        }
    }
}

/// Information needed to create a texture.
pub struct TextureCreateInfo {
    pub format: TextureFormat,
    pub dimensions: TextureDimensions,
    pub extent: Extent3D,
}

pub struct Texture {
    pub(crate) image: Arc<Image>,
}

impl Texture {
    /// Creates a texture with data and returns both the texture and an upload task.
    pub fn new_with_data(
        device: Arc<vislum_render_rhi::device::Device>,
        allocator: Arc<MemoryAllocator>,
        info: TextureCreateInfo,
        data: &[u8],
    ) -> (Arc<Self>, TextureUploadTask) {
        let rhi_format = match info.format {
            TextureFormat::Rgba8Unorm => ImageFormat::Rgba8Unorm,
            TextureFormat::Rgba8Srgb => ImageFormat::Rgba8Srgb,
            TextureFormat::Rgb8Unorm => ImageFormat::Rgb8Unorm,
            TextureFormat::Rgb8Srgb => ImageFormat::Rgb8Srgb,
        };

        let rhi_dimensions = match info.dimensions {
            TextureDimensions::D2 => ImageType::D2,
            TextureDimensions::D3 => ImageType::D3,
        };

        let image = Image::new(
            device.clone(),
            allocator.clone(),
            ImageCreateInfo {
                dimensions: rhi_dimensions,
                format: rhi_format,
                extent: info.extent,
                mip_levels: 1,
                array_layers: 1,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            },
            vislum_render_rhi::memory::MemoryLocation::GpuOnly,
        );

        let texture = Arc::new(Texture { image });

        // Create staging buffer with host-visible memory
        let staging = Buffer::new_staging_with_data(device, allocator, data);

        let upload_task = TextureUploadTask {
            texture: texture.clone(),
            staging_buffer: staging,
            extent: vk::Extent3D {
                width: info.extent.width,
                height: info.extent.height,
                depth: info.extent.depth,
            },
        };

        (texture, upload_task)
    }

    #[inline]
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }
}

pub struct TextureUploadTask {
    texture: Arc<Texture>,
    staging_buffer: Arc<Buffer>,
    extent: vk::Extent3D,
}

impl FrameNode for TextureUploadTask {
    fn name(&self) -> Cow<'static, str> {
        "upload_texture".into()
    }

    fn prepare(
        &self,
        _context: &mut PrepareContext,
    ) -> Box<dyn FnMut(&mut ExecuteContext<'_>) + 'static> {
        let destination = self.texture.image.clone();
        let staging_buffer = self.staging_buffer.clone();
        let extent = self.extent;

        Box::new(move |execute_context| {
            // Transition image from undefined to transfer destination layout
            use vislum_render_rhi::command::types::{
                AccessFlags2, ImageLayout, PipelineStageFlags2,
            };
            execute_context.command_encoder.transition_image(
                &destination,
                ImageLayout::Undefined,
                ImageLayout::TransferDstOptimal,
                AccessFlags2::NONE,
                AccessFlags2::TRANSFER_WRITE,
                PipelineStageFlags2::TOP_OF_PIPE,
                PipelineStageFlags2::TRANSFER,
            );

            // Copy staging buffer to image
            let copy_region = vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0) // 0 means tightly packed
                .buffer_image_height(0) // 0 means tightly packed
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(extent);

            execute_context.command_encoder.copy_buffer_to_image(
                &staging_buffer,
                &destination,
                ImageLayout::TransferDstOptimal,
                &[copy_region],
            );

            // Transition image to shader read layout
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
