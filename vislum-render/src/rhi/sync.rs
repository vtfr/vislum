use std::sync::Arc;

use ash::vk;

use crate::rhi::Device;

#[derive(Default, Debug, Clone, Copy)]
pub struct FenceDescription {
    /// Whether the fence should be created in a signalled state.
    pub signalled: bool,
}

pub struct Fence {
    /// The device that this fence belongs to.
    device: Arc<Device>,

    /// The description of the fence.
    description: FenceDescription,
    
    /// The raw Vulkan fence handle.
    fence: vk::Fence,
}

static_assertions::assert_impl_all!(Fence: Send, Sync);

impl Fence {
    /// Creates a new fence.
    pub fn new(device: Arc<Device>, description: FenceDescription) -> Self {
        let mut flags = vk::FenceCreateFlags::empty();
        if description.signalled {
            flags |= vk::FenceCreateFlags::SIGNALED;
        }

        let create_info = vk::FenceCreateInfo::default()
            .flags(flags);

        let fence = unsafe { 
            device.ash_device()
                .create_fence(&create_info, None)
                .unwrap()
        };

        Self { device, description, fence }
    }

    /// Waits for the fence to be signaled.
    pub fn wait(&self) {
        // If the fence was created in a signalled state, we don't need to wait for it.
        if self.description.signalled {
            return
        }
        
        let ash_device = self.device.ash_device();
        unsafe { 
            ash_device.wait_for_fences(&[self.fence], true, u64::MAX).unwrap();
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        self.wait();

        unsafe { 
            // Destroy the fence.
            self.device.ash_device().destroy_fence(self.fence, None); 
        }
    }
}

impl std::fmt::Debug for Fence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fence")
            .field("description", &self.description)
            .field("fence", &self.fence)
            .finish()
    }
}

