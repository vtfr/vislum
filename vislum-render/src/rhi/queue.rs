use std::sync::Arc;

use ash::vk;

use crate::rhi::device::Device;

pub struct Queue {
    pub device: Arc<Device>,
    pub vk: vk::Queue,
}


impl Queue {
    pub(in crate::rhi) fn new(device: Arc<Device>, vk: vk::Queue) -> Self {
        Self { device, vk }
    }
}
