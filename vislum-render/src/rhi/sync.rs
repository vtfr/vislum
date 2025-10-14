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
    pub fn new(device: Arc<Device>) -> Arc<Self> {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            device
                .handle()
                .create_semaphore(&create_info, None)
                .unwrap()
        };

        Arc::new(Self { device, semaphore })
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
            self.device.handle().destroy_semaphore(self.semaphore, None);
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
    pub fn new(device: Arc<Device>, description: FenceDescription) -> Arc<Self> {
        let mut flags = vk::FenceCreateFlags::empty();
        if description.signaled {
            flags |= vk::FenceCreateFlags::SIGNALED;
        }

        let create_info = vk::FenceCreateInfo::default().flags(flags);

        let fence = unsafe { device.handle().create_fence(&create_info, None).unwrap() };

        Arc::new(Self { device, fence })
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
                .handle()
                .wait_for_fences(&[self.fence], true, timeout)
                .unwrap();
        }
    }

    /// Reset the fence to unsignaled state
    pub fn reset(&self) -> Result<(), vk::Result> {
        unsafe {
            self.device.handle().reset_fences(&[self.fence]).unwrap();
        }

        Ok(())
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_fence(self.fence, None);
        }
    }
}
