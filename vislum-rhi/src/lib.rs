pub mod device;
pub mod instance;
pub mod macros;
pub mod version;

pub trait VkHandle {
    type Handle: ash::vk::Handle;

    /// Returns the inner handle of the object.
    fn vk_handle(&self) -> Self::Handle;

    #[inline]
    fn vk_object_type(&self) -> ash::vk::ObjectType {
        <Self::Handle as ash::vk::Handle>::TYPE
    }
}

pub trait VkRawHandle {
    fn vk_raw_handle(&self) -> u64;
}

impl<T> VkRawHandle for T
where
    T: VkHandle,
{
    #[inline]
    fn vk_raw_handle(&self) -> u64 {
        ash::vk::Handle::as_raw(self.vk_handle())
    }
}