use std::sync::Arc;

use ash::vk;

use crate::{
    AshHandle, VkHandle,
    device::{
        ffi::{DeviceExtensions, DeviceFeatures, DevicePhysicalFeaturesFFI},
        physical::PhysicalDevice,
    },
    instance::Instance,
    version::Version,
};

pub struct DeviceCreateInfo {
    pub api_version: Version,
    pub enabled_extensions: DeviceExtensions,
    pub enabled_features: DeviceFeatures,
}

pub struct Device {
    physical_device: Arc<PhysicalDevice>,
    inner: ash::Device,
    khr_swapchain: Option<ash::khr::swapchain::Device>,
}

impl AshHandle for Device {
    type Handle = ash::Device;

    #[inline]
    fn ash_handle(&self) -> &Self::Handle {
        &self.inner
    }
}

impl VkHandle for Device {
    type Handle = vk::Device;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.inner.handle()
    }
}

impl Device {
    #[inline]
    pub fn physical_device(&self) -> &Arc<PhysicalDevice> {
        &self.physical_device
    }

    pub fn new(
        instance: Arc<Instance>,
        physical_device: Arc<PhysicalDevice>,
        create_info: DeviceCreateInfo,
    ) -> Arc<Self> {
        let DeviceCreateInfo {
            api_version,
            enabled_extensions,
            enabled_features,
        } = create_info;

        let device_extensions = physical_device.device_extensions();
        let device_features = physical_device.device_features();

        let missing_extensions = enabled_extensions.difference(device_extensions);
        if !missing_extensions.is_empty() {
            panic!("Missing extensions: {:?}", missing_extensions);
        }

        let missing_features = enabled_features.difference(device_features);
        if !missing_features.is_empty() {
            // TODO: error handing ;)
            panic!("Missing features: {:?}", missing_features);
        }

        let enabled_extension_names = enabled_extensions
            .to_vk()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        // Request a queue from family 0 (assuming it exists and supports graphics)
        let queue_priorities = [1.0];
        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&queue_priorities);

        let create_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&*enabled_extension_names)
            .queue_create_infos(std::slice::from_ref(&queue_create_info));

        let mut ffi = DevicePhysicalFeaturesFFI::default();
        let create_info = ffi.wire_to_device_create_info(
            api_version,
            &enabled_extensions,
            &enabled_features,
            create_info,
        );

        let device = unsafe {
            instance
                .ash_handle()
                .create_device(physical_device.vk_handle(), &create_info, None)
                .unwrap()
        };

        let khr_swapchain = if enabled_extensions.khr_swapchain {
            Some(ash::khr::swapchain::Device::new(
                instance.ash_handle(),
                &device,
            ))
        } else {
            None
        };

        Arc::new(Device {
            physical_device,
            inner: device,
            khr_swapchain,
        })
    }

    #[inline]
    pub fn ash_khr_swapchain(&self) -> &ash::khr::swapchain::Device {
        self.khr_swapchain
            .as_ref()
            .expect("khr_swapchain extension not enabled")
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.inner.destroy_device(None);
        }
    }
}
