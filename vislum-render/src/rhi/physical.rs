use std::sync::Arc;

use ash::vk;

use crate::rhi::{device::{DeviceExtensions, DeviceFeatures}, instance::Instance, util::Version};


#[derive(Debug)]
pub struct PhysicalDevice {
    instance: Arc<Instance>,
    handle: ash::vk::PhysicalDevice,
    version: Version,
    supported_extensions: DeviceExtensions,
    supported_features: DeviceFeatures,
}

impl PhysicalDevice {
    /// Create a new physical device from a handle.
    ///
    /// Returns `None` if the physical device does not meet the minimum requirements for rendering.
    pub(in crate::rhi) fn new(
        instance: Arc<Instance>,
        handle: vk::PhysicalDevice,
    ) -> Option<Arc<Self>> {
        let version = Self::get_physical_device_version(&instance, handle);
        let supported_device_extensions =
            Self::enumerate_supported_extensions(&instance, handle);
        let supported_features = Self::enumerate_supported_features(
            &instance,
            handle,
            &supported_device_extensions,
        );

        // We expect the physical device to support dynamic rendering and synchronization2.
        if !supported_features.dynamic_rendering || !supported_features.synchronization2 {
            return None;
        }

        Some(Arc::new(Self {
            instance,
            handle,
            version,
            supported_extensions: supported_device_extensions,
            supported_features,
        }))
    }

    #[inline]
    pub(in crate::rhi) fn handle(&self) -> ash::vk::PhysicalDevice {
        self.handle
    }

    #[inline]
    pub(in crate::rhi) fn supported_extensions(&self) -> &DeviceExtensions {
        &self.supported_extensions
    }

    #[inline]
    pub(in crate::rhi) fn supported_features(&self) -> &DeviceFeatures {
        &self.supported_features
    }

    /// The version of the physical device, capped to the instance version.
    #[inline]
    pub(in crate::rhi) fn version(&self) -> Version {
        self.version
    }

    #[inline]
    fn get_physical_device_version(
        instance: &Arc<Instance>,
        handle: ash::vk::PhysicalDevice,
    ) -> Version {
        let mut vk_properties = vk::PhysicalDeviceProperties2::default();

        unsafe {
            match instance.khr_get_physical_device_properties2_handle() {
                Some(instance) => {
                    instance.get_physical_device_properties2(handle, &mut vk_properties)
                }
                None => instance
                    .handle()
                    .get_physical_device_properties2(handle, &mut vk_properties),
            }
        }

        // Cap the version to the instance version
        std::cmp::min(
            instance.version(),
            Version::from_vk(vk_properties.properties.api_version),
        )
    }

    fn enumerate_supported_features(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
        extensions: &DeviceExtensions,
    ) -> DeviceFeatures {
        // Prepare chain of features.
        let mut vk_vulkan_1_1_features = vk::PhysicalDeviceVulkan11Features::default();
        let mut vk_vulkan_1_2_features = vk::PhysicalDeviceVulkan12Features::default();
        let mut vk_vulkan_1_3_features = vk::PhysicalDeviceVulkan13Features::default();
        let mut vk_khr_synchronization2_features =
            vk::PhysicalDeviceSynchronization2Features::default();
        let mut vk_khr_dynamic_rendering_features =
            vk::PhysicalDeviceDynamicRenderingFeatures::default();
        let mut vk_ext_extended_dynamic_state_features =
            vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default();

        let mut vk_features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut vk_vulkan_1_1_features)
            .push_next(&mut vk_vulkan_1_2_features)
            .push_next(&mut vk_vulkan_1_3_features)
            .push_next(&mut vk_khr_synchronization2_features)
            .push_next(&mut vk_khr_dynamic_rendering_features)
            .push_next(&mut vk_ext_extended_dynamic_state_features);

        // Get features.
        match instance.khr_get_physical_device_properties2_handle() {
            Some(instance) => unsafe {
                instance.get_physical_device_features2(physical_device, &mut vk_features)
            },
            None => unsafe {
                instance
                    .handle()
                    .get_physical_device_features2(physical_device, &mut vk_features)
            },
        };

        DeviceFeatures {
            // Dynamic rendering. Promoted in Vulkan 1.3 or extension.
            dynamic_rendering: vk_vulkan_1_3_features.dynamic_rendering == vk::TRUE
                || (extensions.khr_dynamic_rendering
                    && vk_khr_dynamic_rendering_features.dynamic_rendering == vk::TRUE),
            // Synchronization2. Promoted in Vulkan 1.3 or extension.
            synchronization2: vk_vulkan_1_3_features.synchronization2 == vk::TRUE
                || (extensions.khr_synchronization2
                    && vk_khr_synchronization2_features.synchronization2 == vk::TRUE),
            // Extended dynamic state. Promoted in Vulkan 1.3.
            extended_dynamic_state: vk_ext_extended_dynamic_state_features.extended_dynamic_state
                == vk::TRUE,
        }
    }

    fn enumerate_supported_extensions(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> DeviceExtensions {
        let extension_properties = unsafe {
            instance
                .handle()
                .enumerate_device_extension_properties(physical_device)
                .unwrap_or_default()
        };

        let extension_names = extension_properties
            .iter()
            .map(|ext| ext.extension_name_as_c_str().unwrap());

        DeviceExtensions::from_vk_extension_names(extension_names)
    }
}