use std::sync::Arc;

use ash::vk;

use crate::{
    AshHandle, Error, VkHandle, WithContext, device::physical::PhysicalDevice, impl_extensions,
    version::Version,
};

pub struct Library {
    inner: ash::Entry,
}

impl AshHandle for Library {
    type Handle = ash::Entry;

    #[inline]
    fn ash_handle(&self) -> &Self::Handle {
        &self.inner
    }
}

impl Library {
    pub fn new() -> Result<Arc<Self>, Error> {
        let entry = unsafe { ash::Entry::load() }.map_err(|_| Error::Vulkan {
            context: "failed to load vulkan entry".into(),
            result: ash::vk::Result::ERROR_INITIALIZATION_FAILED,
        })?;
        Ok(Arc::new(Self { inner: entry }))
    }
}

impl_extensions! {
    pub struct InstanceExtensions {
        pub khr_surface = ash::khr::surface::NAME,
        pub khr_win32_surface = ash::khr::win32_surface::NAME,
        pub khr_xlib_surface = ash::khr::xlib_surface::NAME,
        pub khr_xcb_surface = ash::khr::xcb_surface::NAME,
        pub khr_wayland_surface = ash::khr::wayland_surface::NAME,
        pub khr_get_physical_device_properties2 = ash::khr::get_physical_device_properties2::NAME,
    }
}

pub struct Instance {
    library: Arc<Library>,
    instance: ash::Instance,
    khr_surface: Option<ash::khr::surface::Instance>,
}

impl AshHandle for Instance {
    type Handle = ash::Instance;

    #[inline]
    fn ash_handle(&self) -> &Self::Handle {
        &self.instance
    }
}

impl VkHandle for Instance {
    type Handle = vk::Instance;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.instance.handle()
    }
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instance")
    }
}

impl Instance {
    pub fn new(library: Arc<Library>, extensions: InstanceExtensions) -> Result<Arc<Self>, Error> {
        let entry = library.ash_handle();

        // Enumerate the instance version.
        let instance_version = Self::try_enumerate_instance_version(entry);

        // Enumerate the available extensions.
        let available_extensions = Self::enumerate_instance_extension(entry);

        // Compute the required extensions for the given instance version and requested extensions.
        let required_extensions =
            Self::compute_required_extensions(instance_version).combine(&extensions);

        // If the required extensions are not available, return error.
        let missing_extensions = required_extensions.difference(&available_extensions);
        if !missing_extensions.is_empty() {
            return Err(Error::Vulkan {
                context: format!(
                    "required extensions are not available: {:?}",
                    missing_extensions
                )
                .into(),
                result: vk::Result::ERROR_EXTENSION_NOT_PRESENT,
            });
        }

        let enabled_extension_names = required_extensions.to_vk_ptr_names();

        let application_info = vk::ApplicationInfo::default()
            .api_version(instance_version.to_vk())
            .engine_name(c"Vislum Rendering Engine")
            .engine_version(vk::API_VERSION_1_0);

        let create_info = vk::InstanceCreateInfo::default()
            .enabled_extension_names(&*enabled_extension_names)
            .application_info(&application_info);

        let instance = unsafe {
            library
                .ash_handle()
                .create_instance(&create_info, None)
                .with_context("failed to create vulkan instance")?
        };

        let khr_surface = if extensions.khr_surface {
            Some(ash::khr::surface::Instance::new(
                library.ash_handle(),
                &instance,
            ))
        } else {
            None
        };

        Ok(Arc::new(Self {
            library,
            instance,
            khr_surface,
        }))
    }

    #[inline]
    pub(crate) fn library(&self) -> &Arc<Library> {
        &self.library
    }

    #[inline]
    pub(crate) fn ash_khr_surface_handle(&self) -> &ash::khr::surface::Instance {
        self.khr_surface
            .as_ref()
            .expect("khr_surface extension not enabled")
    }

    /// Enumerates the physical devices available on the instance.
    ///
    /// As a convenience, the minimal subset of extensions and features required by the the renderer
    /// will be pre-filtered for the caller. But callers ultimately responsible for checking the
    /// compatibility of the physical devices with the advanced features and extensions required
    /// by the application.
    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Result<Vec<Arc<PhysicalDevice>>, Error> {
        let raw_physical_devices = unsafe {
            self.ash_handle()
                .enumerate_physical_devices()
                .with_context("failed to enumerate physical devices")?
        };

        Ok(raw_physical_devices
            .into_iter()
            .map(|physical_device| PhysicalDevice::from_vk(Arc::clone(self), physical_device))
            .collect())
    }

    /// Tries to enumerate the instance version.
    ///
    /// Fallback to V1.0 if the function is not available or has returned an error.
    fn try_enumerate_instance_version(library: &ash::Entry) -> Version {
        match unsafe { library.try_enumerate_instance_version() } {
            Ok(Some(version)) => Version::from_vk(version),
            Ok(None) => Version::V1_0,
            Err(_) => Version::V1_0,
        }
    }

    fn enumerate_instance_extension(library: &ash::Entry) -> InstanceExtensions {
        let extension_properties =
            unsafe { library.enumerate_instance_extension_properties(None) }.unwrap_or_default();

        InstanceExtensions::from_vk(
            extension_properties
                .iter()
                .filter_map(|properties| properties.extension_name_as_c_str().ok()),
        )
    }

    /// Computes the requires extensions for the given instance version and requested extensions.
    fn compute_required_extensions(instance_version: Version) -> InstanceExtensions {
        let mut required_extensions = InstanceExtensions::default();
        if instance_version < Version::V1_1 {
            required_extensions.khr_get_physical_device_properties2 = true;
        }

        required_extensions
    }
}
