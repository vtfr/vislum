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
        let code = code
            .chunks(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<_>>();

        let create_info = vk::ShaderModuleCreateInfo::default().code(&code);

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
