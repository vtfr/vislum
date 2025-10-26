use std::sync::Arc;

use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{AshHandle, VkHandle, instance::Instance};

pub struct Surface {
    instance: Arc<Instance>,
    surface: vk::SurfaceKHR,
}

impl VkHandle for Surface {
    type Handle = vk::SurfaceKHR;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.surface
    }
}

impl Surface {
    pub fn new(
        instance: Arc<Instance>,
        window: &impl HasWindowHandle,
        display: &impl HasDisplayHandle,
    ) -> Arc<Self> {
        let ash_entry = instance.library().ash_handle();
        let ash_instance = instance.ash_handle();

        let surface = unsafe {
            ash_window::create_surface(
                ash_entry,
                ash_instance,
                display.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .expect("Failed to create surface")
        };

        Arc::new(Self { instance, surface })
    }

    #[inline]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .ash_khr_surface_handle()
                .destroy_surface(self.surface, None);
        }
    }
}
