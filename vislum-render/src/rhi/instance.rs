use std::sync::Arc;
use std::{borrow::Cow, ffi::CString};

use ash::vk;

use crate::rhi::device::{Device, CreateDeviceError};
use crate::rhi::physical::PhysicalDevice;

pub struct Instance {
    _entry: ash::Entry,
    pub(crate) handle: ash::Instance,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateInstanceError {
    #[error("Failed to load Vulkan entry: {0}")]
    Loading(#[from] ash::LoadingError),

    #[error("Failed to create Vulkan instance: {0}")]
    InstanceCreation(ash::vk::Result),

    #[error("Failed to enumerate physical devices: {0}")]
    PhysicalDevicesEnumeration(ash::vk::Result),
}

pub struct InstanceFeatures {
    /// Whether the instance will be used for rendering to a display.
    pub surface: bool,
}

/// Description for a Vulkan instance.
pub struct InstanceDescription<'a> {
    pub application_name: Cow<'a, str>,
    pub features: InstanceFeatures,
}

impl Instance {
    pub fn new(description: InstanceDescription) -> Result<Arc<Instance>, CreateInstanceError> {
        let entry = unsafe { ash::Entry::load() }
            .map_err(CreateInstanceError::Loading)?;

        // Convert the application name to a CString.
        let application_name = CString::new(description.application_name.as_ref())
            .expect("Failed to convert application name to CString");

        // Prepare the application info.
        let application_info = vk::ApplicationInfo::default()
            .application_name(&*application_name)
            .api_version(vk::make_api_version(1, 3, 0, 0));

        // Prepare the instance create info.
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info);

        // Enumerate the instance extensions.
        let extensions = unsafe { entry.enumerate_instance_extension_properties(None) }
            .unwrap_or_default();

        // Create the instance.
        let handle = unsafe { entry.create_instance(&create_info, None) }
            .map_err(CreateInstanceError::InstanceCreation)?;

        Ok(Arc::new(Instance { _entry: entry, handle }))
    }

    /// Enumerates the physical devices available on the system.
    /// 
    /// Physical devices are the devices that can be used to render, and must be selected by the application
    /// based on their capabilities.
    /// 
    /// When no physical devices are available, an empty vector is returned.
    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Vec<PhysicalDevice> {
        let raw_physical_devices = match unsafe { self.handle.enumerate_physical_devices() } {
            Ok(raw_physical_devices) => raw_physical_devices,
            Err(e) => {
                log::error!("Failed to enumerate physical devices: {:#?}", e);
                return vec![];
            }
        };

        raw_physical_devices.into_iter()
            .map(|physical_device| {
                PhysicalDevice::new(self.clone(), physical_device)
            })
            .collect()
    }

    /// Creates a new device from a physical device.
    pub fn create_device(&self, physical_device: PhysicalDevice) -> Result<Device, CreateDeviceError> {
        // let create_info = vk::DeviceCreateInfo::default();

        // let device = unsafe { self.handle.create_device(physical_device.vk_physical_device(), &create_info, None) }
        //     .map_err(CreateDeviceError::FailedCreatingVulkanDevice)?;

        // Ok(Device { device })
        todo!()
    }

    /// Returns the Ash Vulkan instance handle.
    pub(crate) fn ash_handle(&self) -> &ash::Instance {
        &self.handle
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.handle.destroy_instance(None) }
    }
}
