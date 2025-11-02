use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use vulkano::{
    command_buffer::PrimaryAutoCommandBuffer, image::Image, sync::{GpuFuture, semaphore::Semaphore}
};

pub mod window;

/// A struct that represents a frame index.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct FrameIndex(pub u8);

impl Into<usize> for FrameIndex {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Into<u32> for FrameIndex {
    fn into(self) -> u32 {
        self.0 as u32
    }
}

/// A struct that represents an acquired image.
pub struct AcquiredImage {
    /// The image that was acquired.
    pub image: Arc<Image>,

    /// The frame index of the image.
    pub index: FrameIndex,

    /// The semaphore that was used to acquire the image.
    /// 
    /// This semaphore should be waited on before submitting the command buffer 
    /// to the queue.
    pub acquire_semaphore: Arc<Semaphore>,

    /// This semaphore should be signaled when the command buffer finishes
    /// rendering, so the presentation engine can use the image.
    pub render_finished_semaphore: Arc<Semaphore>,
}

#[derive(Error, Debug)]
pub enum ScreenError {
    #[error("validation error: {0}")]
    ValidationError(#[from] vulkano::Validated<vulkano::VulkanError>),
}

/// A trait that abstracts both swapchain-based rendering (windows) and offscreen rendering (textures).
pub trait Screen {
    /// Acquires the next image to be drawn into.
    fn acquire_image(&self) -> Result<AcquiredImage, ScreenError>;

    /// Presents the image at the given index.
    fn present(
        &self,
        queue: Arc<vulkano::device::Queue>,
        index: FrameIndex,
    ) -> Result<(), ScreenError>;
}
