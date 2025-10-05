use std::sync::Arc;

use ash::vk;

use crate::rhi::{Instance};

#[derive(Debug)]
pub struct QueueFamily {
    /// The index of the queue family.
    index: u32,

    /// The flags that the queue family supports.
    flags: vk::QueueFlags,

    /// The number of queues in the family.
    queue_count: u32,
}

pub struct PhysicalDevice {
    /// The instance that this physical device belongs to.
    instance: Arc<Instance>,

    /// The raw Vulkan physical device handle.
    handle: vk::PhysicalDevice,
}

pub struct PhysicalDeviceProperties {
    dynamic_rendering: vk::PhysicalDeviceDynamicRenderingFeatures<'static>,
    synchronization2: vk::PhysicalDeviceSynchronization2Features<'static>,
    extended_dynamic_state: vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT<'static>,

}

impl PhysicalDevice {
    pub(crate) fn new(instance: Arc<Instance>, physical_device: vk::PhysicalDevice) -> Self {
        todo!()
    }
}

static_assertions::assert_impl_all!(PhysicalDevice: Send, Sync);
