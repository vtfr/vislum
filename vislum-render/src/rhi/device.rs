use std::sync::Arc;

use ash::{khr, vk};

use crate::{
    new_extensions_struct,
    rhi::{
        instance::Instance,
        util::Version,
    },
};

new_extensions_struct! {
    pub struct DeviceExtensions {
        khr_swapchain => khr::swapchain::NAME,
        khr_synchronization2 => khr::synchronization2::NAME,
        khr_dynamic_rendering => khr::dynamic_rendering::NAME,
        khr_push_descriptor => khr::push_descriptor::NAME,
        ext_extended_dynamic_state => ash::ext::extended_dynamic_state::NAME,
        ext_extended_dynamic_state2 => ash::ext::extended_dynamic_state2::NAME,
        ext_extended_dynamic_state3 => ash::ext::extended_dynamic_state3::NAME,
        khr_acceleration_structure => khr::acceleration_structure::NAME,
        khr_ray_tracing_pipeline => khr::ray_tracing_pipeline::NAME,
        khr_ray_query => khr::ray_query::NAME,
        khr_deferred_host_operations => khr::deferred_host_operations::NAME
    }
}

macro_rules! device_features_impl {
    (
        $(#[$meta:meta])*
        pub struct $ident:ident {
            $(
                pub $field:ident: bool
            ),*
            $(,)?
        }
    ) => {
        $(#[$meta])*
        pub struct $ident {
            $(
                pub $field: bool,
            )*
        }

        impl std::fmt::Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let entries = [
                    $(
                        (stringify!($field), self.$field),
                    )*
                ]
                    .into_iter()
                    .filter_map(|(name, enabled)| enabled.then_some(name));

                f.debug_list().entries(entries).finish()
            }
        }

        impl $ident {
            pub fn difference(&self, other: &Self) -> Self {
                $ident {
                    $(
                        $field: self.$field && !other.$field,
                    )*
                }
            }
        }
    };
}

device_features_impl! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DeviceFeatures {
        pub dynamic_rendering: bool,
        pub synchronization2: bool,
        pub extended_dynamic_state: bool,
    }
}

#[derive(Debug)]
pub struct PhysicalDevice {
    instance: Arc<Instance>,
    physical_device: vk::PhysicalDevice,
    version: Version,
    supported_device_extensions: DeviceExtensions,
    supported_features: DeviceFeatures,
}

