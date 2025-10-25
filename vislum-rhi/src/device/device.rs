use std::sync::Arc;

use ash::vk;

use crate::{
    VkHandle,
    device::{ffi::{DeviceExtensions, DeviceFeatures, DevicePhysicalFeaturesFFI}, physical::PhysicalDevice},
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
    device: vk::Device,
}

impl VkHandle for Device {
    type Handle = vk::Device;

    fn vk_handle(&self) -> Self::Handle {
        self.device
    }
}

impl Device {
    pub fn new(
        instance: Arc<Instance>,
        physical_device: Arc<PhysicalDevice>,
        create_info: DeviceCreateInfo,
    ) -> Self {
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

        let enabled_extension_names = enabled_extensions.to_vk()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let create_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&*enabled_extension_names);

        let mut ffi = DevicePhysicalFeaturesFFI::default();
        let create_info = ffi.wire_to_device_create_info(api_version, &enabled_extensions, &enabled_features, create_info);

        let device = unsafe {
            instance
                .instance()
                .create_device(physical_device.vk_handle(), &create_info, None)
                .unwrap()
        };

        // Arm the bomb.
        //
        // From now one we're guaranteed that the device is valid, so it must be destroyed
        // if any of the following operations fail.
        let mut maybe_drop_device = MaybeDrop::new(move || unsafe {
            device.destroy_device(None);
        });

        // Everything went well. Disarm.
        maybe_drop_device.disarm();

        // Device { physical_device }
        todo!()
    }
}

struct MaybeDrop<F: FnOnce()>(Option<F>);

impl<F> MaybeDrop<F>
where
    F: FnOnce(),
{
    pub fn new(f: F) -> Self {
        Self(Some(f))
    }

    // Disarms the bomb.
    pub fn disarm(&mut self) {
        self.0 = None;
    }
}

impl<F> Drop for MaybeDrop<F>
where
    F: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}
