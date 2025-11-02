use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, device::Device};

pub struct Fence {
    device: Arc<Device>,
    fence: DebugWrapper<vk::Fence>,
}

impl Fence {
    /// Creates a new fence in the signaled state.
    pub fn signaled(device: Arc<Device>) -> Arc<Self> {
        let create_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let fence = unsafe {
            device.ash_handle().create_fence(&create_info, None)
        }.unwrap();

        Arc::new(Self {
            device,
            fence: DebugWrapper(fence),
        })
    }

    /// Creates a new fence in the unsignaled state.
    pub fn unsignaled(device: Arc<Device>) -> Arc<Self> {
        let create_info = vk::FenceCreateInfo::default();

        let fence = unsafe {
            device.ash_handle().create_fence(&create_info, None)
        }.unwrap();

        Arc::new(Self {
            device,
            fence: DebugWrapper(fence),
        })
    }

    /// Waits for the fence to be signaled.
    /// Returns true if the fence was signaled, false if the wait timed out.
    pub fn wait(&self, timeout: u64) -> bool {
        unsafe {
            match self.device.ash_handle().wait_for_fences(
                &[self.fence.0],
                true,
                timeout,
            ) {
                Ok(()) => true,
                Err(vk::Result::TIMEOUT) => false,
                Err(e) => panic!("wait_for_fences failed: {:?}", e),
            }
        }
    }

    /// Resets the fence to unsignaled state.
    pub fn reset(&self) {
        unsafe {
            self.device.ash_handle().reset_fences(&[self.fence.0]).unwrap();
        }
    }

    /// Gets the status of the fence.
    pub fn status(&self) -> bool {
        unsafe {
            self.device.ash_handle().get_fence_status(self.fence.0).unwrap()
        }
    }
}

impl crate::VkHandle for Fence {
    type Handle = vk::Fence;

    fn vk_handle(&self) -> Self::Handle {
        self.fence.0
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_fence(self.fence.0, None);
        }
    }
}

pub struct Semaphore {
    device: Arc<Device>,
    semaphore: DebugWrapper<vk::Semaphore>,
}

impl Semaphore {
    /// Creates a new semaphore.
    pub fn new(device: Arc<Device>) -> Arc<Self> {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            device.ash_handle().create_semaphore(&create_info, None)
        }.unwrap();

        Arc::new(Self {
            device,
            semaphore: DebugWrapper(semaphore),
        })
    }
}

impl crate::VkHandle for Semaphore {
    type Handle = vk::Semaphore;

    fn vk_handle(&self) -> Self::Handle {
        self.semaphore.0
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_semaphore(self.semaphore.0, None);
        }
    }
}

