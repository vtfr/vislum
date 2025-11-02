use std::sync::Arc;

use ash::vk;
use winit::raw_window_handle::HasDisplayHandle;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::Window;

use crate::{AshHandle, DebugWrapper, VkHandle, device::PhysicalDevice, instance::Instance};

pub struct Surface {
    instance: Arc<Instance>,
    surface: DebugWrapper<vk::SurfaceKHR>,
    surface_loader: ash::khr::surface::Instance,
}

impl Surface {
    /// Creates a new surface from a winit window.
    pub fn new(instance: Arc<Instance>, window: &Window) -> Arc<Self> {
        let library = instance.library();
        let surface_loader =
            ash::khr::surface::Instance::new(&library.entry, instance.ash_handle());

        let surface = unsafe {
            ash_window::create_surface(
                &library.entry,
                instance.ash_handle(),
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
        }
        .unwrap();

        Arc::new(Self {
            instance,
            surface: DebugWrapper(surface),
            surface_loader,
        })
    }

    /// Returns the surface loader.
    pub fn surface_loader(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    /// Gets the physical device surface capabilities.
    pub fn get_capabilities(&self, physical_device: &PhysicalDevice) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(
                    physical_device.vk_handle(),
                    self.surface.0,
                )
        }
        .unwrap()
    }

    /// Gets the supported surface formats for a physical device.
    pub fn get_formats(&self, physical_device: &PhysicalDevice) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device.vk_handle(), self.surface.0)
        }
        .unwrap()
    }

    /// Gets the supported present modes for a physical device.
    pub fn get_present_modes(&self, physical_device: &PhysicalDevice) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(
                    physical_device.vk_handle(),
                    self.surface.0,
                )
        }
        .unwrap()
    }

    /// Checks if a queue family supports presentation to this surface.
    pub fn get_physical_device_surface_support(
        &self,
        physical_device: &PhysicalDevice,
        queue_family_index: u32,
    ) -> bool {
        unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device.vk_handle(),
                queue_family_index,
                self.surface.0,
            )
        }
        .unwrap()
    }

    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
}

impl crate::VkHandle for Surface {
    type Handle = vk::SurfaceKHR;

    fn vk_handle(&self) -> Self::Handle {
        self.surface.0
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface.0, None);
        }
    }
}
