use crate::{vk_enum, vk_enum_flags, image::Extent2D};
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
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    pub fn from_vk(viewport: vk::Viewport) -> Self {
        Self {
            x: viewport.x,
            y: viewport.y,
            width: viewport.width,
            height: viewport.height,
            min_depth: viewport.min_depth,
            max_depth: viewport.max_depth,
        }
    }

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

