use std::{mem::MaybeUninit, sync::Arc};

use ash::{khr, vk};

use crate::{
    new_extensions_struct,
    rhi::{
        instance::Instance,
        util::{Version, read_into_vec},
    },
};

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
    pub fn from_handle(
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

        let get_physical_device_properties2 =
            match instance.fns().khr_get_physical_device_properties2() {
                Some(fns) => fns.get_physical_device_properties2_khr,
                None => instance.fns().vk_1_1().get_physical_device_properties2,
            };

        unsafe {
            (get_physical_device_properties2)(physical_device, &mut vk_properties);
        }

        // Cap the version to the instance version
        instance
            .version()
            .min(Version::from_vk(vk_properties.properties.api_version))
    }

    fn enumerate_supported_features(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
        extensions: &DeviceExtensions,
    ) -> DeviceFeatures {
        let fns = instance.fns();
        let get_physical_device_features2 = match fns.khr_get_physical_device_properties2() {
            Some(fns) => fns.get_physical_device_features2_khr,
            None => fns.vk_1_1().get_physical_device_features2,
        };

        let mut vk_vulkan_1_1_features = vk::PhysicalDeviceVulkan11Features::default();
        let mut vk_vulkan_1_2_features = vk::PhysicalDeviceVulkan12Features::default();
        let mut vk_vulkan_1_3_features = vk::PhysicalDeviceVulkan13Features::default();
        let mut vk_khr_synchronization2_features =
            vk::PhysicalDeviceSynchronization2Features::default();
        let mut vk_khr_dynamic_rendering_features =
            vk::PhysicalDeviceDynamicRenderingFeatures::default();

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
        features.dynamic_rendering = vk_vulkan_1_3_features.dynamic_rendering == vk::TRUE
            || (extensions.khr_dynamic_rendering
                && vk_khr_dynamic_rendering_features.dynamic_rendering == vk::TRUE);
        features.synchronization2 = vk_vulkan_1_3_features.synchronization2 == vk::TRUE
            || (extensions.khr_synchronization2
                && vk_khr_synchronization2_features.synchronization2 == vk::TRUE);
        features
    }

    fn enumerate_supported_extensions(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
    ) -> DeviceExtensions {
        let extension_properties = unsafe {
            read_into_vec(|count, data| {
                (instance
                    .fns()
                    .vk_1_0()
                    .enumerate_device_extension_properties)(
                    physical_device,
                    std::ptr::null(),
                    count,
                    data,
                )
            })
        };

        let extension_properties = match extension_properties {
            Ok(extension_properties) => extension_properties,
            Err(_) => return DeviceExtensions::default(),
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

pub struct DeviceFns {
    pub(crate) vk_1_0: ash::DeviceFnV1_0,
    pub(crate) vk_1_1: ash::DeviceFnV1_1,
    pub(crate) vk_1_2: ash::DeviceFnV1_2,
    pub(crate) vk_1_3: ash::DeviceFnV1_3,
    pub(crate) khr_synchronization2: Option<khr::synchronization2::DeviceFn>,
    pub(crate) khr_dynamic_rendering: Option<khr::dynamic_rendering::DeviceFn>,
    pub(crate) khr_swapchain: Option<khr::swapchain::DeviceFn>,
}

impl std::fmt::Debug for DeviceFns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceFns").finish_non_exhaustive()
    }
}

impl DeviceFns {
    #[inline]
    pub fn vk_1_0(&self) -> &ash::DeviceFnV1_0 {
        &self.vk_1_0
    }

    #[inline]
    pub fn vk_1_1(&self) -> &ash::DeviceFnV1_1 {
        &self.vk_1_1
    }

    #[inline]
    pub fn vk_1_3(&self) -> &ash::DeviceFnV1_3 {
        &self.vk_1_3
    }

    #[inline]
    pub fn khr_swapchain(&self) -> Option<&khr::swapchain::DeviceFn> {
        self.khr_swapchain.as_ref()
    }

    #[inline]
    pub fn khr_synchronization2(&self) -> Option<&khr::synchronization2::DeviceFn> {
        self.khr_synchronization2.as_ref()
    }

    #[inline]
    pub fn khr_dynamic_rendering(&self) -> Option<&khr::dynamic_rendering::DeviceFn> {
        self.khr_dynamic_rendering.as_ref()
    }
}

#[derive(Debug)]
pub struct Device {
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    handle: vk::Device,
    device_fns: DeviceFns,
    enabled_extensions: DeviceExtensions,
}

pub struct DeviceDescription {
    pub physical_device: Arc<PhysicalDevice>,
    pub features: DeviceFeatures,
    pub extensions: DeviceExtensions,
}

impl Device {
    pub fn new(
        device_description: DeviceDescription,
    ) -> Result<Arc<Self>, DeviceError> {
        Self::check_extension_compatibility(&device_description.physical_device, &device_description)?;

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
            let next = vk_vulkan_1_3_features.insert(
                vk::PhysicalDeviceVulkan13Features::default()
                    .synchronization2(device_description.features.synchronization2)
                    .dynamic_rendering(device_description.features.dynamic_rendering),
            );
            vk_create_info = vk_create_info.push_next(next);
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
        }

        let handle = {
            let mut handle = MaybeUninit::uninit();
            let result = unsafe {
                (instance.fns().vk_1_0().create_device)(
                    physical_device.handle(),
                    &vk_create_info,
                    std::ptr::null(),
                    handle.as_mut_ptr(),
                )
            };
            result.result().unwrap();

            // SAFETY: The device is created successfully.
            unsafe { handle.assume_init() }
        };

        let device_fns = {
            let mut load_fn = |name: &std::ffi::CStr| -> *const std::ffi::c_void {
                unsafe {
                    std::mem::transmute((instance.fns().vk_1_0().get_device_proc_addr)(
                        handle,
                        name.as_ptr(),
                    ))
                }
            };

            DeviceFns {
                vk_1_0: ash::DeviceFnV1_0::load(&mut load_fn),
                vk_1_1: ash::DeviceFnV1_1::load(&mut load_fn),
                vk_1_2: ash::DeviceFnV1_2::load(&mut load_fn),
                vk_1_3: ash::DeviceFnV1_3::load(&mut load_fn),
                khr_synchronization2: (device_description.extensions.khr_synchronization2)
                    .then(|| khr::synchronization2::DeviceFn::load(&mut load_fn)),
                khr_dynamic_rendering: (device_description.extensions.khr_dynamic_rendering)
                    .then(|| khr::dynamic_rendering::DeviceFn::load(&mut load_fn)),
                khr_swapchain: (device_description.extensions.khr_swapchain)
                    .then(|| khr::swapchain::DeviceFn::load(&mut load_fn)),
            }
        };

        Ok(Arc::new(Self {
            instance,
            physical_device,
            handle,
            device_fns,
            enabled_extensions: device_description.extensions,
        }))
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

    #[inline]
    pub fn fns(&self) -> &DeviceFns {
        &self.device_fns
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
