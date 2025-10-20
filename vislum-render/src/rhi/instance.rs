use std::{fmt::Debug, sync::Arc};

use ash::{ext, khr, vk};

use crate::{
    new_extensions_struct,
    rhi::{
        device::{Device, DeviceDescription, DeviceError}, physical::PhysicalDevice,
        surface::Surface,
        util::Version,
    },
};

new_extensions_struct! {
    pub(in crate::rhi) struct InstanceExtensions {
        /// Surface support for presenting rendered images
        khr_surface => khr::surface::NAME,

        /// Wayland surface support
        khr_wayland_surface => khr::wayland_surface::NAME,

        /// X11/Xlib surface support
        khr_xlib_surface => khr::xlib_surface::NAME,

        /// XCB surface support
        khr_xcb_surface => khr::xcb_surface::NAME,

        /// Win32 surface support
        khr_win32_surface => khr::win32_surface::NAME,

        /// Debug utilities for validation and debugging
        ext_debug_utils => ext::debug_utils::NAME,

        /// Extended device property queries (many extensions depend on this)
        khr_get_physical_device_properties2 => khr::get_physical_device_properties2::NAME,

        /// Needed for querying advanced surface details
        khr_get_surface_capabilities2 => khr::get_surface_capabilities2::NAME,

        /// Adds HDR/wide color support
        ext_swapchain_colorspace => ext::swapchain_colorspace::NAME,

        /// Required when running on macOS (MoltenVK)
        khr_portability_enumeration => khr::portability_enumeration::NAME,
    }
}

/// The description of the instance to create
pub struct InstanceDescription {
    /// The extensions to enable for the instance.
    pub extensions: InstanceExtensions,
}

pub struct Instance {
    entry: ash::Entry,

    /// The instance.
    instance: ash::Instance,

    /// The KHR_get_physical_device_properties2 instance extension.
    ///
    /// Loaded for non Vulkan 1.1 instances.
    khr_get_physical_device_properties2_handle:
        Option<khr::get_physical_device_properties2::Instance>,

    /// The KHR_surface instance extension.
    ///
    /// Used for rendering to surfaces (i.e. windows).
    khr_surface_handle: Option<khr::surface::Instance>,

    /// The version of the instance.
    version: Version,

    /// The extensions enabled for the instance.
    extensions: InstanceExtensions,
}

impl Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("instance", &self.instance.handle())
            .field("version", &self.version)
            .field("extensions", &self.extensions)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("missing extensions: {0}")]
    MissingExtensions(InstanceExtensions),
}

impl Instance {
    pub fn new() -> Result<Arc<Self>, InstanceError> {
        log::debug!("Loading Vulkan entrypoint");
        let entry = unsafe { ash::Entry::load() }.unwrap();

        // Get the instance version
        let instance_version = match unsafe { entry.try_enumerate_instance_version() } {
            Ok(Some(version)) => Version::from_vk(version),
            Ok(None) => Version::VERSION_1_0,
            Err(e) => {
                log::error!("failed to enumerate instance version. Falling back to 1.0.0: {e}");
                Version::VERSION_1_0
            }
        };

        // Cap the version to 1.3.0
        let instance_version = instance_version.min(Version::VERSION_1_3);

        // Prepare the required instance extensions.
        let required_extensions = InstanceExtensions {
            khr_get_physical_device_properties2: instance_version < Version::VERSION_1_1,
            // Inclue the surface extensions for rendering to surfaces (i.e. windows).
            ..Surface::required_instance_extensions()
        };

        let supported_instance_extensions = Self::enumerate_supported_extensions(&entry);
        log::debug!(
            "Instance supported extensions: {:?}",
            supported_instance_extensions
        );

        let missing_extensions = supported_instance_extensions.difference(&required_extensions);
        if !missing_extensions.is_empty() {
            return Err(InstanceError::MissingExtensions(missing_extensions));
        }

        let extension_names = required_extensions
            .to_vk_extension_names()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let application_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&*extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None) }.unwrap();

        let khr_get_physical_device_properties2_handle = required_extensions
            .khr_get_physical_device_properties2
            .then(|| khr::get_physical_device_properties2::Instance::new(&entry, &instance));

        let khr_surface_handle = required_extensions
            .khr_surface
            .then(|| khr::surface::Instance::new(&entry, &instance));

        Ok(Arc::new(Self {
            entry,
            instance,
            khr_get_physical_device_properties2_handle,
            khr_surface_handle,
            version: instance_version,
            extensions: required_extensions,
        }))
    }

    #[inline]
    pub(in crate::rhi) fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    #[inline]
    pub(in crate::rhi) fn handle(&self) -> &ash::Instance {
        &self.instance
    }

    /// The "KHR_get_physical_device_properties2" instance extension.
    ///
    /// When this object is [`Some`], callers should use methods provided by this instance,
    /// instead of those available on the [`ash::Instance`] returned by [`Instance::handle()`].
    #[inline]
    pub(in crate::rhi) fn khr_get_physical_device_properties2_handle(
        &self,
    ) -> Option<&khr::get_physical_device_properties2::Instance> {
        self.khr_get_physical_device_properties2_handle.as_ref()
    }

    /// The "KHR_surface" instance extension.
    #[inline]
    pub(in crate::rhi) fn khr_surface_handle(&self) -> Option<&khr::surface::Instance> {
        self.khr_surface_handle.as_ref()
    }

    #[inline]
    pub(in crate::rhi) fn version(&self) -> Version {
        self.version
    }

    #[inline]
    pub(in crate::rhi) fn extensions(&self) -> &InstanceExtensions {
        &self.extensions
    }

    /// Enumerate the physical devices on the instance.
    ///
    /// Filters based on the minimum requirements for rendering.
    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Vec<Arc<PhysicalDevice>> {
        let vk_physical_devices = unsafe { self.instance.enumerate_physical_devices().unwrap() };

        vk_physical_devices
            .into_iter()
            .filter_map(|vk_physical_device| {
                PhysicalDevice::new(Arc::clone(self), vk_physical_device)
            })
            .collect()
    }

    /// Create a new device.
    pub fn create_device(
        self: &Arc<Self>,
        device_description: DeviceDescription,
    ) -> Result<Arc<Device>, DeviceError> {
        Device::new(Arc::clone(self), device_description)
    }

    /// Enumerate the supported extensions for the instance.
    fn enumerate_supported_extensions(entry: &ash::Entry) -> InstanceExtensions {
        // Store the extension properties.
        let extension_properties = unsafe {
            entry
                .enumerate_instance_extension_properties(None)
                .unwrap_or_default()
        };

        // SAFETY: the extension properties can be casted as a &CStr with the same lifetime as
        // the extension_properties variable.
        //
        // Since this is only used in this scope, this is safe.
        let extension_names = extension_properties
            .iter()
            .map(|property| property.extension_name_as_c_str().unwrap());

        InstanceExtensions::from_vk_extension_names(extension_names)
    }
}
