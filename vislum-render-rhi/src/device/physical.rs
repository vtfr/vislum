use std::{cell::OnceCell, sync::Arc};

use ash::vk;
use smallvec::SmallVec;

use crate::{
    AshHandle, Version, VkHandle,
    device::{
        DeviceExtensions, DeviceFeatures, PhysicalDeviceFeaturesFfi,
        PhysicalDeviceProperties, PhysicalDeviceType, QueueFamilyProperties, QueueFlags,
    },
    instance::Instance,
};

pub struct PhysicalDevice {
    instance: Arc<Instance>,
    physical_device: vk::PhysicalDevice,
    properties: OnceCell<PhysicalDeviceProperties>,
    capabilities: OnceCell<SmallVec<[QueueFamilyProperties; 8]>>,
    extensions: OnceCell<DeviceExtensions>,
}

impl VkHandle for PhysicalDevice {
    type Handle = vk::PhysicalDevice;

    fn vk_handle(&self) -> Self::Handle {
        self.physical_device
    }
}

impl PhysicalDevice {
    pub fn from_raw(instance: Arc<Instance>, physical_device: vk::PhysicalDevice) -> Arc<Self> {
        Arc::new(Self {
            instance,
            physical_device,
            properties: Default::default(),
            capabilities: Default::default(),
            extensions: Default::default(),
        })
    }

    /// Returns the properties of the physical device.
    pub fn properties(&self) -> &PhysicalDeviceProperties {
        self.properties.get_or_init(|| {
            let properties = unsafe {
                self.instance
                    .ash_handle()
                    .get_physical_device_properties(self.physical_device)
            };

            let device_name = properties
                .device_name_as_c_str()
                .ok()
                .map(std::ffi::CStr::to_string_lossy)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "<unknown>".into());

            PhysicalDeviceProperties {
                api_version: Version::from_vk(properties.api_version),
                driver_version: Version::from_vk(properties.driver_version),
                vendor_id: properties.vendor_id,
                device_id: properties.device_id,
                device_type: PhysicalDeviceType::from_vk(properties.device_type)
                    .unwrap_or(PhysicalDeviceType::OTHER),
                device_name,
            }
        })
    }

    /// Returns the capabilities of the physical device.
    pub fn capabilities(&self) -> impl ExactSizeIterator<Item = QueueFamilyProperties> {
        let capabilities = self.capabilities.get_or_init(|| {
            let capabilities = unsafe {
                self.instance
                    .ash_handle()
                    .get_physical_device_queue_family_properties(self.physical_device)
            };

            capabilities
                .into_iter()
                .map(|properties| QueueFamilyProperties {
                    queue_flags: QueueFlags::from_vk(properties.queue_flags),
                    queue_count: properties.queue_count,
                })
                .collect()
        });

        capabilities.iter().copied()
    }

    /// Returns the extensions supported by the physical device.
    pub fn extensions(&self) -> &DeviceExtensions {
        self.extensions.get_or_init(|| {
            let properties = unsafe {
                self.instance
                    .ash_handle()
                    .enumerate_device_extension_properties(self.physical_device)
                    .unwrap_or_default()
            };

            let extension_names = properties
                .iter()
                .map(|extension| extension.extension_name_as_c_str().unwrap());

            DeviceExtensions::from_iter(extension_names)
        })
    }

    /// Returns the features supported by the physical device.
    /// 
    /// These are computed based on the promoted physical device features 
    /// up to the physical device's API version and supported extensions.
    pub fn supported_features(&self) -> DeviceFeatures {
        let api_version = self.properties().api_version;
        let extensions = self.extensions();

        let mut features_ffi = PhysicalDeviceFeaturesFfi::default();
        let mut vk_features =
            features_ffi.wire_to_properties(api_version, extensions, Default::default());

        unsafe {
            self.instance
                .ash_handle()
                .get_physical_device_features2(self.physical_device, &mut vk_features);
        };

        features_ffi.into_device_features()
    }
}
