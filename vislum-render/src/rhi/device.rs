use std::{mem::MaybeUninit, sync::Arc};

use ash::{khr, vk};

use crate::{new_extensions_struct, rhi::{instance::Instance, util::{Version, read_into_vec}}};

new_extensions_struct! {
    pub struct DeviceExtensions {
        khr_swapchain => khr::swapchain::NAME,
        khr_synchronization2 => khr::synchronization2::NAME,
        khr_dynamic_rendering => khr::dynamic_rendering::NAME,
        khr_push_descriptor => khr::push_descriptor::NAME,
    }
}

/// The support level for a device feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Support {
    /// The feature is supported by the core Vulkan API.
    Core,
    /// The feature is supported by an extension.
    Extension,
    /// The feature is not supported at all.
    #[default]
    NotSupported,
}

impl Support {
    #[inline]
    pub const fn is_supported(&self) -> bool {
        matches!(self, Support::Core | Support::Extension)
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
    pub fn from_handle(instance: Arc<Instance>, physical_device: vk::PhysicalDevice) -> Option<Arc<Self>> {
        let version = Self::get_physical_device_version(&instance, physical_device);
        let supported_device_extensions = Self::enumerate_supported_extensions(&instance, physical_device);
        let supported_features = Self::enumerate_supported_features(&instance, physical_device, &supported_device_extensions);

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
    fn get_physical_device_version(instance: &Arc<Instance>, physical_device: vk::PhysicalDevice) -> Version {
        let mut vk_properties = vk::PhysicalDeviceProperties2::default();

        let get_physical_device_properties2 = match instance.fns().khr_get_physical_device_properties2() {
            Some(fns) => fns.get_physical_device_properties2_khr,
            None => instance.fns().vk_1_1().get_physical_device_properties2,
        };
    
        unsafe {
            (get_physical_device_properties2)(physical_device, &mut vk_properties);
        }

        // Cap the version to the instance version
        instance.version().min(Version::from_vk(vk_properties.properties.api_version))
    }

    fn enumerate_supported_features(instance: &Arc<Instance>, physical_device: vk::PhysicalDevice, extensions: &DeviceExtensions) -> DeviceFeatures {
        let fns = instance.fns();
        let get_physical_device_features2 = match fns.khr_get_physical_device_properties2() {
            Some(fns) => fns.get_physical_device_features2_khr,
            None => fns.vk_1_1().get_physical_device_features2,
        };

        let mut vk_vulkan_1_1_features = vk::PhysicalDeviceVulkan11Features::default();
        let mut vk_vulkan_1_2_features = vk::PhysicalDeviceVulkan12Features::default();
        let mut vk_vulkan_1_3_features = vk::PhysicalDeviceVulkan13Features::default();
        let mut vk_khr_synchronization2_features = vk::PhysicalDeviceSynchronization2Features::default();
        let mut vk_khr_dynamic_rendering_features = vk::PhysicalDeviceDynamicRenderingFeatures::default();
        
        let mut vk_features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut vk_vulkan_1_1_features)
            .push_next(&mut vk_vulkan_1_2_features)
            .push_next(&mut vk_vulkan_1_3_features)
            .push_next(&mut vk_khr_synchronization2_features)
            .push_next(&mut vk_khr_dynamic_rendering_features);
        
        unsafe {
            (get_physical_device_features2)(physical_device, &mut vk_features);
        }

        let mut features = DeviceFeatures::default();
        features.dynamic_rendering = vk_vulkan_1_3_features.dynamic_rendering == vk::TRUE || (extensions.khr_dynamic_rendering && vk_khr_dynamic_rendering_features.dynamic_rendering == vk::TRUE);
        features.synchronization2 = vk_vulkan_1_3_features.synchronization2 == vk::TRUE || (extensions.khr_synchronization2 && vk_khr_synchronization2_features.synchronization2 == vk::TRUE);
        features
    }

    fn enumerate_supported_extensions(instance: &Arc<Instance>, physical_device: vk::PhysicalDevice) -> DeviceExtensions {
        let extension_properties = unsafe {
            read_into_vec(|count, data| {
                (instance.fns().vk_1_0().enumerate_device_extension_properties)(physical_device, std::ptr::null(), count, data)
            })
        };

        let extension_properties = match extension_properties {
            Ok(extension_properties) => extension_properties,
            Err(_) => return DeviceExtensions::default(),
        };

        let extension_names = extension_properties.iter()
            .map(|ext| ext.extension_name_as_c_str().unwrap());

        DeviceExtensions::from_vk_extension_names(extension_names)
    }
}

#[derive(Debug)]
pub struct Device {
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    handle: vk::Device,
}

pub struct DeviceDescription {
}

impl Device {
    pub fn from_raw(instance: Arc<Instance>, physical_device: Arc<PhysicalDevice>) -> Arc<Self> {
        let vk_queue_family_properties = [1.0f32];
        let vk_queue_create_infos = [
            vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&vk_queue_family_properties),
        ];

