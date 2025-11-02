use std::{collections::HashMap, sync::Arc};

use vulkano::{
    image::{Image, ImageLayout},
    sync::{AccessFlags, ImageMemoryBarrier, PipelineStages},
};

struct ImageState {
    initial_layout: ImageLayout,
    current_layout: ImageLayout,
    current_stages: PipelineStages,
    current_access: AccessFlags,
}

#[derive(Default)]
pub struct ResourceStateTracker {
    images: HashMap<Arc<Image>, ImageState>,
}

impl ResourceStateTracker {
    /// Determines appropriate source stages and access flags based on the current layout.
    fn layout_to_stages_access(layout: ImageLayout) -> (PipelineStages, AccessFlags) {
        match layout {
            ImageLayout::Undefined | ImageLayout::Preinitialized => {
                (PipelineStages::TOP_OF_PIPE, AccessFlags::empty())
            }
            ImageLayout::General => {
                (PipelineStages::ALL_COMMANDS, AccessFlags::MEMORY_WRITE | AccessFlags::MEMORY_READ)
            }
            ImageLayout::ColorAttachmentOptimal => {
                (
                    PipelineStages::COLOR_ATTACHMENT_OUTPUT,
                    AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
            }
            ImageLayout::DepthStencilAttachmentOptimal | ImageLayout::DepthAttachmentOptimal => {
                (
                    PipelineStages::EARLY_FRAGMENT_TESTS | PipelineStages::LATE_FRAGMENT_TESTS,
                    AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                )
            }
            ImageLayout::DepthStencilReadOnlyOptimal => {
                (
                    PipelineStages::FRAGMENT_SHADER,
                    AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                )
            }
            ImageLayout::ShaderReadOnlyOptimal => {
                (PipelineStages::FRAGMENT_SHADER, AccessFlags::SHADER_READ)
            }
            ImageLayout::TransferSrcOptimal => {
                (PipelineStages::ALL_COMMANDS, AccessFlags::TRANSFER_READ)
            }
            ImageLayout::TransferDstOptimal => {
                (PipelineStages::ALL_COMMANDS, AccessFlags::TRANSFER_WRITE)
            }
            ImageLayout::PresentSrc => {
                (PipelineStages::BOTTOM_OF_PIPE, AccessFlags::empty())
            }
            _ => {
                // Fallback for unknown layouts
                (PipelineStages::TOP_OF_PIPE, AccessFlags::empty())
            }
        }
    }

    pub fn transition_image_layout(
        &mut self,
        image: Arc<Image>,
        dst_stages: PipelineStages,
        dst_access: AccessFlags,
        new_layout: ImageLayout,
    ) -> Option<ImageMemoryBarrier> {
        let image_state = self
            .images
            .entry(image.clone())
            .or_insert_with(|| {
                let initial_layout = image.initial_layout();
                let (stages, access) = Self::layout_to_stages_access(initial_layout);
                ImageState {
                    initial_layout,
                    current_layout: initial_layout,
                    current_stages: stages,
                    current_access: access,
                }
            });

        // No need to transition if the layout is already the same.
        if image_state.current_layout == new_layout {
            return None;
        }

        // Use stored source stages/access, or derive from current layout
        let (src_stages, src_access) = if image_state.current_layout == image.initial_layout()
            && image_state.current_stages == PipelineStages::TOP_OF_PIPE
        {
            // First transition from initial state
            Self::layout_to_stages_access(image_state.current_layout)
        } else {
            (image_state.current_stages, image_state.current_access)
        };

        let barrier = ImageMemoryBarrier {
            image: image.clone(),
            new_layout,
            src_stages,
            src_access,
            dst_stages,
            dst_access,
            old_layout: image_state.current_layout,
            ..ImageMemoryBarrier::image(image)
        };

        // Update state
        image_state.current_layout = new_layout;
        image_state.current_stages = dst_stages;
        image_state.current_access = dst_access;

        Some(barrier)
    }
}
