use std::sync::Arc;

use ash::vk;

use crate::{DebugWrapper, VkHandle, device::Device};

pub struct Queue {
    device: Arc<Device>,
    queue: DebugWrapper<vk::Queue>,
}

impl Queue {
    pub fn new(device: Arc<Device>, queue: vk::Queue) -> Self {
        Self { device, queue: DebugWrapper(queue) }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl VkHandle for Queue {
    type Handle = vk::Queue;

    fn vk_handle(&self) -> Self::Handle {
        self.queue.0
    }
}