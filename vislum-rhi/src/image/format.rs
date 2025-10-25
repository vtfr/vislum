use ash::vk;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    // 8-bit formats
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,
    R8Srgb,

    // 16-bit formats
    R16Unorm,
    R16Snorm,
    R16Uint,
    R16Sint,
    R16Sfloat,

    // 32-bit formats
    R32Uint,
    R32Sint,
    R32Sfloat,

    // RG formats
    R8G8Unorm,
    R8G8Snorm,
    R8G8Uint,
    R8G8Sint,
    R8G8Srgb,

    R16G16Unorm,
    R16G16Snorm,
    R16G16Uint,
    R16G16Sint,
    R16G16Sfloat,

    R32G32Uint,
    R32G32Sint,
    R32G32Sfloat,

    // RGB formats
    R32G32B32Uint,
    R32G32B32Sint,
    R32G32B32Sfloat,

    // RGBA formats
    R8G8B8A8Unorm,
    R8G8B8A8Snorm,
    R8G8B8A8Uint,
    R8G8B8A8Sint,
    R8G8B8A8Srgb,

    B8G8R8A8Unorm,
    B8G8R8A8Snorm,
    B8G8R8A8Uint,
    B8G8R8A8Sint,
    B8G8R8A8Srgb,

    R16G16B16A16Unorm,
    R16G16B16A16Snorm,
    R16G16B16A16Uint,
    R16G16B16A16Sint,
    R16G16B16A16Sfloat,

    R32G32B32A32Uint,
    R32G32B32A32Sint,
    R32G32B32A32Sfloat,

    // Depth/Stencil formats
    D16Unorm,
    D32Sfloat,
    D16UnormS8Uint,
    D24UnormS8Uint,
    D32SfloatS8Uint,

    // Compressed formats
    BC1RgbUnorm,
    BC1RgbSrgb,
    BC1RgbaUnorm,
    BC1RgbaSrgb,
    BC2Unorm,
    BC2Srgb,
    BC3Unorm,
    BC3Srgb,
    BC4Unorm,
    BC4Snorm,
    BC5Unorm,
    BC5Snorm,
    BC6hUfloat,
    BC6hSfloat,
    BC7Unorm,
    BC7Srgb,

    // 10-bit formats
    A2R10G10B10UnormPack32,
    A2R10G10B10UintPack32,
}

