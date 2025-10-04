use std::{num::NonZeroU32, sync::{Arc, OnceLock}};

use ash::vk;

use crate::rhi::Instance;

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
    
    /// The queue family properties of the physical device. 
    queue_families: Vec<QueueFamily>,
    
    /// The memory properties of the physical device. Stored behind a once-lock to enable lazy querying.
    memory_properties: OnceLock<vk::PhysicalDeviceMemoryProperties>,
    
    /// The features of the physical device. Stored behind a once-lock to enable lazy querying.
    features: OnceLock<vk::PhysicalDeviceFeatures>,
}

static_assertions::assert_impl_all!(PhysicalDevice: Send, Sync);

impl std::fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalDevice")
            .field("handle", &self.handle)
            .field("queue_family_properties", &self.queue_families)
            .finish()
    }
}

impl PhysicalDevice {
    /// Creates a new PhysicalDevice from raw Vulkan handles and properties.
    pub(crate) fn new(
        instance: Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        let queue_family_properties = unsafe { 
            instance.handle.get_physical_device_queue_family_properties(physical_device)
                .into_iter()
                .enumerate()
                .map(|(queue_family_index, properties)| QueueFamily {
                    index: queue_family_index as u32,
                    flags: properties.queue_flags,
                    queue_count: properties.queue_count,
                })
                .collect()
        };

        Self {
            handle: physical_device,
            instance,
            queue_families: queue_family_properties,
            memory_properties: Default::default(),
            features: Default::default(),
        }
    }

    /// Returns the memory properties of the physical device.
    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        self.memory_properties.get_or_init(|| unsafe {
            self.instance.ash_handle().get_physical_device_memory_properties(self.handle)
        })
    }

    /// Returns the features of the physical device.
    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        self.features.get_or_init(|| unsafe {
            self.instance.ash_handle().get_physical_device_features(self.handle)
        })
    }
}