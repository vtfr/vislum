pub mod device;
pub mod instance;
pub mod physical;
pub mod sync;

pub use device::*;
pub use instance::*;
pub use physical::*;
pub use sync::*;

/// A trait for types that represent Vulkan handles.
pub trait VulkanHandle {
    /// The type of the Vulkan handle.
    type Handle;
    
    /// Returns the Vulkan handle.
    fn vk_handle(&self) -> Self::Handle;
}