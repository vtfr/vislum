use std::sync::Arc;

use crate::{VkHandle, buffer::Buffer, image::{Extent2D, Extent3D, Image}, vk_enum, vk_enum_flags};
use ash::vk;

vk_enum_flags! {
    pub struct CommandBufferUsageFlags: vk::CommandBufferUsageFlags {
        ONE_TIME_SUBMIT => ONE_TIME_SUBMIT,
        RENDER_PASS_CONTINUE => RENDER_PASS_CONTINUE,
        SIMULTANEOUS_USE => SIMULTANEOUS_USE,
    }
}

vk_enum! {
    pub enum ImageLayout: vk::ImageLayout {
        Undefined => UNDEFINED,
        General => GENERAL,
        ColorAttachmentOptimal => COLOR_ATTACHMENT_OPTIMAL,
        DepthStencilAttachmentOptimal => DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        DepthStencilReadOnlyOptimal => DEPTH_STENCIL_READ_ONLY_OPTIMAL,
        ShaderReadOnlyOptimal => SHADER_READ_ONLY_OPTIMAL,
        TransferSrcOptimal => TRANSFER_SRC_OPTIMAL,
        TransferDstOptimal => TRANSFER_DST_OPTIMAL,
        PresentSrcKhr => PRESENT_SRC_KHR,
    }
}

vk_enum_flags! {
    pub struct AccessFlags2: vk::AccessFlags2 {
        NONE => NONE,
        INDIRECT_COMMAND_READ => INDIRECT_COMMAND_READ,
        INDEX_READ => INDEX_READ,
        VERTEX_ATTRIBUTE_READ => VERTEX_ATTRIBUTE_READ,
        UNIFORM_READ => UNIFORM_READ,
        INPUT_ATTACHMENT_READ => INPUT_ATTACHMENT_READ,
        SHADER_READ => SHADER_READ,
        SHADER_WRITE => SHADER_WRITE,
        COLOR_ATTACHMENT_READ => COLOR_ATTACHMENT_READ,
        COLOR_ATTACHMENT_WRITE => COLOR_ATTACHMENT_WRITE,
        DEPTH_STENCIL_ATTACHMENT_READ => DEPTH_STENCIL_ATTACHMENT_READ,
        DEPTH_STENCIL_ATTACHMENT_WRITE => DEPTH_STENCIL_ATTACHMENT_WRITE,
        TRANSFER_READ => TRANSFER_READ,
        TRANSFER_WRITE => TRANSFER_WRITE,
        HOST_READ => HOST_READ,
        HOST_WRITE => HOST_WRITE,
        MEMORY_READ => MEMORY_READ,
        MEMORY_WRITE => MEMORY_WRITE,
    }
}

vk_enum_flags! {
    pub struct PipelineStageFlags2: vk::PipelineStageFlags2 {
        NONE => NONE,
        TOP_OF_PIPE => TOP_OF_PIPE,
        DRAW_INDIRECT => DRAW_INDIRECT,
        VERTEX_INPUT => VERTEX_INPUT,
        VERTEX_SHADER => VERTEX_SHADER,
        TESSELLATION_CONTROL_SHADER => TESSELLATION_CONTROL_SHADER,
        TESSELLATION_EVALUATION_SHADER => TESSELLATION_EVALUATION_SHADER,
        GEOMETRY_SHADER => GEOMETRY_SHADER,
        FRAGMENT_SHADER => FRAGMENT_SHADER,
        EARLY_FRAGMENT_TESTS => EARLY_FRAGMENT_TESTS,
        LATE_FRAGMENT_TESTS => LATE_FRAGMENT_TESTS,
        COLOR_ATTACHMENT_OUTPUT => COLOR_ATTACHMENT_OUTPUT,
        COMPUTE_SHADER => COMPUTE_SHADER,
        ALL_COMMANDS => ALL_COMMANDS,
        ALL_GRAPHICS => ALL_GRAPHICS,
        TRANSFER => TRANSFER,
        BOTTOM_OF_PIPE => BOTTOM_OF_PIPE,
        HOST => HOST,
    }
}

vk_enum! {
    pub enum PipelineBindPoint: vk::PipelineBindPoint {
        Graphics => GRAPHICS,
        Compute => COMPUTE,
    }
}

