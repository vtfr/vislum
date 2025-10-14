use super::device::Device;
use ash::vk;
use std::{
    ops::{Range, RangeBounds, RangeFull, RangeInclusive},
    sync::Arc,
};

/// A command pool for allocating command buffers
#[derive(Debug)]
pub struct CommandPool {
    device: Arc<Device>,
    pool: vk::CommandPool,
    queue_family_index: u32,
}

impl CommandPool {
    /// Create a new command pool
    pub fn new(
        device: Arc<Device>,
        queue_family_index: u32,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<Arc<Self>, vk::Result> {
        let create_info = vk::CommandPoolCreateInfo::default()
            .flags(flags)
            .queue_family_index(queue_family_index);

        let pool = unsafe {
            device
                .vk()
                .create_command_pool(&create_info, None)
                .unwrap()
        };

        Ok(Arc::new(Self {
            device,
            pool,
            queue_family_index,
        }))
    }

    #[inline]
    pub fn handle(&self) -> vk::CommandPool {
        self.pool
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    #[inline]
    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }

    /// Allocate command buffers from this pool
    pub fn allocate_command_buffers(
        self: &Arc<Self>,
        level: vk::CommandBufferLevel,
        count: u32,
    ) -> Result<Vec<CommandBuffer>, vk::Result> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool)
            .level(level)
            .command_buffer_count(count);

        let command_buffers = unsafe {
            self.device.vk()
                .allocate_command_buffers(&alloc_info)?
        };

        Ok(command_buffers
            .into_iter()
            .map(|buffer| CommandBuffer {
                pool: Arc::clone(self),
                buffer,
            })
            .collect())
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_command_pool(self.pool, None);
        }
    }
}

/// A command buffer for recording GPU commands
#[derive(Debug)]
pub struct CommandBuffer {
    pool: Arc<CommandPool>,
    buffer: vk::CommandBuffer,
}

impl CommandBuffer {
    #[inline]
    pub fn vk(&self) -> vk::CommandBuffer {
        self.buffer
    }

    #[inline]
    pub fn pool(&self) -> &Arc<CommandPool> {
        &self.pool
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        self.pool.device()
    }

    /// Reset the command buffer
    pub fn reset(&self, flags: vk::CommandBufferResetFlags) {
        unsafe {
            self.pool
                .device()
                .vk()
                .reset_command_buffer(self.buffer, flags)
                .unwrap()
        }
    }

    /// Begin recording commands
    pub fn begin(&self, flags: vk::CommandBufferUsageFlags) {
        let begin_info = vk::CommandBufferBeginInfo::default().flags(flags);

        unsafe {
            self.pool
                .device()
                .vk()
                .begin_command_buffer(self.buffer, &begin_info)
                .unwrap()
        }
    }

    /// End recording commands
    pub fn end(&self) {
        unsafe {
            self.pool
                .device()
                .vk()
                .end_command_buffer(self.buffer)
                .unwrap()
        }
    }

    /// Bind a graphics pipeline
    pub fn bind_pipeline(&self, bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe {
            self.pool
                .device()
                .vk()
                .cmd_bind_pipeline(self.buffer, bind_point, pipeline);
        }
    }

    /// Draw primitives
    pub fn draw(&self, vertices: Range<u32>, instances: Range<u32>) {
        let vertex_count = vertices.len() as u32;
        let instance_count = instances.len() as u32;
        let first_vertex = vertices.start;
        let first_instance = instances.start;

        unsafe {
            self.pool.device().vk().cmd_draw(
                self.buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    /// Begin dynamic rendering (Vulkan 1.3 / VK_KHR_dynamic_rendering)
    pub fn begin_rendering(&self, rendering_info: &vk::RenderingInfo) {
        let device = self.pool.device();
        unsafe {
            match device.khr_dynamic_rendering_device() {
                Some(device) => device.cmd_begin_rendering(self.buffer, rendering_info),
                None => device.vk().cmd_begin_rendering(self.buffer, rendering_info),
            }
        }
    }

    /// End dynamic rendering (Vulkan 1.3 / VK_KHR_dynamic_rendering)
    pub fn end_rendering(&self) {
        let device = self.pool.device();
        unsafe {
            match device.khr_dynamic_rendering_device() {
                Some(device) => device.cmd_end_rendering(self.buffer),
                None => device.vk().cmd_end_rendering(self.buffer),
            }
        }
    }

    // Extended Dynamic State 1 (Vulkan 1.3 core / VK_EXT_extended_dynamic_state)

    /// Set cull mode dynamically
    pub fn set_cull_mode(&self, cull_mode: vk::CullModeFlags) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_cull_mode(self.buffer, cull_mode),
                None => device.vk().cmd_set_cull_mode(self.buffer, cull_mode),
            }
        }
    }

