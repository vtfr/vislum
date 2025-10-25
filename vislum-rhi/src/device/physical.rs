use std::sync::Arc;

use ash::vk;

use crate::{VkHandle, device::ffi::{DeviceExtensions, DeviceFeatures, DevicePhysicalFeaturesFFI}, impl_extensions, impl_features, instance::Instance, version::Version, vk_enum};

vk_enum! {
    pub enum PhysicalDeviceType: vk::PhysicalDeviceType {
        Other = OTHER,
        IntegratedGpu = INTEGRATED_GPU,
        DiscreteGpu = DISCRETE_GPU,
        VirtualGpu = VIRTUAL_GPU,
        Cpu = CPU,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalDeviceProperties {
    pub api_version: Version,
    pub driver_version: Version,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: PhysicalDeviceType,
    pub device_name: String,
}

pub struct PhysicalDevice {
    instance: Arc<Instance>,
    physical_device: vk::PhysicalDevice,
    device_extensions: DeviceExtensions,
    device_properties: PhysicalDeviceProperties,
    device_features: DeviceFeatures,
}

impl VkHandle for PhysicalDevice {
    type Handle = vk::PhysicalDevice;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.physical_device
    }
}

impl std::fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalDevice")
            .field("instance", &self.instance.instance().handle())
            .field("physical_device", &self.physical_device)
            .field("device_extensions", &self.device_extensions)
            .field("device_properties", &self.device_properties)
            .field("device_features", &self.device_features)
            .finish()
    }
}

impl PhysicalDevice {
    pub(crate) fn from_vk(
        instance: Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> Arc<Self> {
        let device_extensions = Self::enumerate_device_extensions(&instance, physical_device);
        let device_properties = Self::enumerate_device_properties(&instance, physical_device);
        let device_features = Self::enumerate_device_features(
            &instance,
            device_properties.api_version,
            &device_extensions,
            physical_device,
        );

        Arc::new(Self {
            instance,
            physical_device,
            device_extensions,
            device_properties,
            device_features,
        })
    }

    #[inline]
    pub fn device_extensions(&self) -> &DeviceExtensions {
        &self.device_extensions
    }

    #[inline]
    pub fn device_properties(&self) -> &PhysicalDeviceProperties {
        &self.device_properties
    }

    #[inline]
    pub fn device_features(&self) -> &DeviceFeatures {
        &self.device_features
    }

    fn enumerate_device_properties(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> PhysicalDeviceProperties {
        let properties = unsafe {
            instance
                .instance()
                .get_physical_device_properties(physical_device)
        };

        PhysicalDeviceProperties {
            api_version: Version::from_vk(properties.api_version),
            driver_version: Version::from_vk(properties.driver_version),
            vendor_id: properties.vendor_id,
            device_id: properties.device_id,
            device_type: PhysicalDeviceType::from_vk(properties.device_type),
            device_name: properties
                .device_name_as_c_str()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        }
    }

    fn enumerate_device_extensions(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> DeviceExtensions {
        let extension_properties = unsafe {
            instance
                .instance()
                .enumerate_device_extension_properties(physical_device)
        }
        .unwrap_or_default();

        DeviceExtensions::from_vk(
            extension_properties
                .iter()
                .map(|property| property.extension_name_as_c_str().unwrap()),
        )
    }

    fn enumerate_device_features(
        instance: &Arc<Instance>,
        api_version: Version,
        device_extensions: &DeviceExtensions,
        physical_device: vk::PhysicalDevice,
    ) -> DeviceFeatures {
        let mut ffi = DevicePhysicalFeaturesFFI::default();
        let mut physical_device_features2 = ffi.wire_to_physical_features2(api_version, device_extensions, Default::default());

        unsafe {
            instance.instance()
                .get_physical_device_features2(physical_device, &mut physical_device_features2);
        }

        ffi.supported_features()
    }
}
