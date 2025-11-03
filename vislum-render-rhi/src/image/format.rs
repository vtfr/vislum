use crate::vk_enum;

vk_enum! {
    #[derive(Default)]
    pub enum ImageFormat: ash::vk::Format {
        #[default]
        Rgba8Unorm => R8G8B8A8_UNORM,
        Rgba8Srgb => R8G8B8A8_SRGB,
        Rgb8Unorm => R8G8B8_UNORM,
        Rgb8Srgb => R8G8B8_SRGB,
        Bgra8Unorm => B8G8R8A8_UNORM,
    }
}

