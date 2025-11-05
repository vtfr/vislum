use std::{borrow::Cow, sync::Arc};

use crate::graph::{ExecuteContext, FrameNode, PrepareContext};
use ash::vk;
use vislum_render_rhi::{
    buffer::Buffer,
    command::{
        AccessFlags2, BufferImageCopy, BufferMemoryBarrier2, ImageAspectFlags, ImageLayout, ImageMemoryBarrier2, ImageSubresourceLayers, MemoryBarrier2, PipelineStageFlags2
    },
    image::{
        Extent3D, Image, ImageCreateInfo, ImageFormat, ImageType, ImageUsage, ImageView,
        ImageViewCreateInfo, ImageViewType,
    },
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
    image: Arc<Image>,
    view: Arc<ImageView>,
}

impl Texture {
    /// Creates a texture with data and returns both the texture and an upload task.
    pub fn new_with_data(
        device: Arc<vislum_render_rhi::device::Device>,
        allocator: Arc<MemoryAllocator>,
        info: TextureCreateInfo,
        data: &[u8],
    ) -> (Self, TextureUploadTask) {
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

        // Create default image view
        let view_type = ImageViewType::from(rhi_dimensions);
        let view = ImageView::new(
            device.clone(),
            ImageViewCreateInfo {
                image: image.clone(),
                view_type,
                format: rhi_format,
                components: vk::ComponentMapping::default(),
                subresource_range: vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            },
        );

        // Create staging buffer with host-visible memory
        let staging = Buffer::new_staging_with_data(device.clone(), allocator, data);

        let upload_task = TextureUploadTask {
            image: image.clone(),
            staging_buffer: staging,
            extent: vk::Extent3D {
                width: info.extent.width,
                height: info.extent.height,
                depth: info.extent.depth,
            },
        };

        (Texture { image, view }, upload_task)
    }

    #[inline]
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    #[inline]
    pub fn view(&self) -> &Arc<ImageView> {
        &self.view
    }
}

pub struct TextureUploadTask {
    image: Arc<Image>,
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
    ) -> Box<dyn FnMut(&mut ExecuteContext) + 'static> {
        let destination = self.image.clone();
        let staging_buffer = self.staging_buffer.clone();
        let extent = Extent3D::from_vk(self.extent);

        Box::new(move |execute_context| {
            let cmd = &mut execute_context.command_buffer;

            // Transition image from undefined to transfer destination layout
            cmd.pipeline_barrier(
                std::iter::empty(),
                std::iter::once(BufferMemoryBarrier2{
                    buffer: staging_buffer.clone(),
                    src_stage_mask: PipelineStageFlags2::TOP_OF_PIPE,
                    src_access_mask: AccessFlags2::NONE,
                    dst_stage_mask: PipelineStageFlags2::TRANSFER,
                    dst_access_mask: AccessFlags2::TRANSFER_WRITE,
                    offset: 0,
                    size: staging_buffer.size(),
                }),
                std::iter::once(ImageMemoryBarrier2 {
                    image: destination.clone(),
                    src_stage_mask: PipelineStageFlags2::TOP_OF_PIPE,
                    src_access_mask: AccessFlags2::NONE,
                    dst_stage_mask: PipelineStageFlags2::TRANSFER,
                    dst_access_mask: AccessFlags2::TRANSFER_WRITE,
                    old_layout: ImageLayout::Undefined,
                    new_layout: ImageLayout::TransferDstOptimal,
                }),
            );


            cmd.copy_buffer_to_image(
                staging_buffer.clone(),
                destination.clone(),
                ImageLayout::TransferDstOptimal,
                std::iter::once(BufferImageCopy {
                    buffer_offset: 0,
                    buffer_row_length: 0,
                    buffer_image_height: 0,
                    image_subresource: ImageSubresourceLayers {
                        aspect_mask: ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image_offset: [0, 0, 0],
                    image_extent: extent,
                }),
            );

            // Transition image to shader read layout
            cmd.pipeline_barrier(
                std::iter::empty(),
                std::iter::empty(),
                std::iter::once(ImageMemoryBarrier2 {
                    image: destination.clone(),
                    src_stage_mask: PipelineStageFlags2::TRANSFER,
                    src_access_mask: AccessFlags2::TRANSFER_WRITE,
                    dst_stage_mask: PipelineStageFlags2::FRAGMENT_SHADER,
                    dst_access_mask: AccessFlags2::SHADER_READ,
                    old_layout: ImageLayout::TransferDstOptimal,
                    new_layout: ImageLayout::ShaderReadOnlyOptimal,
                }),
            );
        })
    }
}
