use ash::vk;

use crate::rhi::{PhysicalDevice, VulkanHandle};

pub struct Device {
    device: ash::Device,
    fns: ash::khr::swapchain::Device,
}

impl VulkanHandle for Device {
    type Handle = vk::Device;

    fn vk_handle(&self) -> Self::Handle {
        self.device.handle()
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