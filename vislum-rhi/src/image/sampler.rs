use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, device::device::Device, vk_enum};

vk_enum! {
    pub enum Filter: vk::Filter {
        Nearest = NEAREST,
        Linear = LINEAR,
    }
}

vk_enum! {
    pub enum SamplerMipmapMode: vk::SamplerMipmapMode {
        Nearest = NEAREST,
        Linear = LINEAR,
    }
}

vk_enum! {
    pub enum SamplerAddressMode: vk::SamplerAddressMode {
        Repeat = REPEAT,
        MirroredRepeat = MIRRORED_REPEAT,
        ClampToEdge = CLAMP_TO_EDGE,
        ClampToBorder = CLAMP_TO_BORDER,
        MirrorClampToEdge = MIRROR_CLAMP_TO_EDGE,
    }
}

vk_enum! {
    pub enum BorderColor: vk::BorderColor {
        FloatTransparentBlack = FLOAT_TRANSPARENT_BLACK,
        IntTransparentBlack = INT_TRANSPARENT_BLACK,
        FloatOpaqueBlack = FLOAT_OPAQUE_BLACK,
        IntOpaqueBlack = INT_OPAQUE_BLACK,
        FloatOpaqueWhite = FLOAT_OPAQUE_WHITE,
        IntOpaqueWhite = INT_OPAQUE_WHITE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SamplerCreateInfo {
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: bool,
    pub max_anisotropy: f32,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
    pub unnormalized_coordinates: bool,
}

impl Default for SamplerCreateInfo {
    fn default() -> Self {
        Self {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            mipmap_mode: SamplerMipmapMode::Linear,
            address_mode_u: SamplerAddressMode::Repeat,
            address_mode_v: SamplerAddressMode::Repeat,
            address_mode_w: SamplerAddressMode::Repeat,
            mip_lod_bias: 0.0,
            anisotropy_enable: false,
            max_anisotropy: 1.0,
            min_lod: 0.0,
            max_lod: vk::LOD_CLAMP_NONE,
            border_color: BorderColor::FloatOpaqueBlack,
            unnormalized_coordinates: false,
        }
    }
}

pub struct Sampler {
    device: Arc<Device>,
    sampler: vk::Sampler,
}

impl VkHandle for Sampler {
    type Handle = vk::Sampler;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.sampler
    }
}

impl Sampler {
    pub fn new(device: Arc<Device>, create_info: SamplerCreateInfo) -> Arc<Self> {
        let vk_create_info = vk::SamplerCreateInfo::default()
            .mag_filter(create_info.mag_filter.to_vk())
            .min_filter(create_info.min_filter.to_vk())
            .mipmap_mode(create_info.mipmap_mode.to_vk())
            .address_mode_u(create_info.address_mode_u.to_vk())
            .address_mode_v(create_info.address_mode_v.to_vk())
            .address_mode_w(create_info.address_mode_w.to_vk())
            .mip_lod_bias(create_info.mip_lod_bias)
            .anisotropy_enable(create_info.anisotropy_enable)
            .max_anisotropy(create_info.max_anisotropy)
            .min_lod(create_info.min_lod)
            .max_lod(create_info.max_lod)
            .border_color(create_info.border_color.to_vk())
            .unnormalized_coordinates(create_info.unnormalized_coordinates);

        let sampler = unsafe {
            device
                .ash_handle()
                .create_sampler(&vk_create_info, None)
                .expect("Failed to create sampler")
        };

        Arc::new(Self { device, sampler })
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_sampler(self.sampler, None);
        }
    }
}
