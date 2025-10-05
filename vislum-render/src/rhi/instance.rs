use std::collections::HashSet;
use std::ffi::CStr;
use std::sync::Arc;
use std::{borrow::Cow, ffi::CString};

use ash::vk;

use crate::rhi::device::Device;
use crate::rhi::physical::PhysicalDevice;

pub struct Instance {
    entry: ash::Entry,
    handle: ash::Instance,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateInstanceError {
    #[error("Failed to load Vulkan entry: {0}")]
    Loading(#[from] ash::LoadingError),

    #[error("Failed to create Vulkan instance: {0}")]
    InstanceCreation(ash::vk::Result),

    #[error("Failed to enumerate physical devices: {0}")]
    PhysicalDevicesEnumeration(ash::vk::Result),

    #[error("Failed to enumerate instance extensions: {0}")]
    InstanceExtensionEnumeration(ash::vk::Result),
}

/// Description for a Vulkan instance.
pub struct InstanceDescription<'a> {
    /// The name of the application.
    pub application_name: Cow<'a, str>,
}

impl Instance {
    pub fn new(description: InstanceDescription) -> Result<Arc<Instance>, CreateInstanceError> {
        let entry = unsafe { ash::Entry::load() }.map_err(CreateInstanceError::Loading)?;

        // Convert the application name to a CString.
        let application_name = CString::new(description.application_name.as_ref())
            .expect("Failed to convert application name to CString");

        // Prepare the application info.
        let application_info = vk::ApplicationInfo::default()
            .application_name(&*application_name)
            .api_version(vk::API_VERSION_1_3);

        let mut extensions = vec![Extension::from(
            vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME,
        )];

        // Request the surface extension.
        extensions.push(Extension::from(vk::KHR_SURFACE_NAME));

        // Request the platform-specific surface extension.
        if cfg!(target_os = "linux") {
            extensions.push(Extension::from(vk::KHR_WAYLAND_SURFACE_NAME));
            extensions.push(Extension::from(vk::KHR_XLIB_SURFACE_NAME));
        } else if cfg!(target_os = "windows") {
            extensions.push(Extension::from(vk::KHR_WIN32_SURFACE_NAME));
        } else {
            unimplemented!("Only Linux and Windows are supported for now");
        }

        check_instance_extensions_supported(&entry, &extensions).unwrap();

        let extension_names = extensions
            .iter()
            .map(|extension| extension.as_ptr())
            .collect::<Vec<_>>();

        // Prepare the instance create info.
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&*extension_names);

        // Create the instance.
        let instance = unsafe { entry.create_instance(&create_info, None) }
            .map_err(CreateInstanceError::InstanceCreation)?;

        Ok(Arc::new(Instance {
            entry,
            handle: instance,
        }))
    }

    /// Enumerates the physical devices available on the system.
    ///
    /// [`PhysicalDevice`]s are ordered based on the provided requirements,
    /// and ordered based on their fitness.
    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Vec<PhysicalDevice> {
        let raw_physical_devices = match unsafe { self.handle.enumerate_physical_devices() } {
            Ok(raw_physical_devices) => raw_physical_devices,
            Err(e) => {
                log::error!("Failed to enumerate physical devices: {:#?}", e);
                return vec![];
            }
        };

        raw_physical_devices
            .into_iter()
            .filter_map(|physical_device| PhysicalDevice::new(self.clone(), physical_device))
            .collect()
    }

    pub fn create_device(self: &Arc<Self>, physical_device: PhysicalDevice) -> Device {
        let mut vulkan_1_3_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);

        let mut vk_features = vk::PhysicalDeviceFeatures2::default()
            .features(vk::PhysicalDeviceFeatures::default())
            .push_next(&mut vulkan_1_3_features);
        let queue_priorities = [1.0];
        let queue_create_info = [
            vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&queue_priorities)
        ];

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_info)
            .push_next(&mut vk_features);

        let device = unsafe { self.handle.create_device(physical_device.vk_physical_device(), &create_info, None) }
            .unwrap();

        Device { device, physical_device }
    }

    /// Returns the Ash Vulkan instance handle.
    pub(crate) fn ash_handle(&self) -> &ash::Instance {
        &self.handle
    }
}

/// Checks if the instance extensions are supported.
///
/// Returns a vector of extension names that are not supported.
fn check_instance_extensions_supported(
    entry: &ash::Entry,
    extensions: &[Extension],
) -> Result<(), Vec<Extension>> {
    let supported_extensions_properties =
        unsafe { entry.enumerate_instance_extension_properties(None) }.unwrap_or_default();

    let supported_extensions: HashSet<Extension> = supported_extensions_properties
        .into_iter()
        .map(|extension| Extension::from(extension.extension_name))
        .collect();

    log::info!("Supported extensions: {supported_extensions:#?}");

    let mut missing_extensions = vec![];

    for extension in extensions {
        if !supported_extensions.contains(&extension) {
            missing_extensions.push(extension.clone());
        }
    }

    missing_extensions
        .is_empty()
        .then_some(())
        .ok_or(missing_extensions)
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.handle.destroy_instance(None) }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct Extension(Cow<'static, CStr>);

impl Extension {
    /// Returns the pointer to the extension name.
    ///
    /// SAFETY: The pointer is valid for the lifetime of the extension.
    pub fn as_ptr(&self) -> *const std::ffi::c_char {
        match &self.0 {
            Cow::Borrowed(cstr) => cstr.as_ptr(),
            Cow::Owned(cstr) => cstr.as_ptr(),
        }
    }

    /// Returns the extension name as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            Cow::Borrowed(cstr) => cstr.to_bytes(),
            Cow::Owned(cstr) => cstr.to_bytes(),
        }
    }

    /// Returns the extension name as a string.
    pub fn as_str(&self) -> &str {
        // SAFETY: The extension name is a valid UTF-8 encoded C string.
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl std::fmt::Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Extension({})", self.as_str())
    }
}

impl From<&'static CStr> for Extension {
    fn from(value: &'static CStr) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<CString> for Extension {
    fn from(value: CString) -> Self {
        Self(Cow::Owned(value))
    }
}

impl From<[std::ffi::c_char; vk::MAX_EXTENSION_NAME_SIZE]> for Extension {
    fn from(bytes: [std::ffi::c_char; vk::MAX_EXTENSION_NAME_SIZE]) -> Self {
        // SAFETY: The array is a valid UTF-8 encoded C string.
        let cstr = unsafe { CStr::from_ptr(bytes.as_ptr()) };

        // We don't own the CStr and it's a temporary value bound to the stack,
        // so we have to clone it.
        Self(Cow::Owned(cstr.to_owned()))
    }
}
