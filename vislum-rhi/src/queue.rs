use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, device::device::Device};

pub struct Queue {
    device: Arc<Device>,
    queue: vk::Queue,
    family_index: u32,
}

impl VkHandle for Queue {
    type Handle = vk::Queue;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.queue
    }
}

impl Queue {
    pub fn new(device: Arc<Device>, family_index: u32, queue_index: u32) -> Arc<Self> {
        let queue = unsafe {
            device
                .ash_handle()
                .get_device_queue(family_index, queue_index)
        };

        Arc::new(Self {
            device,
            queue,
            family_index,
        })
    }

    pub fn submit(&self, submit_info: &vk::SubmitInfo, fence: vk::Fence) {
        unsafe {
            self.device
                .ash_handle()
                .queue_submit(self.queue, &[*submit_info], fence)
                .expect("Failed to submit queue");
        }
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.device
                .ash_handle()
                .queue_wait_idle(self.queue)
                .expect("Failed to wait for queue idle");
        }
    }

    #[inline]
    pub fn family_index(&self) -> u32 {
        self.family_index
    }
}
