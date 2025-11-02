use std::{cell::OnceCell, sync::Arc};

use ash::vk;
use smallvec::SmallVec;

use crate::{AshDebugWrapper, AshHandle, device::PhysicalDevice, impl_extensions};

impl_extensions! {
    pub struct InstanceExtensions {
        khr_surface => ash::khr::surface::NAME,
        khr_wayland_surface => ash::khr::wayland_surface::NAME,
        khr_xlib_surface => ash::khr::xlib_surface::NAME,
        khr_xcb_surface => ash::khr::xcb_surface::NAME,
        khr_win32_surface => ash::khr::win32_surface::NAME,
    }
}

pub struct Library {
    pub(crate) entry: ash::Entry,
}

impl Library {
    pub fn new() -> Arc<Library> {
        let entry = unsafe { ash::Entry::load() }.expect("failed to load vulkan library");
        Arc::new(Library { entry })
    }
}

pub struct Instance {
    entry: Arc<Library>,
    instance: AshDebugWrapper<ash::Instance>,
    physical_devices: OnceCell<SmallVec<[Arc<PhysicalDevice>; 2]>>,
}

impl Instance {
    pub fn new(entry: Arc<Library>, extensions: InstanceExtensions) -> Arc<Self> {
        let application_info = vk::ApplicationInfo::default()
            .application_name(c"Vislum RHI Demo")
            .application_version(vk::API_VERSION_1_0)
            .api_version(vk::API_VERSION_1_3)
            .engine_name(c"Vislum RHI");

        let enabled_extensions = extensions.iter_c_ptrs().collect::<Vec<_>>();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&*enabled_extensions);

        let instance = unsafe { entry.entry.create_instance(&create_info, None) }.unwrap();

        Arc::new(Self {
            entry,
            instance: AshDebugWrapper(instance),
            physical_devices: Default::default(),
        })
    }

    pub fn enumerate_physical_devices(
        self: &Arc<Self>,
    ) -> impl ExactSizeIterator<Item = Arc<PhysicalDevice>> {
        let physical_devices = self.physical_devices.get_or_init(|| {
            let physical_devices = unsafe { self.instance.enumerate_physical_devices() }.unwrap();

            physical_devices
                .into_iter()
                .map(|physical_device| PhysicalDevice::from_raw(Arc::clone(&self), physical_device))
                .collect()
        });

        physical_devices.iter().cloned()
    }

    /// Returns the library (entry) associated with this instance.
    pub fn library(&self) -> &Arc<Library> {
        &self.entry
    }
}

impl AshHandle for Instance {
    type Handle = ash::Instance;

    fn ash_handle(&self) -> &Self::Handle {
        &self.instance
    }
}
