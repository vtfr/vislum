#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

impl Extent2D {
    #[inline]
    pub const fn to_vk(self) -> ash::vk::Extent2D {
        ash::vk::Extent2D {
            width: self.width,
            height: self.height,
        }
    }

    #[inline]
    pub const fn from_vk(extent: ash::vk::Extent2D) -> Self {
        Self {
            width: extent.width,
            height: extent.height,
        }
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extent3D {
    #[inline]
    pub const fn to_vk(self) -> ash::vk::Extent3D {
        ash::vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }

    #[inline]
    pub const fn from_vk(extent: ash::vk::Extent3D) -> Self {
        Self {
            width: extent.width,
            height: extent.height,
            depth: extent.depth,
        }
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0 || self.depth == 0
    }
}

