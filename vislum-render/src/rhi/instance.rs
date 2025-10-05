use std::ffi::CStr;
use std::sync::Arc;
use std::{borrow::Cow, ffi::CString};

use anyhow::Context;
use ash::{khr, vk};

use crate::rhi::device::Device;
use crate::rhi::physical::PhysicalDevice;

pub struct Instance {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) surface: khr::surface::Instance,
    pub(crate) swapchain: khr::swapchain::Instance,
    pub(crate) get_physical_device_properties2: Option<khr::get_physical_device_properties2::Instance>,
    pub(crate) extensions: Vec<&'static CStr>,
}

/// Description for a Vulkan instance.
pub struct InstanceDescription<'a> {
    /// The name of the application.
    pub application_name: Cow<'a, str>,
}

impl Instance {
    pub fn new(description: InstanceDescription) -> anyhow::Result<Arc<Instance>> {
        let entry = unsafe { ash::Entry::load() }.with_context(|| "failed to load vulkan entry")?;

        // Enumerate the instance version.
        let instance_version = match unsafe { entry.try_enumerate_instance_version() } {
            Ok(Some(version)) => version,
            Ok(None) => vk::API_VERSION_1_0,
            Err(error) => {
                anyhow::bail!("failed to enumerate instance version: {error:?}");
            }
        };

        // Compute the extensions that are required for the instance.
        let extensions = Self::available_desired_extensions(&entry);

        let instance = {
            // Convert the application name to a CString.
            let application_name = CString::new(description.application_name.as_ref())
                .with_context(|| "failed to convert application name to CString")?;

            // Prepare the application info.
            let application_info = vk::ApplicationInfo::default()
                .application_name(&*application_name)
                .api_version(if instance_version < vk::API_VERSION_1_1 {
                    // If the instance version is less than 1.1, we use 1.0.
                    vk::API_VERSION_1_0
                } else {
                    vk::API_VERSION_1_3
                });

            let extensions_ptrs = extensions
                .iter()
                .map(|name| name.as_ptr())
                .collect::<Vec<_>>();

            // Prepare the instance create info.
            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&application_info)
                .enabled_extension_names(&*extensions_ptrs);

            // Create the instance.
            unsafe { entry.create_instance(&create_info, None) }
                .with_context(|| "failed to call create_instance()")?
        };

        let surface = khr::surface::Instance::new(&entry, &instance);
        let swapchain = khr::swapchain::Instance::new(&entry, &instance);
        let get_physical_device_properties2 = extensions
            .contains(&khr::get_physical_device_properties2::NAME)
            .then(|| khr::get_physical_device_properties2::Instance::new(&entry, &instance));

