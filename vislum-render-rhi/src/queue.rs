use std::sync::Arc;

use ash::vk;

use crate::{
    AshHandle, DebugWrapper, VkHandle,
    command::RawCommandBuffer,
    device::Device,
    sync::{Fence, Semaphore},
};

pub struct Queue {
    device: Arc<Device>,
    queue: DebugWrapper<vk::Queue>,
}

impl Queue {
    pub fn new(device: Arc<Device>, queue: vk::Queue) -> Self {
        Self {
            device,
            queue: DebugWrapper(queue),
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Submits a command buffer to this queue.
    pub fn submit(
        &self,
        command_buffer: impl Into<RawCommandBuffer>,
        wait_semaphores: Vec<Arc<Semaphore>>,
        signal_semaphores: Vec<Arc<Semaphore>>,
        fence: Option<Arc<Fence>>,
    ) {
        let command_buffer = command_buffer.into().vk_handle();

        let wait_semaphore_handles: Vec<_> =
            wait_semaphores.iter().map(|s| s.vk_handle()).collect();
        let wait_dst_stage_masks: Vec<_> =
            vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; wait_semaphore_handles.len()];

        let signal_semaphore_handles: Vec<_> =
            signal_semaphores.iter().map(|s| s.vk_handle()).collect();

        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&command_buffers)
            .wait_semaphores(&wait_semaphore_handles)
            .wait_dst_stage_mask(&wait_dst_stage_masks)
            .signal_semaphores(&signal_semaphore_handles);

        let fence_handle = fence.map(|f| f.vk_handle()).unwrap_or(vk::Fence::null());

        unsafe {
            self.device
                .ash_handle()
                .queue_submit(self.queue.0, &[submit_info], fence_handle)
                .unwrap();
        }
    }
}

impl VkHandle for Queue {
    type Handle = vk::Queue;

    fn vk_handle(&self) -> Self::Handle {
        self.queue.0
    }
}
