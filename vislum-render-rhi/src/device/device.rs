use std::sync::Arc;

use ash::vk;

use crate::{
    AshDebugWrapper, AshHandle, Version, VkHandle,
    device::{DeviceExtensions, DeviceFeatures, PhysicalDevice, PhysicalDeviceFeaturesFfi},
    instance::Instance,
};

pub struct Device {
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    device: AshDebugWrapper<ash::Device>,
}

impl AshHandle for Device {
    type Handle = ash::Device;

    fn ash_handle(&self) -> &Self::Handle {
        &self.device
    }
}

pub struct DeviceCreateInfo {
    pub api_version: Version,
    pub physical_device: Arc<PhysicalDevice>,
    pub extensions: DeviceExtensions,
    pub features: DeviceFeatures,
}

impl Device {
    /// Creates a new device.
    pub fn new(instance: Arc<Instance>, create_info: DeviceCreateInfo) -> Arc<Self> {
        let queue_priorities = [1.0];

        let queue_create_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_priorities(&queue_priorities)
            .queue_family_index(0)];

        let enabled_extension_names = create_info.extensions.iter_c_ptrs().collect::<Vec<_>>();

        let vk_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&enabled_extension_names);

        let mut features_ffi = PhysicalDeviceFeaturesFfi::default();
        let vk_create_info = features_ffi.wire_to_create_info(
            create_info.api_version,
            &create_info.extensions,
            &create_info.features,
            vk_create_info,
        );

        let vk_physical_device = create_info.physical_device.vk_handle();
        let device = unsafe {
            instance
                .ash_handle()
                .create_device(vk_physical_device, &vk_create_info, None)
        }
        .unwrap();

        Arc::new(Self {
            instance,
            physical_device: create_info.physical_device,
            device: AshDebugWrapper(device),
        })
    }

    /// Returns the instance associated with the device.
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    /// Returns the physical device associated with the device.
    pub fn physical_device(&self) -> &Arc<PhysicalDevice> {
        &self.physical_device
    }
}
