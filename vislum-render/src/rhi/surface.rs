use std::sync::Arc;

use crate::rhi::instance::{Instance, InstanceExtensions};

/// A Vulkan surface for presenting rendered images to a window or display.
#[derive(Debug)]
pub struct Surface {
    instance: Arc<Instance>,
    handle: ash::vk::SurfaceKHR,
}

impl Surface {
    /// Create a surface from a winit window.
    pub fn new(instance: Arc<Instance>, window: &winit::window::Window) -> Arc<Self> {
        use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

        if let Err(missing_extensions) = Self::sanity_check_required_extensions(&instance) {
            log::error!("rhi error: missing required extensions for creating surface: {:?}", missing_extensions);
            unreachable!();
        }

        let display_handle = window.display_handle().unwrap().as_raw();
        let window_handle = window.window_handle().unwrap().as_raw();

        let surface = unsafe {
            ash_window::create_surface(
                instance.entry(),
                instance.handle(),
                display_handle,
                window_handle,
                None,
            )
        }
        .expect("failed creating surface");

        Arc::new(Self {
            instance,
            handle: surface,
        })
    }

    /// Get the handle to the surface.
    #[inline]
    pub(in crate::rhi) fn handle(&self) -> ash::vk::SurfaceKHR {
        self.handle
    }

    /// Get the required extensions for the surface based on the target operating system.
    ///
    /// This is used to create the instance extensions. As we currently support linux and windows,
    /// we only need to check for these operating system.
    pub(in crate::rhi) fn required_instance_extensions() -> InstanceExtensions {
        // Always enable the KHR_surface extension.
        let mut instance_extensions = InstanceExtensions {
            khr_surface: true,
            ..Default::default()
        };

        // Enable all surface extensions for Linux.
        if cfg!(target_os = "linux") {
            instance_extensions.khr_wayland_surface = true;
            instance_extensions.khr_xlib_surface = true;
            instance_extensions.khr_xcb_surface = true;
        }

        // Enable the Win32 surface extension for Windows.
        if cfg!(target_os = "windows") {
            instance_extensions.khr_win32_surface = true;
        }

        instance_extensions
    }

    /// Ensure that the instance extensions are enabled for the instance.
    /// 
    /// Returns the missing extensions.
    fn sanity_check_required_extensions(instance: &Arc<Instance>) -> Result<(), InstanceExtensions> {
        let missing_extensions = instance.extensions().difference(&Self::required_instance_extensions());
        if !missing_extensions.is_empty() {
            return Err(missing_extensions);
        }

        Ok(())

    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .khr_surface_handle()
                .unwrap()
                .destroy_surface(self.handle, None);
        }
    }
}
