use std::{ops::Range, sync::Arc};

use ash::vk::{self};
use smallvec::SmallVec;

use crate::rhi::{device::Device, image::ImageLayout};

pub struct CommandPool {
    device: Arc<Device>,
    inner: vk::CommandPool,
}

pub struct CommandBuffer {
    device: Arc<Device>,
    command_pool: Arc<CommandPool>,
    inner: vk::CommandBuffer,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AccessFlags: u8 {
        const TRANSFER_READ = 1 << 0;
        const TRANSFER_WRITE = 1 << 1;
        const HOST_READ = 1 << 2;
        const HOST_WRITE = 1 << 3;
    }
}

impl AccessFlags {
    pub fn to_vk(&self) -> vk::AccessFlags2 {
        let mut flags = vk::AccessFlags2::empty();
        if self.contains(AccessFlags::TRANSFER_READ) {
            flags |= vk::AccessFlags2::TRANSFER_READ;
        }
        if self.contains(AccessFlags::TRANSFER_WRITE) {
            flags |= vk::AccessFlags2::TRANSFER_WRITE;
        }
        if self.contains(AccessFlags::HOST_READ) {
            flags |= vk::AccessFlags2::HOST_READ;
        }
        if self.contains(AccessFlags::HOST_WRITE) {
            flags |= vk::AccessFlags2::HOST_WRITE;
        }
        flags
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageBarrier {
    pub image: (),
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_access_flags: AccessFlags,
    pub dst_access_flags: AccessFlags,
}

#[derive(Default)]
pub struct Barrier {
    pub image_memory_barriers: SmallVec<[ImageBarrier; 4]>,
}

impl Barrier {
    pub fn add_image_barriers(&mut self, barriers: impl Iterator<Item=ImageBarrier>) -> &mut Self {
        self.image_memory_barriers.extend(barriers);
        self
    }
}


impl CommandBuffer {
    pub fn begin_rendering(&self) {
        let begin_info = vk::RenderingInfo::default();

        self.branch_rendering(
            |device| {
                unsafe { device.cmd_begin_rendering(self.inner, &begin_info); }
            },
            |device| {
                unsafe { device.cmd_begin_rendering(self.inner, &begin_info) };
            },
        );
    }

    /// Add barriers to the command buffer.
    pub fn barrier(&self, barriers: Barrier) {
        let image_memory_barriers: SmallVec<[_; 4]> = barriers.image_memory_barriers
            .iter()
            .map(|barrier| 
                vk::ImageMemoryBarrier2::default()
                    .src_access_mask(barrier.src_access_flags.to_vk())
                    .dst_access_mask(barrier.dst_access_flags.to_vk())
                    .old_layout(barrier.old_layout.to_vk())
                    .new_layout(barrier.new_layout.to_vk())
                    .subresource_range(vk::ImageSubresourceRange::default())
            )
            .collect();

        let transition_info = vk::DependencyInfo::default()
            .image_memory_barriers(&*image_memory_barriers);

        unsafe {
            self.device.handle()
                .cmd_pipeline_barrier2(self.inner, &transition_info);
        }
    }

    pub fn draw(&self, vertices: Range<u32>, instances: Range<u32>) {
        let vertex_count = vertices.len() as u32;
        let instance_count = instances.len() as u32;
        unsafe {
            self.device.handle().cmd_draw(self.inner, vertex_count, instance_count, vertices.start, instances.start);
        }
    }

    pub fn end_rendering(&self) {
        self.branch_rendering(
            |device| {
                unsafe { device.cmd_end_rendering(self.inner); }
            },
            |device| {
                unsafe { device.cmd_end_rendering(self.inner); }
            },
        );
    }

    #[inline]
    fn branch_rendering(&self,
        vk13: impl FnOnce(&ash::Device),
        ext: impl FnOnce(&ash::khr::dynamic_rendering::Device),
    ) {
        match self.device.khr_dynamic_rendering_device() {
            Some(device) => ext(device),
            None => {
                cold();
                vk13(self.device.handle());
            },
        };
    }
    
    pub fn set_vertex_buffer(&self, buffer: vk::Buffer, offset: u64) {
        unsafe {
            self.device.handle()
                .cmd_bind_vertex_buffers(self.inner, 0, &[buffer], &[offset]);
        }
    }
}

#[inline(always)]
#[cold]
fn cold() {}