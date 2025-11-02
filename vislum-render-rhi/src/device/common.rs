use crate::{Version, impl_extensions, vk_enum, vk_enum_flags};

impl_extensions! {
    pub struct DeviceExtensions {
        khr_swapchain => ash::khr::swapchain::NAME,
        khr_synchronization2 => ash::khr::synchronization2::NAME,
        khr_dynamic_rendering => ash::khr::dynamic_rendering::NAME,
        khr_ext_descriptor_indexing => ash::ext::descriptor_indexing::NAME,
    }
}

vk_enum! {
    pub enum PhysicalDeviceType: ash::vk::PhysicalDeviceType {
        DISCRETE_GPU => DISCRETE_GPU,
        INTEGRATED_GPU => INTEGRATED_GPU,
        VIRTUAL_GPU => VIRTUAL_GPU,
        CPU => CPU,
        OTHER => OTHER,
    }
}

vk_enum_flags! {
    pub struct QueueFlags: ash::vk::QueueFlags {
        GRAPHICS => GRAPHICS,
        COMPUTE => COMPUTE,
        TRANSFER => TRANSFER,
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalDeviceProperties {
    pub api_version: Version,
    pub driver_version: Version,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: PhysicalDeviceType,
    pub device_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct QueueFamilyProperties {
    pub queue_flags: QueueFlags,
    pub queue_count: u32,
}