impl ImageFormat {
    pub fn to_vk(self) -> vk::Format {
        match self {
            // 8-bit formats
            ImageFormat::R8Unorm => vk::Format::R8_UNORM,
            ImageFormat::R8Snorm => vk::Format::R8_SNORM,
            ImageFormat::R8Uint => vk::Format::R8_UINT,
            ImageFormat::R8Sint => vk::Format::R8_SINT,
            ImageFormat::R8Srgb => vk::Format::R8_SRGB,

            // 16-bit formats
            ImageFormat::R16Unorm => vk::Format::R16_UNORM,
            ImageFormat::R16Snorm => vk::Format::R16_SNORM,
            ImageFormat::R16Uint => vk::Format::R16_UINT,
            ImageFormat::R16Sint => vk::Format::R16_SINT,
            ImageFormat::R16Sfloat => vk::Format::R16_SFLOAT,

            // 32-bit formats
            ImageFormat::R32Uint => vk::Format::R32_UINT,
            ImageFormat::R32Sint => vk::Format::R32_SINT,
            ImageFormat::R32Sfloat => vk::Format::R32_SFLOAT,

            // RG formats
            ImageFormat::R8G8Unorm => vk::Format::R8G8_UNORM,
            ImageFormat::R8G8Snorm => vk::Format::R8G8_SNORM,
            ImageFormat::R8G8Uint => vk::Format::R8G8_UINT,
            ImageFormat::R8G8Sint => vk::Format::R8G8_SINT,
            ImageFormat::R8G8Srgb => vk::Format::R8G8_SRGB,

            ImageFormat::R16G16Unorm => vk::Format::R16G16_UNORM,
            ImageFormat::R16G16Snorm => vk::Format::R16G16_SNORM,
            ImageFormat::R16G16Uint => vk::Format::R16G16_UINT,
            ImageFormat::R16G16Sint => vk::Format::R16G16_SINT,
            ImageFormat::R16G16Sfloat => vk::Format::R16G16_SFLOAT,

            ImageFormat::R32G32Uint => vk::Format::R32G32_UINT,
            ImageFormat::R32G32Sint => vk::Format::R32G32_SINT,
            ImageFormat::R32G32Sfloat => vk::Format::R32G32_SFLOAT,

            // RGB formats
            ImageFormat::R32G32B32Uint => vk::Format::R32G32B32_UINT,
            ImageFormat::R32G32B32Sint => vk::Format::R32G32B32_SINT,
            ImageFormat::R32G32B32Sfloat => vk::Format::R32G32B32_SFLOAT,

            // RGBA formats
            ImageFormat::R8G8B8A8Unorm => vk::Format::R8G8B8A8_UNORM,
            ImageFormat::R8G8B8A8Snorm => vk::Format::R8G8B8A8_SNORM,
            ImageFormat::R8G8B8A8Uint => vk::Format::R8G8B8A8_UINT,
            ImageFormat::R8G8B8A8Sint => vk::Format::R8G8B8A8_SINT,
            ImageFormat::R8G8B8A8Srgb => vk::Format::R8G8B8A8_SRGB,

            ImageFormat::B8G8R8A8Unorm => vk::Format::B8G8R8A8_UNORM,
            ImageFormat::B8G8R8A8Snorm => vk::Format::B8G8R8A8_SNORM,
            ImageFormat::B8G8R8A8Uint => vk::Format::B8G8R8A8_UINT,
            ImageFormat::B8G8R8A8Sint => vk::Format::B8G8R8A8_SINT,
            ImageFormat::B8G8R8A8Srgb => vk::Format::B8G8R8A8_SRGB,

            ImageFormat::R16G16B16A16Unorm => vk::Format::R16G16B16A16_UNORM,
            ImageFormat::R16G16B16A16Snorm => vk::Format::R16G16B16A16_SNORM,
            ImageFormat::R16G16B16A16Uint => vk::Format::R16G16B16A16_UINT,
            ImageFormat::R16G16B16A16Sint => vk::Format::R16G16B16A16_SINT,
            ImageFormat::R16G16B16A16Sfloat => vk::Format::R16G16B16A16_SFLOAT,

            ImageFormat::R32G32B32A32Uint => vk::Format::R32G32B32A32_UINT,
            ImageFormat::R32G32B32A32Sint => vk::Format::R32G32B32A32_SINT,
            ImageFormat::R32G32B32A32Sfloat => vk::Format::R32G32B32A32_SFLOAT,

            // Depth/Stencil formats
            ImageFormat::D16Unorm => vk::Format::D16_UNORM,
            ImageFormat::D32Sfloat => vk::Format::D32_SFLOAT,
            ImageFormat::D16UnormS8Uint => vk::Format::D16_UNORM_S8_UINT,
            ImageFormat::D24UnormS8Uint => vk::Format::D24_UNORM_S8_UINT,
            ImageFormat::D32SfloatS8Uint => vk::Format::D32_SFLOAT_S8_UINT,

            // Compressed formats
            ImageFormat::BC1RgbUnorm => vk::Format::BC1_RGB_UNORM_BLOCK,
            ImageFormat::BC1RgbSrgb => vk::Format::BC1_RGB_SRGB_BLOCK,
            ImageFormat::BC1RgbaUnorm => vk::Format::BC1_RGBA_UNORM_BLOCK,
            ImageFormat::BC1RgbaSrgb => vk::Format::BC1_RGBA_SRGB_BLOCK,
            ImageFormat::BC2Unorm => vk::Format::BC2_UNORM_BLOCK,
            ImageFormat::BC2Srgb => vk::Format::BC2_SRGB_BLOCK,
            ImageFormat::BC3Unorm => vk::Format::BC3_UNORM_BLOCK,
            ImageFormat::BC3Srgb => vk::Format::BC3_SRGB_BLOCK,
            ImageFormat::BC4Unorm => vk::Format::BC4_UNORM_BLOCK,
            ImageFormat::BC4Snorm => vk::Format::BC4_SNORM_BLOCK,
            ImageFormat::BC5Unorm => vk::Format::BC5_UNORM_BLOCK,
            ImageFormat::BC5Snorm => vk::Format::BC5_SNORM_BLOCK,
            ImageFormat::BC6hUfloat => vk::Format::BC6H_UFLOAT_BLOCK,
            ImageFormat::BC6hSfloat => vk::Format::BC6H_SFLOAT_BLOCK,
            ImageFormat::BC7Unorm => vk::Format::BC7_UNORM_BLOCK,
            ImageFormat::BC7Srgb => vk::Format::BC7_SRGB_BLOCK,

            // 10-bit formats
            ImageFormat::A2R10G10B10UnormPack32 => vk::Format::A2R10G10B10_UNORM_PACK32,
            ImageFormat::A2R10G10B10UintPack32 => vk::Format::A2R10G10B10_UINT_PACK32,
        }
    }