        let mut vk_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&vk_queue_create_infos);

        let mut vk_vulkan_1_1_features = None::<vk::PhysicalDeviceVulkan11Features>;
        let mut vk_vulkan_1_2_features = None::<vk::PhysicalDeviceVulkan12Features>;
        let mut vk_vulkan_1_3_features = None::<vk::PhysicalDeviceVulkan13Features>;
        let mut vk_khr_synchronization2_features = None::<vk::PhysicalDeviceSynchronization2Features>;
        let mut vk_khr_dynamic_rendering_features = None::<vk::PhysicalDeviceDynamicRenderingFeatures>;

        // Add features promoted to Vulkan 1.1
        if physical_device.version() >= Version::VERSION_1_1 {
            let next = vk_vulkan_1_1_features.get_or_insert(vk::PhysicalDeviceVulkan11Features::default());
            vk_create_info = vk_create_info.push_next(next);
        }

        // Add features promoted to Vulkan 1.2
        if physical_device.version() >= Version::VERSION_1_2 {
            let next = vk_vulkan_1_2_features.get_or_insert(vk::PhysicalDeviceVulkan12Features::default());
            vk_create_info = vk_create_info.push_next(next);
        }

        // Add features promoted to Vulkan 1.3
        if physical_device.version() >= Version::VERSION_1_3 {
            let next = vk_vulkan_1_3_features.get_or_insert(vk::PhysicalDeviceVulkan13Features::default());
            *next = next.synchronization2(true);
            *next = next.dynamic_rendering(true);
            vk_create_info = vk_create_info.push_next(next);
        } else {
            // Add features from extensions if the physical device does not support Vulkan 1.3
            let next = vk_khr_synchronization2_features.get_or_insert(vk::PhysicalDeviceSynchronization2Features::default());
            *next = next.synchronization2(true);
            vk_create_info = vk_create_info.push_next(next);

            let next = vk_khr_dynamic_rendering_features.get_or_insert(vk::PhysicalDeviceDynamicRenderingFeatures::default());
            *next = next.dynamic_rendering(true);
            vk_create_info = vk_create_info.push_next(next);
        }

        let handle = {
            let mut handle = MaybeUninit::uninit();
            let result= unsafe { (instance.fns().vk_1_0().create_device)(physical_device.handle(), &vk_create_info, std::ptr::null(), handle.as_mut_ptr()) };
            result.result().unwrap();

            // SAFETY: The device is created successfully.
            unsafe { handle.assume_init() }
        };

        Arc::new(Self {
            instance,
            physical_device,
            handle,
        })
    }

    #[inline]
    pub fn handle(&self) -> vk::Device {
        self.handle
    }

    #[inline]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    #[inline]
    pub fn physical_device(&self) -> &Arc<PhysicalDevice> {
        &self.physical_device
    }
}