        Ok(Arc::new(Instance {
            entry,
            instance,
            extensions,
            surface,
            swapchain,
            get_physical_device_properties2,
        }))
    }

    /// Enumerates the physical devices available on the system.
    ///
    /// [`PhysicalDevice`]s are ordered based on the provided requirements,
    /// and ordered based on their fitness.
    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Vec<PhysicalDevice> {
        let raw_physical_devices = match unsafe { self.instance.enumerate_physical_devices() } {
            Ok(raw_physical_devices) => raw_physical_devices,
            Err(e) => {
                log::error!("Failed to enumerate physical devices: {:#?}", e);
                return vec![];
            }
        };

        raw_physical_devices
            .into_iter()
            .map(|physical_device| PhysicalDevice::new(self.clone(), physical_device))
            .collect()
    }

    /// Computes the extensions that are required for the instance,
    /// and the platform-specific surface extension.
    ///
    /// If an extension is not available, it will be removed from the list.
    fn available_desired_extensions(entry: &ash::Entry) -> Vec<&'static CStr> {
        let mut extensions = vec![
            ash::khr::get_physical_device_properties2::NAME,
            ash::khr::dynamic_rendering::NAME,
            ash::khr::synchronization2::NAME,
            ash::khr::surface::NAME,
            ash::khr::swapchain::NAME,
        ];

        // Request the platform-specific surface extension.
        if cfg!(target_os = "linux") {
            extensions.push(khr::wayland_surface::NAME);
            extensions.push(khr::xlib_surface::NAME);
        } else if cfg!(target_os = "windows") {
            extensions.push(khr::win32_surface::NAME);
        }

        let supported_extensions =
            unsafe { entry.enumerate_instance_extension_properties(None) }.unwrap_or_default();

        // Retain only the extensions that are supported by the instance.
        extensions.retain(|extension| {
            supported_extensions
                .iter()
                .any(|ep| ep.extension_name_as_c_str() == Ok(extension))
        });

        extensions
    }
    pub fn create_device(self: &Arc<Self>, physical_device: PhysicalDevice) -> Device {
        // let mut vulkan_1_3_features = vk::PhysicalDeviceVulkan13Features::default()
        //     .dynamic_rendering(true)
        //     .synchronization2(true);

        // let mut vk_features = vk::PhysicalDeviceFeatures2::default()
        //     .features(vk::PhysicalDeviceFeatures::default())
        //     .push_next(&mut vulkan_1_3_features);
        // let queue_priorities = [1.0];
        // let queue_create_info = [
        //     vk::DeviceQueueCreateInfo::default()
        //     .queue_family_index(0)
        //     .queue_priorities(&queue_priorities)
        // ];

        // let create_info = vk::DeviceCreateInfo::default()
        //     .queue_create_infos(&queue_create_info)
        //     .push_next(&mut vk_features);

        // let device = unsafe { self.instance.create_device(physical_device.vk_physical_device(), &create_info, None) }
        //     .unwrap();

        // Device { device, physical_device }
        todo!()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) }
    }
}

// #[derive(PartialEq, Eq, Hash, Clone)]
// #[repr(transparent)]
// pub struct Extension(Cow<'static, CStr>);

// impl Extension {
//     /// Returns the pointer to the extension name.
//     ///
//     /// SAFETY: The pointer is valid for the lifetime of the extension.
//     pub fn as_ptr(&self) -> *const std::ffi::c_char {
//         match &self.0 {
//             Cow::Borrowed(cstr) => cstr.as_ptr(),
//             Cow::Owned(cstr) => cstr.as_ptr(),
//         }
//     }

//     /// Returns the extension name as a byte slice.
//     pub fn as_bytes(&self) -> &[u8] {
//         match &self.0 {
//             Cow::Borrowed(cstr) => cstr.to_bytes(),
//             Cow::Owned(cstr) => cstr.to_bytes(),
//         }
//     }

//     /// Returns the extension name as a string.
//     pub fn as_str(&self) -> &str {
//         // SAFETY: The extension name is a valid UTF-8 encoded C string.
//         unsafe { str::from_utf8_unchecked(self.as_bytes()) }
//     }
// }

// impl std::fmt::Display for Extension {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.as_str())
//     }
// }

// impl std::fmt::Debug for Extension {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Extension({})", self.as_str())
//     }
// }

// impl From<&'static CStr> for Extension {
//     fn from(value: &'static CStr) -> Self {
//         Self(Cow::Borrowed(value))
//     }
// }

// impl From<CString> for Extension {
//     fn from(value: CString) -> Self {
//         Self(Cow::Owned(value))
//     }
// }

// impl From<[std::ffi::c_char; vk::MAX_EXTENSION_NAME_SIZE]> for Extension {
//     fn from(bytes: [std::ffi::c_char; vk::MAX_EXTENSION_NAME_SIZE]) -> Self {
//         // SAFETY: The array is a valid UTF-8 encoded C string.
//         let cstr = unsafe { CStr::from_ptr(bytes.as_ptr()) };

//         // We don't own the CStr and it's a temporary value bound to the stack,
//         // so we have to clone it.
//         Self(Cow::Owned(cstr.to_owned()))
//     }
// }
