use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, device::device::Device, vk_enum};

vk_enum! {
    pub enum ShaderStage: vk::ShaderStageFlags {
        Vertex = VERTEX,
        Fragment = FRAGMENT,
        Compute = COMPUTE,
    }
}

pub struct ShaderModule {
    device: Arc<Device>,
    module: vk::ShaderModule,
}

impl VkHandle for ShaderModule {
    type Handle = vk::ShaderModule;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.module
    }
}

impl ShaderModule {
    pub fn new(device: Arc<Device>, code: &[u8]) -> Arc<Self> {
        let code_aligned = unsafe {
            std::slice::from_raw_parts(
                code.as_ptr() as *const u32,
                code.len() / std::mem::size_of::<u32>(),
            )
        };

        let create_info = vk::ShaderModuleCreateInfo::default().code(code_aligned);

        let module = unsafe {
            device
                .ash_handle()
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        };

        Arc::new(Self { device, module })
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .destroy_shader_module(self.module, None);
        }
    }
}

