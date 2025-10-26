use std::sync::Arc;

use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{VkHandle, instance::Instance};

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
    pub fn new(instance: Arc<Instance>, window: &impl HasWindowHandle, display: &impl HasDisplayHandle) -> Arc<Self> {
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry(),
                &instance.instance(),
                display.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .expect("Failed to create surface")
        };

        Arc::new(Self {
            instance,
            surface,
        })
    }

    #[inline]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.instance.ash_khr_surface().destroy_surface(self.surface, None);
        }
    }
}

