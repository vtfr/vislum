use std::ops::Deref;

pub mod buffer;
pub mod command;
pub mod device;
pub mod image;
pub mod image_view;
pub mod sampler;
pub mod instance;
pub mod memory;
pub mod queue;
pub mod surface;
pub mod swapchain;
pub mod sync;

mod macros;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub const V1_1: Self = Self::new(1, 1, 0);
    pub const V1_2: Self = Self::new(1, 2, 0);
    pub const V1_3: Self = Self::new(1, 3, 0);

    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn from_vk(version: u32) -> Self {
        let major = ash::vk::api_version_major(version) as u8;
        let minor = ash::vk::api_version_minor(version) as u8;
        let patch = ash::vk::api_version_patch(version) as u8;

        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn to_vk(self) -> u32 {
        ash::vk::make_api_version(0, self.major as u32, self.minor as u32, self.patch as u32)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// A trait for objects that wrap Vulkan handles.
pub trait VkHandle {
    type Handle: ash::vk::Handle;

    /// Returns the Vulkan handle of the object.
    fn vk_handle(&self) -> Self::Handle;
}

impl VkHandle for ash::Instance {
    type Handle = ash::vk::Instance;

    fn vk_handle(&self) -> Self::Handle {
        self.handle()
    }
}

impl VkHandle for ash::Device {
    type Handle = ash::vk::Device;

    fn vk_handle(&self) -> Self::Handle {
        self.handle()
    }
}

/// A trait for objects that wrap Ash handles.
pub trait AshHandle {
    type Handle: VkHandle;

    fn ash_handle(&self) -> &Self::Handle;
}

// Blanket implementation for all Ash handles.
//
// All ash handles are VkHandles, as they wrap Vulkan handles.
impl<T> VkHandle for T
where
    T: AshHandle,
{
    type Handle = <T::Handle as VkHandle>::Handle;

    fn vk_handle(&self) -> Self::Handle {
        self.ash_handle().vk_handle()
    }
}

pub struct DebugWrapper<T: ash::vk::Handle>(pub T);

impl<T> Deref for DebugWrapper<T>
where
    T: ash::vk::Handle,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::fmt::Debug for DebugWrapper<T>
where
    T: ash::vk::Handle + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", self.0.as_raw())
    }
}

pub struct AshDebugWrapper<T: VkHandle>(pub T);

impl<T> Deref for AshDebugWrapper<T>
where
    T: VkHandle,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::fmt::Debug for AshDebugWrapper<T>
where
    T: VkHandle,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", ash::vk::Handle::as_raw(self.0.vk_handle()))
    }
}
