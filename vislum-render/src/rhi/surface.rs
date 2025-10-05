use std::sync::Arc;

use ash::{self, vk, Entry};
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};

use crate::rhi::{Instance, PhysicalDevice, VulkanHandle};

/// A Vulkan surface that represents a window or display surface.
pub struct Surface {
    /// The instance that this surface belongs to.
    instance: Arc<Instance>,
    
    /// The raw Vulkan surface handle.
    surface: vk::SurfaceKHR,
}

static_assertions::assert_impl_all!(Surface: Send, Sync);

impl VulkanHandle for Surface {
    type Handle = vk::SurfaceKHR;

    fn vk_handle(&self) -> Self::Handle {
        self.surface
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSurfaceError {
    #[error("Failed to create Vulkan surface: {0}")]
    SurfaceCreation(vk::Result),
    
    #[error("Failed to get surface capabilities: {0}")]
    SurfaceCapabilities(vk::Result),
    
    #[error("Failed to get surface formats: {0}")]
    SurfaceFormats(vk::Result),
    
    #[error("Failed to get surface present modes: {0}")]
    PresentModes(vk::Result),
}

/// Trait for window handles that can be used to create surfaces.
pub trait WindowHandle: HasWindowHandle + HasDisplayHandle + Send + Sync {}

impl<T> WindowHandle for T where T: HasWindowHandle + HasDisplayHandle + Send + Sync {}

/// Description for creating a surface.
pub struct SurfaceDescription<'a, W> {
    /// The window to create the surface for.
    pub window: &'a W,
}

impl<'a, W> SurfaceDescription<'a, W> 
where 
    W: HasDisplayHandle + HasWindowHandle,
{
    pub fn new(window: &'a W) -> Self {
        Self { 
            window,
        }
    }
}

/// Information about surface capabilities and supported formats.
#[derive(Debug, Clone)]
pub struct SurfaceInfo {
    /// The surface capabilities.
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    
    /// The supported surface formats.
    pub formats: Vec<vk::SurfaceFormatKHR>,
    
    /// The supported present modes.
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl Surface {
    /// Creates a new surface from a window.
    pub fn new(
        instance: Arc<Instance>,
        description: SurfaceDescription,
    ) -> Result<Self, CreateSurfaceError> {
        let surface = unsafe {
            ash_window::create_surface(
                &ash::Entry::load().unwrap(),
                instance.ash_handle(),
                description.window.as_ref(),
                None,
            )
        }
        .map_err(CreateSurfaceError::SurfaceCreation)?;

        Ok(Self {
            instance,
            surface,
        })
    }

    /// Gets information about the surface capabilities and supported formats for a physical device.
    pub fn get_surface_info(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<SurfaceInfo, CreateSurfaceError> {
        let capabilities = unsafe {
            self.instance
                .ash_handle().get_physical_device_surface_capabilities_khr(
                    physical_device.vk_physical_device(),
                    self.surface,
                )
        }
        .map_err(CreateSurfaceError::SurfaceCapabilities)?;

        let formats = unsafe {
            self.instance
                .ash_handle().get_physical_device_surface_formats_khr(
                    physical_device.vk_physical_device(),
                    self.surface,
                )
        }
        .map_err(CreateSurfaceError::SurfaceFormats)?;

        let present_modes = unsafe {
            self.instance
                .ash_handle().get_physical_device_surface_present_modes_khr(
                    physical_device.vk_physical_device(),
                    self.surface,
                )
        }
        .map_err(CreateSurfaceError::PresentModes)?;

        Ok(SurfaceInfo {
            capabilities,
            formats,
            present_modes,
        })
    }

    /// Returns the raw Vulkan surface handle.
    pub fn ash_surface(&self) -> vk::SurfaceKHR {
        self.surface
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_surface_khr(self.surface);
        }
    }
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("surface", &self.surface)
            .finish()
    }
}