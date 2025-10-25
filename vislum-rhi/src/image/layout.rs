use ash::vk;

use crate::vk_enum;

vk_enum! {
    pub enum ImageLayout: vk::ImageLayout {
        Undefined = UNDEFINED,
        General = GENERAL,
        ColorAttachment = COLOR_ATTACHMENT_OPTIMAL,
        DepthStencilAttachment = DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        DepthStencilReadOnly = DEPTH_STENCIL_READ_ONLY_OPTIMAL,
        ShaderReadOnly = SHADER_READ_ONLY_OPTIMAL,
        TransferSrc = TRANSFER_SRC_OPTIMAL,
        TransferDst = TRANSFER_DST_OPTIMAL,
        Preinitialized = PREINITIALIZED,
        DepthReadOnlyStencilAttachment = DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
        DepthAttachmentStencilReadOnly = DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
        DepthAttachment = DEPTH_ATTACHMENT_OPTIMAL,
        DepthReadOnly = DEPTH_READ_ONLY_OPTIMAL,
        StencilAttachment = STENCIL_ATTACHMENT_OPTIMAL,
        StencilReadOnly = STENCIL_READ_ONLY_OPTIMAL,
        ReadOnly = READ_ONLY_OPTIMAL,
        Attachment = ATTACHMENT_OPTIMAL,
        Present = PRESENT_SRC_KHR,
    }
}

