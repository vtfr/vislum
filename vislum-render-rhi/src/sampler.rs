use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, VkHandle, device::Device};

pub struct SamplerCreateInfo {
    pub mag_filter: vk::Filter,
    pub min_filter: vk::Filter,
    pub address_mode_u: vk::SamplerAddressMode,
    pub address_mode_v: vk::SamplerAddressMode,
    pub address_mode_w: vk::SamplerAddressMode,
}

pub struct Sampler {
    device: Arc<Device>,
    sampler: DebugWrapper<vk::Sampler>,
}

impl Sampler {
    pub fn new(device: Arc<Device>, create_info: SamplerCreateInfo) -> Arc<Self> {
        let vk_create_info = vk::SamplerCreateInfo::default()
            .mag_filter(create_info.mag_filter)
            .min_filter(create_info.min_filter)
            .address_mode_u(create_info.address_mode_u)
            .address_mode_v(create_info.address_mode_v)
            .address_mode_w(create_info.address_mode_w);

        let sampler = unsafe {
            device.ash_handle().create_sampler(&vk_create_info, None).unwrap()
        };

        Arc::new(Self {
            device,
            sampler: DebugWrapper(sampler),
        })
    }
}

impl VkHandle for Sampler {
    type Handle = vk::Sampler;

    fn vk_handle(&self) -> Self::Handle {
        self.sampler.0
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_sampler(self.sampler.0, None);
        }
    }
}

