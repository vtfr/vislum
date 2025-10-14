use std::sync::Arc;
use ash::vk;
use super::device::Device;

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
        
        let mut semaphore = vk::Semaphore::null();
        unsafe {
            (device.fns().vk_1_0().create_semaphore)(
                device.handle(),
                &create_info,
                std::ptr::null(),
                &mut semaphore,
            ).result().unwrap()
        }

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
            (self.device.fns().vk_1_0().destroy_semaphore)(
                self.device.handle(),
                self.semaphore,
                std::ptr::null(),
            );
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

        let create_info = vk::FenceCreateInfo::default()
            .flags(flags);

        let mut fence = vk::Fence::null();
        unsafe {
            (device.fns().vk_1_0().create_fence)(
                device.handle(),
                &create_info,
                std::ptr::null(),
                &mut fence,
            ).result().unwrap();
        }

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
        let fences = [self.fence];
        unsafe {
            (self.device.fns().vk_1_0().wait_for_fences)(
                self.device.handle(),
                1,
                fences.as_ptr(),
                vk::TRUE,
                timeout,
            ).result().unwrap();
        }
    }

    /// Reset the fence to unsignaled state
    pub fn reset(&self) -> Result<(), vk::Result> {
        let fences = [self.fence];
        unsafe {
            (self.device.fns().vk_1_0().reset_fences)(
                self.device.handle(),
                1,
                fences.as_ptr(),
            ).result()
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            (self.device.fns().vk_1_0().destroy_fence)(
                self.device.handle(),
                self.fence,
                std::ptr::null(),
            );
        }
    }
}