    /// Set front face winding order dynamically
    pub fn set_front_face(&self, front_face: vk::FrontFace) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_front_face(self.buffer, front_face),
                None => device.vk().cmd_set_front_face(self.buffer, front_face),
            }
        }
    }

    /// Set primitive topology dynamically
    pub fn set_primitive_topology(&self, topology: vk::PrimitiveTopology) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_primitive_topology(self.buffer, topology),
                None => device.vk().cmd_set_primitive_topology(self.buffer, topology),
            }
        }
    }

    /// Set viewport with count dynamically
    pub fn set_viewport_with_count(&self, viewports: &[vk::Viewport]) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_viewport_with_count(self.buffer, viewports),
                None => device.vk().cmd_set_viewport_with_count(self.buffer, viewports),
            }
        }
    }

    /// Set scissor with count dynamically
    pub fn set_scissor_with_count(&self, scissors: &[vk::Rect2D]) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_scissor_with_count(self.buffer, scissors),
                None => device.vk().cmd_set_scissor_with_count(self.buffer, scissors),
            }
        }
    }

    /// Enable/disable depth test dynamically
    pub fn set_depth_test_enable(&self, enable: bool) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_depth_test_enable(self.buffer, enable.into()),
                None => device.vk().cmd_set_depth_test_enable(self.buffer, enable.into()),
            }
        }
    }

    /// Enable/disable depth write dynamically
    pub fn set_depth_write_enable(&self, enable: bool) {
        let device = self.pool
                .device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_depth_write_enable(self.buffer, enable.into()),
                None => device.vk().cmd_set_depth_write_enable(self.buffer, enable.into()),
            }
        }
    }

    /// Set depth compare operation dynamically
    pub fn set_depth_compare_op(&self, compare_op: vk::CompareOp) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_depth_compare_op(self.buffer, compare_op),
                None => device.vk().cmd_set_depth_compare_op(self.buffer, compare_op),
            }
        }
    }

    /// Enable/disable depth bounds test dynamically
    pub fn set_depth_bounds_test_enable(&self, enable: bool) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_depth_bounds_test_enable(self.buffer, enable.into()),
                None => device.vk().cmd_set_depth_bounds_test_enable(self.buffer, enable.into()),
            }
        }
    }

    /// Enable/disable stencil test dynamically
    pub fn set_stencil_test_enable(&self, enable: bool) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_stencil_test_enable(self.buffer, enable.into()),
                None => device.vk().cmd_set_stencil_test_enable(self.buffer, enable.into()),
            }
        }
    }

    /// Set stencil operations dynamically
    pub fn set_stencil_op(
        &self,
        face_mask: vk::StencilFaceFlags,
        fail_op: vk::StencilOp,
        pass_op: vk::StencilOp,
        depth_fail_op: vk::StencilOp,
        compare_op: vk::CompareOp,
    ) {
        let device = self.pool.device();
        unsafe {
            match device.ext_extended_dynamic_state_device() {
                Some(device) => device.cmd_set_stencil_op(
                    self.buffer,
                    face_mask,
                    fail_op,
                    pass_op,
                    depth_fail_op,
                    compare_op,
                ),
                None => device.vk().cmd_set_stencil_op(
                    self.buffer,
                    face_mask,
                    fail_op,
                    pass_op,
                    depth_fail_op,
                    compare_op,
                ),
            }
        }
    }
}
