use std::{ops::{Range, RangeBounds, RangeFull, RangeInclusive}, sync::Arc};
use ash::vk;
use super::device::Device;

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

        let mut pool = vk::CommandPool::null();
        unsafe {
            (device.fns().vk_1_0().create_command_pool)(
                device.handle(),
                &create_info,
                std::ptr::null(),
                &mut pool,
            ).result()?;
        }

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

        let mut command_buffers = vec![vk::CommandBuffer::null(); count as usize];
        unsafe {
            (self.device.fns().vk_1_0().allocate_command_buffers)(
                self.device.handle(),
                &alloc_info,
                command_buffers.as_mut_ptr(),
            ).result()?;
        }

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
            (self.device.fns().vk_1_0().destroy_command_pool)(
                self.device.handle(),
                self.pool,
                std::ptr::null(),
            );
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
    pub fn handle(&self) -> vk::CommandBuffer {
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
    pub fn reset(&self, flags: vk::CommandBufferResetFlags) -> Result<(), vk::Result> {
        unsafe {
            (self.device().fns().vk_1_0().reset_command_buffer)(
                self.buffer,
                flags,
            ).result()
        }
    }

    /// Begin recording commands
    pub fn begin(&self, flags: vk::CommandBufferUsageFlags) -> Result<(), vk::Result> {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(flags);

        unsafe {
            (self.device().fns().vk_1_0().begin_command_buffer)(
                self.buffer,
                &begin_info,
            ).result()
        }
    }

    /// End recording commands
    pub fn end(&self) -> Result<(), vk::Result> {
        unsafe {
            (self.device().fns().vk_1_0().end_command_buffer)(self.buffer).result()
        }
    }

    /// Bind a graphics pipeline
    pub fn bind_pipeline(&self, bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe {
            (self.device().fns().vk_1_0().cmd_bind_pipeline)(
                self.buffer,
                bind_point,
                pipeline,
            );
        }
    }

    /// Draw primitives
    pub fn draw(
        &self,
        vertices: Range<u32>,
        instances: Range<u32>,
    ) {
        let vertex_count = vertices.len() as u32;
        let instance_count = instances.len() as u32;
        let first_vertex = vertices.start;
        let first_instance = instances.start;

        unsafe {
            (self.device().fns().vk_1_0().cmd_draw)(
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
        let device = self.device();
        
        let cmd_begin_rendering = match device.fns().khr_dynamic_rendering() {
            Some(fns) => fns.cmd_begin_rendering_khr,
            None => device.fns().vk_1_3().cmd_begin_rendering,
        };

        unsafe {
            (cmd_begin_rendering)(self.buffer, rendering_info);
        }
    }

    /// End dynamic rendering (Vulkan 1.3 / VK_KHR_dynamic_rendering)
    pub fn end_rendering(&self) {
        let device = self.device();
        
        let cmd_end_rendering = match device.fns().khr_dynamic_rendering() {
            Some(fns) => fns.cmd_end_rendering_khr,
            None => device.fns().vk_1_3().cmd_end_rendering,
        };

        unsafe {
            (cmd_end_rendering)(self.buffer);
        }
    }

    // Extended Dynamic State 1 (Vulkan 1.3 core / VK_EXT_extended_dynamic_state)

    /// Set cull mode dynamically
    pub fn set_cull_mode(&self, cull_mode: vk::CullModeFlags) {
        let device = self.device();
        
        let cmd_set_cull_mode = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_cull_mode_ext,
            None => device.fns().vk_1_3().cmd_set_cull_mode,
        };
        
        unsafe {
            (cmd_set_cull_mode)(self.buffer, cull_mode);
        }
    }

    /// Set front face winding order dynamically
    pub fn set_front_face(&self, front_face: vk::FrontFace) {
        let device = self.device();
        
        let cmd_set_front_face = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_front_face_ext,
            None => device.fns().vk_1_3().cmd_set_front_face,
        };
        
        unsafe {
            (cmd_set_front_face)(self.buffer, front_face);
        }
    }

    /// Set primitive topology dynamically
    pub fn set_primitive_topology(&self, topology: vk::PrimitiveTopology) {
        let device = self.device();
        
        let cmd_set_primitive_topology = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_primitive_topology_ext,
            None => device.fns().vk_1_3().cmd_set_primitive_topology,
        };
        
        unsafe {
            (cmd_set_primitive_topology)(self.buffer, topology);
        }
    }

    /// Set viewport with count dynamically
    pub fn set_viewport_with_count(&self, viewports: &[vk::Viewport]) {
        let device = self.device();
        
        let cmd_set_viewport_with_count = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_viewport_with_count_ext,
            None => device.fns().vk_1_3().cmd_set_viewport_with_count,
        };
        
        unsafe {
            (cmd_set_viewport_with_count)(
                self.buffer,
                viewports.len() as u32,
                viewports.as_ptr(),
            );
        }
    }

    /// Set scissor with count dynamically
    pub fn set_scissor_with_count(&self, scissors: &[vk::Rect2D]) {
        let device = self.device();
        
        let cmd_set_scissor_with_count = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_scissor_with_count_ext,
            None => device.fns().vk_1_3().cmd_set_scissor_with_count,
        };
        
        unsafe {
            (cmd_set_scissor_with_count)(
                self.buffer,
                scissors.len() as u32,
                scissors.as_ptr(),
            );
        }
    }

    /// Enable/disable depth test dynamically
    pub fn set_depth_test_enable(&self, enable: bool) {
        let device = self.device();
        
        let cmd_set_depth_test_enable = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_depth_test_enable_ext,
            None => device.fns().vk_1_3().cmd_set_depth_test_enable,
        };
        
        unsafe {
            (cmd_set_depth_test_enable)(self.buffer, enable.into());
        }
    }

    /// Enable/disable depth write dynamically
    pub fn set_depth_write_enable(&self, enable: bool) {
        let device = self.device();
        
        let cmd_set_depth_write_enable = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_depth_write_enable_ext,
            None => device.fns().vk_1_3().cmd_set_depth_write_enable,
        };
        
        unsafe {
            (cmd_set_depth_write_enable)(self.buffer, enable.into());
        }
    }

    /// Set depth compare operation dynamically
    pub fn set_depth_compare_op(&self, compare_op: vk::CompareOp) {
        let device = self.device();
        
        let cmd_set_depth_compare_op = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_depth_compare_op_ext,
            None => device.fns().vk_1_3().cmd_set_depth_compare_op,
        };
        
        unsafe {
            (cmd_set_depth_compare_op)(self.buffer, compare_op);
        }
    }

    /// Enable/disable depth bounds test dynamically
    pub fn set_depth_bounds_test_enable(&self, enable: bool) {
        let device = self.device();
        
        let cmd_set_depth_bounds_test_enable = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_depth_bounds_test_enable_ext,
            None => device.fns().vk_1_3().cmd_set_depth_bounds_test_enable,
        };
        
        unsafe {
            (cmd_set_depth_bounds_test_enable)(self.buffer, enable.into());
        }
    }

    /// Enable/disable stencil test dynamically
    pub fn set_stencil_test_enable(&self, enable: bool) {
        let device = self.device();
        
        let cmd_set_stencil_test_enable = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_stencil_test_enable_ext,
            None => device.fns().vk_1_3().cmd_set_stencil_test_enable,
        };
        
        unsafe {
            (cmd_set_stencil_test_enable)(self.buffer, enable.into());
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
        let device = self.device();
        
        let cmd_set_stencil_op = match device.fns().ext_extended_dynamic_state() {
            Some(fns) => fns.cmd_set_stencil_op_ext,
            None => device.fns().vk_1_3().cmd_set_stencil_op,
        };
        
        unsafe {
            (cmd_set_stencil_op)(
                self.buffer,
                face_mask,
                fail_op,
                pass_op,
                depth_fail_op,
                compare_op,
            );
        }
    }
}