    pub fn from_vk(format: vk::Format) -> Option<Self> {
        Some(match format {
            // 8-bit formats
            vk::Format::R8_UNORM => ImageFormat::R8Unorm,
            vk::Format::R8_SNORM => ImageFormat::R8Snorm,
            vk::Format::R8_UINT => ImageFormat::R8Uint,
            vk::Format::R8_SINT => ImageFormat::R8Sint,
            vk::Format::R8_SRGB => ImageFormat::R8Srgb,

            // 16-bit formats
            vk::Format::R16_UNORM => ImageFormat::R16Unorm,
            vk::Format::R16_SNORM => ImageFormat::R16Snorm,
            vk::Format::R16_UINT => ImageFormat::R16Uint,
            vk::Format::R16_SINT => ImageFormat::R16Sint,
            vk::Format::R16_SFLOAT => ImageFormat::R16Sfloat,

            // 32-bit formats
            vk::Format::R32_UINT => ImageFormat::R32Uint,
            vk::Format::R32_SINT => ImageFormat::R32Sint,
            vk::Format::R32_SFLOAT => ImageFormat::R32Sfloat,

            // RG formats
            vk::Format::R8G8_UNORM => ImageFormat::R8G8Unorm,
            vk::Format::R8G8_SNORM => ImageFormat::R8G8Snorm,
            vk::Format::R8G8_UINT => ImageFormat::R8G8Uint,
            vk::Format::R8G8_SINT => ImageFormat::R8G8Sint,
            vk::Format::R8G8_SRGB => ImageFormat::R8G8Srgb,

            vk::Format::R16G16_UNORM => ImageFormat::R16G16Unorm,
            vk::Format::R16G16_SNORM => ImageFormat::R16G16Snorm,
            vk::Format::R16G16_UINT => ImageFormat::R16G16Uint,
            vk::Format::R16G16_SINT => ImageFormat::R16G16Sint,
            vk::Format::R16G16_SFLOAT => ImageFormat::R16G16Sfloat,

            vk::Format::R32G32_UINT => ImageFormat::R32G32Uint,
            vk::Format::R32G32_SINT => ImageFormat::R32G32Sint,
            vk::Format::R32G32_SFLOAT => ImageFormat::R32G32Sfloat,

            // RGB formats
            vk::Format::R32G32B32_UINT => ImageFormat::R32G32B32Uint,
            vk::Format::R32G32B32_SINT => ImageFormat::R32G32B32Sint,
            vk::Format::R32G32B32_SFLOAT => ImageFormat::R32G32B32Sfloat,

            // RGBA formats
            vk::Format::R8G8B8A8_UNORM => ImageFormat::R8G8B8A8Unorm,
            vk::Format::R8G8B8A8_SNORM => ImageFormat::R8G8B8A8Snorm,
            vk::Format::R8G8B8A8_UINT => ImageFormat::R8G8B8A8Uint,
            vk::Format::R8G8B8A8_SINT => ImageFormat::R8G8B8A8Sint,
            vk::Format::R8G8B8A8_SRGB => ImageFormat::R8G8B8A8Srgb,

            vk::Format::B8G8R8A8_UNORM => ImageFormat::B8G8R8A8Unorm,
            vk::Format::B8G8R8A8_SNORM => ImageFormat::B8G8R8A8Snorm,
            vk::Format::B8G8R8A8_UINT => ImageFormat::B8G8R8A8Uint,
            vk::Format::B8G8R8A8_SINT => ImageFormat::B8G8R8A8Sint,
            vk::Format::B8G8R8A8_SRGB => ImageFormat::B8G8R8A8Srgb,

            vk::Format::R16G16B16A16_UNORM => ImageFormat::R16G16B16A16Unorm,
            vk::Format::R16G16B16A16_SNORM => ImageFormat::R16G16B16A16Snorm,
            vk::Format::R16G16B16A16_UINT => ImageFormat::R16G16B16A16Uint,
            vk::Format::R16G16B16A16_SINT => ImageFormat::R16G16B16A16Sint,
            vk::Format::R16G16B16A16_SFLOAT => ImageFormat::R16G16B16A16Sfloat,

            vk::Format::R32G32B32A32_UINT => ImageFormat::R32G32B32A32Uint,
            vk::Format::R32G32B32A32_SINT => ImageFormat::R32G32B32A32Sint,
            vk::Format::R32G32B32A32_SFLOAT => ImageFormat::R32G32B32A32Sfloat,

            // Depth/Stencil formats
            vk::Format::D16_UNORM => ImageFormat::D16Unorm,
            vk::Format::D32_SFLOAT => ImageFormat::D32Sfloat,
            vk::Format::D16_UNORM_S8_UINT => ImageFormat::D16UnormS8Uint,
            vk::Format::D24_UNORM_S8_UINT => ImageFormat::D24UnormS8Uint,
            vk::Format::D32_SFLOAT_S8_UINT => ImageFormat::D32SfloatS8Uint,

            // Compressed formats
            vk::Format::BC1_RGB_UNORM_BLOCK => ImageFormat::BC1RgbUnorm,
            vk::Format::BC1_RGB_SRGB_BLOCK => ImageFormat::BC1RgbSrgb,
            vk::Format::BC1_RGBA_UNORM_BLOCK => ImageFormat::BC1RgbaUnorm,
            vk::Format::BC1_RGBA_SRGB_BLOCK => ImageFormat::BC1RgbaSrgb,
            vk::Format::BC2_UNORM_BLOCK => ImageFormat::BC2Unorm,
            vk::Format::BC2_SRGB_BLOCK => ImageFormat::BC2Srgb,
            vk::Format::BC3_UNORM_BLOCK => ImageFormat::BC3Unorm,
            vk::Format::BC3_SRGB_BLOCK => ImageFormat::BC3Srgb,
            vk::Format::BC4_UNORM_BLOCK => ImageFormat::BC4Unorm,
            vk::Format::BC4_SNORM_BLOCK => ImageFormat::BC4Snorm,
            vk::Format::BC5_UNORM_BLOCK => ImageFormat::BC5Unorm,
            vk::Format::BC5_SNORM_BLOCK => ImageFormat::BC5Snorm,
            vk::Format::BC6H_UFLOAT_BLOCK => ImageFormat::BC6hUfloat,
            vk::Format::BC6H_SFLOAT_BLOCK => ImageFormat::BC6hSfloat,
            vk::Format::BC7_UNORM_BLOCK => ImageFormat::BC7Unorm,
            vk::Format::BC7_SRGB_BLOCK => ImageFormat::BC7Srgb,

            // 10-bit formats
            vk::Format::A2R10G10B10_UNORM_PACK32 => ImageFormat::A2R10G10B10UnormPack32,
            vk::Format::A2R10G10B10_UINT_PACK32 => ImageFormat::A2R10G10B10UintPack32,

            _ => return None,
        })
    }
}

