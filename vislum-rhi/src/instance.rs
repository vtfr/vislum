use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, device::physical::PhysicalDevice, impl_extensions, version::Version};

pub struct Library {
    inner: ash::Entry,
}

impl Library {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: unsafe { ash::Entry::load().unwrap() },
        })
    }

    #[inline]
    pub fn entry(&self) -> &ash::Entry {
        &self.inner
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

    fn ash_handle(&self) -> Self::Handle {
        self.instance.clone()
    }
}

impl VkHandle for Instance {
    type Handle = vk::Instance;

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
    pub fn new(library: Arc<Library>, extensions: InstanceExtensions) -> Arc<Self> {
        // Enumerate the instance version. Fallback to V1.0 if the function is not available
        // or has returned an error.
        let instance_version =
            Self::try_enumerate_instance_version(library.entry()).unwrap_or(Version::V1_0);

        // Enumerate the available extensions.
        let available_extensions = Self::enumerate_instance_extension(library.entry());

        // Compute the required extensions for the given instance version and requested extensions.
        let required_extensions =
            Self::compute_required_extensions(instance_version).combine(&available_extensions);

        // If the required extensions are not available, panic.
        let missing_extensions = required_extensions.difference(&available_extensions);
        if !missing_extensions.is_empty() {
            panic!(
                "Required extensions are not available: {:?}",
                missing_extensions
            );
        }

        let enabled_extension_names = required_extensions
            .to_vk()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let application_info = vk::ApplicationInfo::default().api_version(Version::V1_3.to_vk());

        let create_info = vk::InstanceCreateInfo::default()
            .enabled_extension_names(&*enabled_extension_names)
            .application_info(&application_info);

        let instance = unsafe { library.entry().create_instance(&create_info, None).unwrap() };

        let khr_surface = if extensions.khr_surface {
            Some(ash::khr::surface::Instance::new(library.entry(), &instance))
        } else {
            None
        };

        Arc::new(Self {
            library,
            instance,
            khr_surface,
        })
    }

    #[inline]
    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    #[inline]
    pub fn entry(&self) -> &ash::Entry {
        self.library.entry()
    }

    #[inline]
    pub fn ash_khr_surface(&self) -> &ash::khr::surface::Instance {
        self.khr_surface.as_ref().expect("khr_surface extension not enabled")
    }

    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Vec<Arc<PhysicalDevice>> {
        let raw_physical_devices = unsafe { self.instance().enumerate_physical_devices().unwrap() };

        raw_physical_devices
            .into_iter()
            .map(|physical_device| PhysicalDevice::from_vk(Arc::clone(self), physical_device))
            .collect()
    }

    fn try_enumerate_instance_version(library: &ash::Entry) -> Option<Version> {
        match unsafe { library.try_enumerate_instance_version() } {
            Ok(Some(version)) => Some(Version::from_vk(version)),
            Ok(None) => None,
            Err(result) => None,
        }
    }

    fn enumerate_instance_extension(library: &ash::Entry) -> InstanceExtensions {
        let extension_properties =
            unsafe { library.enumerate_instance_extension_properties(None) }.unwrap_or_default();

        InstanceExtensions::from_vk(
            extension_properties
                .iter()
                .map(|properties| properties.extension_name_as_c_str().unwrap()),
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
