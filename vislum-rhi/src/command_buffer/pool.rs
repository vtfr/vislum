use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, device::device::Device};

use super::buffer::CommandBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandPoolCreateInfo {
    pub queue_family_index: u32,
    pub transient: bool,
    pub reset_command_buffer: bool,
}

pub struct CommandPool {
    device: Arc<Device>,
    pool: vk::CommandPool,
    queue_family_index: u32,
}

impl VkHandle for CommandPool {
    type Handle = vk::CommandPool;

    fn vk_handle(&self) -> Self::Handle {
        self.pool
    }
}

impl CommandPool {
    pub fn new(device: Arc<Device>, create_info: CommandPoolCreateInfo) -> Arc<Self> {
        let mut flags = vk::CommandPoolCreateFlags::empty();
        if create_info.transient {
            flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }
        if create_info.reset_command_buffer {
            flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }

        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(create_info.queue_family_index)
            .flags(flags);

        let pool = unsafe {
            device
                .ash_handle()
                .create_command_pool(&pool_create_info, None)
                .expect("Failed to create command pool")
        };

        Arc::new(Self {
            device,
            pool,
            queue_family_index: create_info.queue_family_index,
        })
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn allocate_command_buffers(self: &Arc<Self>, count: u32) -> Vec<CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let buffers = unsafe {
            self.device
                .ash_handle()
                .allocate_command_buffers(&alloc_info)
                .expect("Failed to allocate command buffers")
        };

        buffers
            .into_iter()
            .map(|buffer| CommandBuffer::new(self.clone(), buffer))
            .collect()
    }

    pub fn reset(&self, release_resources: bool) {
        let flags = if release_resources {
            vk::CommandPoolResetFlags::RELEASE_RESOURCES
        } else {
            vk::CommandPoolResetFlags::empty()
        };

        unsafe {
            self.device
                .ash_handle()
                .reset_command_pool(self.pool, flags)
                .expect("Failed to reset command pool");
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .destroy_command_pool(self.pool, None);
        }
    }
}
