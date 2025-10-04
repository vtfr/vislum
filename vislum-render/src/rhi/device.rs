use ash::vk;

use crate::rhi::VulkanHandle;

pub struct Device {
    pub device: ash::Device,
}

impl VulkanHandle for Device {
    type Handle = vk::Device;

    fn vk_handle(&self) -> Self::Handle {
        self.device.handle()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateDeviceError {
    #[error("Failed to create Vulkan device: {0}")]
    FailedCreatingVulkanDevice(vk::Result),
}

impl Device {
    /// Returns the raw Vulkan device handle.
    pub fn ash_device(&self) -> &ash::Device {
        &self.device
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { 
            // Wait for the device to finish all operations before destroying it.
            self.device.device_wait_idle().expect("Failed to wait for device to finish all operations");
            self.device.destroy_device(None);
        }
    }
}