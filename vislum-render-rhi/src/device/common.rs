use ash::vk::{self, PhysicalDeviceFaultFeaturesEXT, PhysicalDeviceProperties2};

use crate::{Version, impl_extensions};

impl_extensions! {
    pub struct DeviceExtensions {
        khr_swapchain => ash::khr::swapchain::NAME,
        khr_synchronization2 => ash::khr::synchronization2::NAME,
        khr_dynamic_rendering => ash::khr::dynamic_rendering::NAME,
        khr_ext_descriptor_indexing => ash::ext::descriptor_indexing::NAME,
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalDeviceProperties {
    pub api_version: Version,
    pub driver_version: Version,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: vk::PhysicalDeviceType,
    pub device_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct QueueFamilyProperties {
    pub queue_flags: vk::QueueFlags,
    pub queue_count: u32,
}
