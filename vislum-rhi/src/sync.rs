use std::sync::Arc;

use ash::vk;

use crate::{device::device::Device, AshHandle, VkHandle};

/// A Vulkan semaphore.
/// 
/// # Safety
/// The semaphore must be kept alive for the duration of all operations that use it.
pub struct Semaphore {
    device: Arc<Device>,
    semaphore: vk::Semaphore,
}

impl VkHandle for Semaphore {
    type Handle = vk::Semaphore;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.semaphore
    }
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Arc<Self> {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            device
                .ash_handle()
                .create_semaphore(&create_info, None)
                .expect("Failed to create semaphore")
        };

        Arc::new(Self { device, semaphore })
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .destroy_semaphore(self.semaphore, None);
        }
    }
}

pub struct Fence {
    device: Arc<Device>,
    fence: vk::Fence,
}

impl VkHandle for Fence {
    type Handle = vk::Fence;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.fence
    }
}

impl Fence {
    pub fn new(device: Arc<Device>, signaled: bool) -> Arc<Self> {
        let flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        };

        let create_info = vk::FenceCreateInfo::default().flags(flags);

        let fence = unsafe {
            device
                .ash_handle()
                .create_fence(&create_info, None)
                .expect("Failed to create fence")
        };

        Arc::new(Self { device, fence })
    }

    pub fn wait(&self, timeout: u64) {
        unsafe {
            self.device
                .ash_handle()
                .wait_for_fences(&[self.fence], true, timeout)
                .expect("Failed to wait for fence");
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device
                .ash_handle()
                .reset_fences(&[self.fence])
                .expect("Failed to reset fence");
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_fence(self.fence, None);
        }
    }
}

