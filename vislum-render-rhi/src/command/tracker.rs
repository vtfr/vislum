// use std::{collections::HashMap, sync::Arc};

// use crate::{command::{AccessFlags2, ImageLayout, ImageMemoryBarrier2, PipelineStageFlags2}, image::{Image, image::ImageId}};

// #[derive(Copy, Clone)]
// pub(crate) struct ImageState {
//     pub id: ImageId,
//     /// The layout of the image.
//     pub current_layout: ImageLayout,
//     /// The access flags of the image last seen.
//     pub last_seen_access: AccessFlags2,
//     /// The stage of the image last seen.
//     pub last_seen_stage: PipelineStageFlags2,
// }

// #[derive(Default)]
// pub struct ResourceStateTracker {
//     images: HashMap<ImageId, ImageState>,
// }

// impl ResourceStateTracker {
//     /// Marks an image as seen.
//     pub fn transition_image(
//         &mut self, 
//         image: Arc<Image>, 
//         layout: ImageLayout,
//         access: AccessFlags2, 
//         stage: PipelineStageFlags2
//     ) -> Option<ImageMemoryBarrier2> {
//         let state = self.resolve_image_state(&image);
//         let (new_state, barrier) = transition_image(image, *state, ImageState {
//             current_layout: layout,
//             last_seen_access: access,
//             last_seen_stage: stage,
//         });

//         *state = new_state;
//         barrier
//     }

//     fn resolve_image_state(&mut self, image: &Arc<Image>) -> &mut ImageState {
//         let id = image.id();
//         self.images.entry(id)
//             .or_insert_with(|| {
//                 ImageState {
//                     current_layout: ImageLayout::Undefined,
//                     last_seen_access: AccessFlags2::NONE,
//                     last_seen_stage: PipelineStageFlags2::NONE,
//                 }
//             })
//     }
// }

// // The natural order of the pipeline stages used for drawing.
// const DRAW_PIPELINE_STAGES: [PipelineStageFlags2; 5] = [
//     PipelineStageFlags2::TOP_OF_PIPE,
//     PipelineStageFlags2::FRAGMENT_SHADER,
//     PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
//     PipelineStageFlags2::EARLY_FRAGMENT_TESTS,
//     PipelineStageFlags2::LATE_FRAGMENT_TESTS,
// ];

// fn transition_image(
//     image: Arc<Image>,
//     current_state: ImageState,
//     desired_state: ImageState,
// ) -> (ImageState, Option<ImageMemoryBarrier2>) {
//     // No transition needed. Image is in the correct layout.
//     //
//     // Just update when it was last seen.
//     if current_state.current_layout == desired_state.current_layout {
//         return (desired_state, None);
//     }

//     // Find the index of the desired stage in the draw pipeline stages.
//     //
//     // If the desired stage is not found, use the top of the pipeline to signal
//     // that the barrier is split between the top of the pipeline and the desired stage.
//     let (last_seen_stage, last_seen_access) = DRAW_PIPELINE_STAGES
//         .iter()
//         .copied()
//         .rev()
//         .find(|stage| (desired_state.last_seen_stage.contains(*stage)))
//         .map(|stage| (stage, desired_state.last_seen_access))
//         .unwrap_or((PipelineStageFlags2::TOP_OF_PIPE, AccessFlags2::NONE));

//     // Transition needed. Generate the barrier.
//     let barrier = ImageMemoryBarrier2 {
//         image,
//         src_stage_mask: current_state.last_seen_stage,
//         src_access_mask: current_state.last_seen_access,
//         dst_stage_mask: last_seen_stage,
//         dst_access_mask: last_seen_access,
//         old_layout: current_state.current_layout,
//         new_layout: desired_state.current_layout,
//     };

//     (desired_state, Some(barrier))
// }

// fn next_image_stage(last_seen_stage: PipelineStageFlags2) -> PipelineStageFlags2 {
//     macro_rules! c {
//         ($expr:expr) => {
//             last_seen_stage.contains($expr)
//         };
//     };

//     if c!(PipelineStageFlags2::VERTEX_SHADER) {
//         return PipelineStageFlags2::FRAGMENT_SHADER
//     }
    
//     if c!(PipelineStageFlags2::FRAGMENT_SHADER) {
//         PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT
//     }
    
//     if c!(PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT) {
//         PipelineStageFlags2::EARLY_FRAGMENT_TESTS
//     }
    
//     if c!(PipelineStageFlags2::EARLY_FRAGMENT_TESTS) {
//         PipelineStageFlags2::LATE_FRAGMENT_TESTS
//     }
    
//     if c!(PipelineStageFlags2::LATE_FRAGMENT_TESTS) {
//         PipelineStageFlags2::ALL_COMMANDS
//     }
    
//     PipelineStageFlags2::TOP_OF_PIPE
// }