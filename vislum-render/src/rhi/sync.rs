use super::device::Device;
use ash::vk;
use std::sync::Arc;

/// A synchronization semaphore
#[derive(Debug)]
pub struct Semaphore {
    device: Arc<Device>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(device: Arc<Device>) -> Self {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            device
                .vk()
                .create_semaphore(&create_info, None)
                .unwrap()
        };

        Self { device, semaphore }
    }

    #[inline]
    pub fn handle(&self) -> vk::Semaphore {
        self.semaphore
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_semaphore(self.semaphore, None);
        }
    }
}

/// A synchronization fence
#[derive(Debug)]
pub struct Fence {
    device: Arc<Device>,
    fence: vk::Fence,
}

#[derive(Debug)]
pub struct FenceDescription {
    pub signaled: bool,
}

impl Fence {
    /// Create a new fence
    pub fn new(device: Arc<Device>, description: FenceDescription) -> Self {
        let mut flags = vk::FenceCreateFlags::empty();
        if description.signaled {
            flags |= vk::FenceCreateFlags::SIGNALED;
        }

        let create_info = vk::FenceCreateInfo::default().flags(flags);

        let fence = unsafe { device.vk().create_fence(&create_info, None).unwrap() };

        Self { device, fence }
    }

    #[inline]
    pub fn handle(&self) -> vk::Fence {
        self.fence
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Wait for the fence to be signaled
    pub fn wait(&self, timeout: u64) {
        unsafe {
            self.device
                .vk()
                .wait_for_fences(&[self.fence], true, timeout)
                .unwrap();
        }
    }

    /// Reset the fence to unsignaled state
    pub fn reset(&self) -> Result<(), vk::Result> {
        unsafe {
            self.device.vk().reset_fences(&[self.fence]).unwrap();
        }

        Ok(())
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_fence(self.fence, None);
        }
    }
}
