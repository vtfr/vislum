use std::{fmt::Debug, sync::Arc};

use ash::{ext, khr, vk};

use crate::{new_extensions_struct, rhi::{debug::debug_trampoline, device::{Device, DeviceDescription, DeviceError, PhysicalDevice}, util::{Version, read_into_vec}}};

new_extensions_struct! {
    pub struct InstanceExtensions {
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

pub struct InstanceFns {
    pub(crate) vk_1_0: ash::InstanceFnV1_0,
    pub(crate) vk_1_1: ash::InstanceFnV1_1,
    pub(crate) vk_1_3: ash::InstanceFnV1_3,
    pub(crate) khr_get_physical_device_properties2: Option<khr::get_physical_device_properties2::InstanceFn>,
    pub(crate) khr_surface: Option<khr::surface::InstanceFn>,
}

impl InstanceFns {
    #[inline]
    pub fn vk_1_0(&self) -> &ash::InstanceFnV1_0 {
        &self.vk_1_0
    }

    #[inline]
    pub fn vk_1_1(&self) -> &ash::InstanceFnV1_1 {
        &self.vk_1_1
    }

    #[inline]
    pub fn vk_1_3(&self) -> &ash::InstanceFnV1_3 {
        &self.vk_1_3
    }

    #[inline]
    pub fn khr_get_physical_device_properties2(&self) -> Option<&khr::get_physical_device_properties2::InstanceFn> {
        self.khr_get_physical_device_properties2.as_ref()
    }

    #[inline]
    pub fn khr_surface(&self) -> Option<&khr::surface::InstanceFn> {
        self.khr_surface.as_ref()
    }
}

/// The description of the instance to create
pub struct InstanceDescription {
    /// The extensions to enable for the instance.
    pub extensions: InstanceExtensions,
}

pub struct Instance {
    pub(crate) entry: ash::Entry,
    instance: vk::Instance,
    instance_fns: InstanceFns,
    instance_version: Version,
    enabled_extensions: InstanceExtensions,
}

impl Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("instance", &self.instance)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("missing extensions: {0}")]
    MissingExtensions(InstanceExtensions),
}

impl Instance {
    pub fn new(description: InstanceDescription) -> Result<Arc<Self>, InstanceError> {
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

        let supported_instance_extensions = Self::enumerate_supported_extensions(&entry);
        log::debug!("Instance supported extensions: {:?}", supported_instance_extensions);

        let missing_extensions = supported_instance_extensions.difference(&description.extensions);
        if !missing_extensions.is_empty() {
            return Err(InstanceError::MissingExtensions(missing_extensions));
        }

        let extension_names = description.extensions.to_vk_extension_names()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let application_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_3);
        
        let mut create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&*extension_names);


        let mut debug_utils_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default();
        if description.extensions.ext_debug_utils {
            debug_utils_create_info.message_severity = vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE;
            debug_utils_create_info.message_type = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL;
            debug_utils_create_info.pfn_user_callback = Some(debug_trampoline);
            create_info = create_info.push_next(&mut debug_utils_create_info);
        }

        let ash_instance = unsafe { entry.create_instance(&create_info, None) }.unwrap();
        let instance = ash_instance.handle();

        let instance_fns = {
            let mut load_fn = load_fn_for(&entry, instance);
            InstanceFns {
                vk_1_0: ash_instance.fp_v1_0().clone(),
                vk_1_1: ash_instance.fp_v1_1().clone(),
                vk_1_3: ash_instance.fp_v1_3().clone(),
                khr_get_physical_device_properties2: Some(khr::get_physical_device_properties2::InstanceFn::load(&mut load_fn)),
                khr_surface: description.extensions.khr_surface.then(|| khr::surface::InstanceFn::load(&mut load_fn)),
           }
        };

        Ok(Arc::new(Self { 
            entry,
            instance: ash_instance.handle(),
            instance_fns,
            instance_version,
            enabled_extensions: description.extensions,
        }))
    }

    #[inline]
    pub fn vk_instance(&self) -> vk::Instance {
        self.instance
    }

    #[inline]
    pub fn fns(&self) -> &InstanceFns {
        &self.instance_fns
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.instance_version
    }

    #[inline]
    pub fn extensions(&self) -> &InstanceExtensions {
        &self.enabled_extensions
    }

    #[inline]
    pub(crate) fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    /// Enumerate the physical devices on the instance.
    /// 
    /// Filters based on the minimum requirements for rendering.
    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Vec<Arc<PhysicalDevice>> {
        let vk_physical_devices = unsafe {
            read_into_vec(|count, data| {
                (self.instance_fns.vk_1_0().enumerate_physical_devices)(self.instance, count, data)
            }).unwrap()
        };
        
        vk_physical_devices
            .into_iter()
            .filter_map(|vk_physical_device| PhysicalDevice::from_handle(Arc::clone(self), vk_physical_device))
            .collect()
    }

    pub fn create_device(self: &Arc<Self>, device_description: DeviceDescription) -> Result<Arc<Device>, DeviceError> {
        Device::new(device_description)
    }

    /// Enumerate the supported extensions for the instance.
    fn enumerate_supported_extensions(entry: &ash::Entry) -> InstanceExtensions {
        let extension_properties = unsafe {
            entry.enumerate_instance_extension_properties(None)
                .unwrap_or_default()
        };

        let extension_names = extension_properties.iter()
            .map(|property| property.extension_name_as_c_str().unwrap());

        InstanceExtensions::from_vk_extension_names(extension_names)
    }
}


pub(in crate::rhi) fn load_fn_for(entry: &ash::Entry, instance: vk::Instance) -> impl FnMut(&std::ffi::CStr) -> *const std::ffi::c_void {
    move |name: &std::ffi::CStr| -> *const std::ffi::c_void {
        unsafe {
            std::mem::transmute(entry.get_instance_proc_addr(instance, name.as_ptr() as *const std::ffi::c_char))
        }
    }
}