impl PhysicalDevice {
    /// Create a new physical device from a handle.
    ///
    /// Returns `None` if the physical device does not meet the minimum requirements for rendering.
    pub(crate) fn from_raw(
        instance: Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> Option<Arc<Self>> {
        let version = Self::get_physical_device_version(&instance, physical_device);
        let supported_device_extensions =
            Self::enumerate_supported_extensions(&instance, physical_device);
        let supported_features = Self::enumerate_supported_features(
            &instance,
            physical_device,
            &supported_device_extensions,
        );

        // We expect the physical device to support dynamic rendering and synchronization2.
        if !supported_features.dynamic_rendering || !supported_features.synchronization2 {
            return None;
        }

        Some(Arc::new(Self {
            instance,
            physical_device,
            version,
            supported_device_extensions,
            supported_features,
        }))
    }

    #[inline]
    pub fn handle(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    #[inline]
    pub fn supported_extensions(&self) -> &DeviceExtensions {
        &self.supported_device_extensions
    }

    #[inline]
    pub fn supported_features(&self) -> &DeviceFeatures {
        &self.supported_features
    }

    /// The version of the physical device, capped to the instance version.
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    #[inline]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    #[inline]
    fn get_physical_device_version(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> Version {
        let mut vk_properties = vk::PhysicalDeviceProperties2::default();

        unsafe {
            match instance.khr_get_physical_device_properties2_instance() {
                Some(instance) => {
                    instance.get_physical_device_properties2(physical_device, &mut vk_properties)
                }
                None => instance
                    .vk()
                    .get_physical_device_properties2(physical_device, &mut vk_properties),
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
        match instance.khr_get_physical_device_properties2_instance() {
            Some(instance) => unsafe {
                instance.get_physical_device_features2(physical_device, &mut vk_features)
            },
            None => unsafe {
                instance
                    .vk()
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
                .vk()
                .enumerate_device_extension_properties(physical_device)
                .unwrap_or_default()
        };

        let extension_names = extension_properties
            .iter()
            .map(|ext| ext.extension_name_as_c_str().unwrap());

        DeviceExtensions::from_vk_extension_names(extension_names)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("missing extensions: {0}")]
    MissingExtensions(DeviceExtensions),
}

pub struct Device {
    instance: Arc<Instance>,

    physical_device: Arc<PhysicalDevice>,

    device: ash::Device,
    khr_synchronization2_device: Option<khr::synchronization2::Device>,
    khr_dynamic_rendering_device: Option<khr::dynamic_rendering::Device>,
    khr_swapchain_device: Option<khr::swapchain::Device>,
    ext_extended_dynamic_state_device: Option<ash::ext::extended_dynamic_state::Device>,

    enabled_extensions: DeviceExtensions,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("instance", &self.instance)
            .field("physical_device", &self.physical_device)
            .field("device", &self.device.handle())
            .field("enabled_extensions", &self.enabled_extensions)
            .finish()
    }
}

pub struct DeviceDescription {
    pub physical_device: Arc<PhysicalDevice>,
    pub features: DeviceFeatures,
    pub extensions: DeviceExtensions,
}

impl Device {
    pub fn new(device_description: DeviceDescription) -> Result<Arc<Self>, DeviceError> {
        Self::check_extension_compatibility(
            &device_description.physical_device,
            &device_description,
        )?;

        let physical_device = device_description.physical_device;
        let instance = Arc::clone(&physical_device.instance);

        let vk_queue_family_properties = [1.0f32];
        let vk_queue_create_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&vk_queue_family_properties)];

        // Enable swapchain extension if supported
        let extension_names = device_description
            .extensions
            .to_vk_extension_names()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let mut vk_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&vk_queue_create_infos)
            .enabled_extension_names(&extension_names);

        let mut vk_vulkan_1_1_features = None::<vk::PhysicalDeviceVulkan11Features>;
        let mut vk_vulkan_1_2_features = None::<vk::PhysicalDeviceVulkan12Features>;
        let mut vk_vulkan_1_3_features = None::<vk::PhysicalDeviceVulkan13Features>;
        let mut vk_khr_synchronization2_features =
            None::<vk::PhysicalDeviceSynchronization2Features>;
        let mut vk_khr_dynamic_rendering_features =
            None::<vk::PhysicalDeviceDynamicRenderingFeatures>;
        let mut vk_ext_extended_dynamic_state_features =
            None::<vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT>;

        // Add features promoted to Vulkan 1.1
        if physical_device.version() >= Version::VERSION_1_1 {
            let next = vk_vulkan_1_1_features.insert(vk::PhysicalDeviceVulkan11Features::default());
            vk_create_info = vk_create_info.push_next(next);
        }

        // Add features promoted to Vulkan 1.2
        if physical_device.version() >= Version::VERSION_1_2 {
            let next = vk_vulkan_1_2_features.insert(vk::PhysicalDeviceVulkan12Features::default());
            vk_create_info = vk_create_info.push_next(next);
        }

        // Add features promoted to Vulkan 1.3
        if physical_device.version() >= Version::VERSION_1_3 {
            vk_create_info = vk_create_info.push_next(
                vk_vulkan_1_3_features.insert(
                    vk::PhysicalDeviceVulkan13Features::default()
                        .synchronization2(device_description.features.synchronization2)
                        .dynamic_rendering(device_description.features.dynamic_rendering),
                ),
            );

            // Add extended dynamic state feature. Supported in Vulkan 1.3.
            vk_create_info = vk_create_info.push_next(
                vk_ext_extended_dynamic_state_features.insert(
                    vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default()
                        .extended_dynamic_state(device_description.features.extended_dynamic_state),
                ),
            );
        } else {
            // Add synchronization2 feature from extension if supported
            if device_description.extensions.khr_synchronization2 {
                let next = vk_khr_synchronization2_features.insert(
                    vk::PhysicalDeviceSynchronization2Features::default()
                        .synchronization2(device_description.features.synchronization2),
                );

                vk_create_info = vk_create_info.push_next(next);
            }

            // Add dynamic rendering feature from extension if supported
            if device_description.extensions.khr_dynamic_rendering {
                let next = vk_khr_dynamic_rendering_features.insert(
                    vk::PhysicalDeviceDynamicRenderingFeatures::default()
                        .dynamic_rendering(device_description.features.dynamic_rendering),
                );

                vk_create_info = vk_create_info.push_next(next);
            }

            // Add extended dynamic state feature.
            if device_description.features.extended_dynamic_state {
                let next = vk_ext_extended_dynamic_state_features.insert(
                    vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default()
                        .extended_dynamic_state(device_description.features.extended_dynamic_state),
                );

                vk_create_info = vk_create_info.push_next(next);
            }
        }

        let device = unsafe {
            instance
                .vk()
                .create_device(physical_device.handle(), &vk_create_info, None)
                .unwrap()
        };

        let khr_synchronization2_device = (device_description.extensions.khr_synchronization2)
            .then(|| khr::synchronization2::Device::new(instance.vk(), &device));
        let khr_dynamic_rendering_device = (device_description.extensions.khr_dynamic_rendering)
            .then(|| khr::dynamic_rendering::Device::new(instance.vk(), &device));
        let khr_swapchain_device = (device_description.extensions.khr_swapchain)
            .then(|| khr::swapchain::Device::new(instance.vk(), &device));
        let ext_extended_dynamic_state_device = (device_description
            .extensions
            .ext_extended_dynamic_state)
            .then(|| ash::ext::extended_dynamic_state::Device::new(instance.vk(), &device));

        Ok(Arc::new(Self {
            instance,
            physical_device,
            device,
            khr_synchronization2_device,
            khr_dynamic_rendering_device,
            khr_swapchain_device,
            ext_extended_dynamic_state_device,
            enabled_extensions: device_description.extensions,
        }))
    }

    #[inline]
    pub fn vk(&self) -> &ash::Device {
        &self.device
    }

    #[inline]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    #[inline]
    pub fn physical_device(&self) -> &Arc<PhysicalDevice> {
        &self.physical_device
    }

    #[inline]
    pub fn khr_synchronization2_device(&self) -> Option<&khr::synchronization2::Device> {
        self.khr_synchronization2_device.as_ref()
    }

    #[inline]
    pub fn khr_dynamic_rendering_device(&self) -> Option<&khr::dynamic_rendering::Device> {
        self.khr_dynamic_rendering_device.as_ref()
    }

    #[inline]
    pub fn khr_swapchain_device(&self) -> Option<&khr::swapchain::Device> {
        self.khr_swapchain_device.as_ref()
    }

    #[inline]
    pub fn ext_extended_dynamic_state_device(&self) -> Option<&ash::ext::extended_dynamic_state::Device> {
        self.ext_extended_dynamic_state_device.as_ref()
    }

    #[inline]
    pub fn extensions(&self) -> &DeviceExtensions {
        &self.enabled_extensions
    }

    fn check_extension_compatibility(
        physical_device: &Arc<PhysicalDevice>,
        device_description: &DeviceDescription,
    ) -> Result<(), DeviceError> {
        let missing_extensions = physical_device
            .supported_extensions()
            .difference(&device_description.extensions);

        if !missing_extensions.is_empty() {
            return Err(DeviceError::MissingExtensions(missing_extensions));
        }

        Ok(())
    }
}