vk_enum! {
    pub enum IndexType: vk::IndexType {
        Uint16 => UINT16,
        Uint32 => UINT32,
        Uint8Ext => UINT8_EXT,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn to_vk(self) -> vk::Viewport {
        vk::Viewport {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            min_depth: self.min_depth,
            max_depth: self.max_depth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rect2D {
    pub offset: [i32; 2],
    pub extent: Extent2D,
}

impl Rect2D {
    pub fn new(offset: [i32; 2], extent: Extent2D) -> Self {
        Self { offset, extent }
    }

    pub fn from_vk(rect: vk::Rect2D) -> Self {
        Self {
            offset: [rect.offset.x, rect.offset.y],
            extent: Extent2D::from_vk(rect.extent),
        }
    }

    pub fn to_vk(self) -> vk::Rect2D {
        vk::Rect2D {
            offset: vk::Offset2D {
                x: self.offset[0],
                y: self.offset[1],
            },
            extent: self.extent.to_vk(),
        }
    }
}

// vk_enum_flags! {
//     pub struct DependencyFlags: vk::DependencyFlags {
//         BY_REGION => BY_REGION,
//         VIEW_LOCAL => VIEW_LOCAL,
//         DEVICE_GROUP => DEVICE_GROUP,
//     }
// }

vk_enum_flags! {
    pub struct ImageAspectFlags: vk::ImageAspectFlags {
        COLOR => COLOR,
        DEPTH => DEPTH,
        STENCIL => STENCIL,
        METADATA => METADATA,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageSubresourceRange {
    pub aspect_mask: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl ImageSubresourceRange {
    pub fn new(
        aspect_mask: ImageAspectFlags,
        base_mip_level: u32,
        level_count: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Self {
        Self {
            aspect_mask,
            base_mip_level,
            level_count,
            base_array_layer,
            layer_count,
        }
    }

    pub fn to_vk(self) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: self.aspect_mask.to_vk(),
            base_mip_level: self.base_mip_level,
            level_count: self.level_count,
            base_array_layer: self.base_array_layer,
            layer_count: self.layer_count,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryBarrier2 {
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
}

impl MemoryBarrier2 {
    pub fn to_vk(self) -> vk::MemoryBarrier2<'static> {
        vk::MemoryBarrier2::default()
            .src_stage_mask(self.src_stage_mask.to_vk())
            .src_access_mask(self.src_access_mask.to_vk())
            .dst_stage_mask(self.dst_stage_mask.to_vk())
            .dst_access_mask(self.dst_access_mask.to_vk())
    }
}

#[derive(Clone)]
pub struct BufferMemoryBarrier2 {
    pub buffer: Arc<Buffer>,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
    pub offset: u64,
    pub size: u64,
}

impl BufferMemoryBarrier2 {
    pub fn to_vk(self) -> vk::BufferMemoryBarrier2<'static> {
        vk::BufferMemoryBarrier2::default()
            .buffer(self.buffer.vk_handle())
            .src_stage_mask(self.src_stage_mask.to_vk())
            .src_access_mask(self.src_access_mask.to_vk())
            .dst_stage_mask(self.dst_stage_mask.to_vk())
            .dst_access_mask(self.dst_access_mask.to_vk())
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .offset(self.offset)
            .size(self.size)
    }
}

pub struct ImageMemoryBarrier2 {
    pub image: Arc<Image>,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
}

impl ImageMemoryBarrier2 {
    pub fn to_vk(self) -> vk::ImageMemoryBarrier2<'static> {
        vk::ImageMemoryBarrier2::default()
            .image(self.image.vk_handle())
            .src_stage_mask(self.src_stage_mask.to_vk())
            .src_access_mask(self.src_access_mask.to_vk())
            .dst_stage_mask(self.dst_stage_mask.to_vk())
            .dst_access_mask(self.dst_access_mask.to_vk())
            .old_layout(self.old_layout.to_vk())
            .new_layout(self.new_layout.to_vk())
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageSubresourceLayers {
    pub aspect_mask: ImageAspectFlags,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl ImageSubresourceLayers {
    pub fn to_vk(self) -> vk::ImageSubresourceLayers {
        vk::ImageSubresourceLayers {
            aspect_mask: self.aspect_mask.to_vk(),
            mip_level: self.mip_level,
            base_array_layer: self.base_array_layer,
            layer_count: self.layer_count,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferImageCopy {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceLayers,
    pub image_offset: [i32; 3],
    pub image_extent: Extent3D,
}


impl BufferImageCopy {
    pub fn to_vk(self) -> vk::BufferImageCopy {
        vk::BufferImageCopy {
            buffer_offset: self.buffer_offset,
            buffer_row_length: self.buffer_row_length,
            buffer_image_height: self.buffer_image_height,
            image_subresource: self.image_subresource.to_vk(),
            image_offset: vk::Offset3D {
                x: self.image_offset[0],
                y: self.image_offset[1],
                z: self.image_offset[2],
            },
            image_extent: self.image_extent.to_vk(),
        }
    }
}

