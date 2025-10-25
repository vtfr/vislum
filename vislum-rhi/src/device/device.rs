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
}

impl AshHandle for Device {
    type Handle = ash::Device;

    #[inline]
    fn ash_handle(&self) -> Self::Handle {
        self.inner.clone()
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

        let create_info =
            vk::DeviceCreateInfo::default().enabled_extension_names(&*enabled_extension_names);

        let mut ffi = DevicePhysicalFeaturesFFI::default();
        let create_info = ffi.wire_to_device_create_info(
            api_version,
            &enabled_extensions,
            &enabled_features,
            create_info,
        );

        let device = unsafe {
            instance
                .instance()
                .create_device(physical_device.vk_handle(), &create_info, None)
                .unwrap()
        };

        Arc::new(Device {
            physical_device,
            inner: device,
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.inner.destroy_device(None);
        }
    }
}
