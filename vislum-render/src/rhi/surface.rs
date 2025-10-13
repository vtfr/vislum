use std::{ffi::CStr, sync::Arc};

use ash::{khr, vk};

use super::instance::Instance;

/// A Vulkan surface for presenting rendered images to a window or display.
#[derive(Debug)]
pub struct Surface {
    instance: Arc<Instance>,
    surface: vk::SurfaceKHR,
}

#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    #[error("unsupported surface type: only Wayland, Xlib, XCB, and Win32 are supported (what fucking underground windowing system are you using?)")]
    UnsupportedSurfaceType,
    #[error("required surface extension not enabled: {0}")]
    ExtensionNotEnabled(&'static str),
}

impl SurfaceError {
    #[inline]
    pub(crate) const fn extension_not_enabled(extension: &'static CStr) -> Self {
        match extension.to_str() {
            Ok(extension) => Self::ExtensionNotEnabled(extension),
            Err(_) => unreachable!(),
        }
    }
}

impl Surface {
    /// Create a surface from a winit window.
    pub fn new(
        instance: Arc<Instance>,
        window: &winit::window::Window,
    ) -> Result<Arc<Self>, SurfaceError> {
        use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
        
        let display_handle = window.display_handle().unwrap();
        let window_handle = window.window_handle().unwrap();
        
        let surface = unsafe {
            match (display_handle.as_raw(), window_handle.as_raw()) {
                #[cfg(target_os = "linux")]
                (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) => {
                    Self::create_wayland_surface(&instance, display, window)?
                }
                #[cfg(target_os = "linux")]
                (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(window)) => {
                    Self::create_xlib_surface(&instance, display, window)?
                }
                #[cfg(target_os = "linux")]
                (RawDisplayHandle::Xcb(display), RawWindowHandle::Xcb(window)) => {
                    Self::create_xcb_surface(&instance, display, window)?
                }
                #[cfg(target_os = "windows")]
                (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
                    Self::create_win32_surface(&instance, window)?
                }
                (_, _) => return Err(SurfaceError::UnsupportedSurfaceType),
            }
        };
        
        Ok(Arc::new(Self {
            instance,
            surface,
        }))
    }

    #[cfg(target_os = "linux")]
    unsafe fn create_wayland_surface(
        instance: &Arc<Instance>,
        display: winit::raw_window_handle::WaylandDisplayHandle,
        window: winit::raw_window_handle::WaylandWindowHandle,
    ) -> Result<vk::SurfaceKHR, SurfaceError> {
        use crate::rhi::instance::load_fn_for;

        if !instance.extensions().khr_wayland_surface {
            return Err(SurfaceError::extension_not_enabled(khr::wayland_surface::NAME));
        }

        let mut load_fn = load_fn_for(instance.entry(), instance.vk_instance());
        let fns = khr::wayland_surface::InstanceFn::load(&mut load_fn);
        
        let create_info = vk::WaylandSurfaceCreateInfoKHR::default()
            .display(display.display.as_ptr())
            .surface(window.surface.as_ptr());
        
        let mut surface_khr = vk::SurfaceKHR::null();
        unsafe {
            (fns.create_wayland_surface_khr)(
                instance.vk_instance(),
                &create_info,
                std::ptr::null(),
                &mut surface_khr,
            ).result().unwrap();
        }
        
        Ok(surface_khr)
    }

    #[cfg(target_os = "linux")]
    unsafe fn create_xlib_surface(
        instance: &Arc<Instance>,
        display: winit::raw_window_handle::XlibDisplayHandle,
        window: winit::raw_window_handle::XlibWindowHandle,
    ) -> Result<vk::SurfaceKHR, SurfaceError> {
        use crate::rhi::instance::load_fn_for;

        if !instance.extensions().khr_xlib_surface {
            return Err(SurfaceError::extension_not_enabled(khr::xlib_surface::NAME));
        }

        let mut load_fn = load_fn_for(instance.entry(), instance.vk_instance());
        let fns = khr::xlib_surface::InstanceFn::load(&mut load_fn);
        
        let create_info = vk::XlibSurfaceCreateInfoKHR::default()
            .dpy(display.display.unwrap().as_ptr() as *mut _)
            .window(window.window);
        
        let mut surface_khr = vk::SurfaceKHR::null();
        unsafe {
            (fns.create_xlib_surface_khr)(
                instance.vk_instance(),
                &create_info,
                std::ptr::null(),
                &mut surface_khr,
            ).result().unwrap();
        }
        
        Ok(surface_khr)
    }

    #[cfg(target_os = "linux")]
    unsafe fn create_xcb_surface(
        instance: &Arc<Instance>,
        display: winit::raw_window_handle::XcbDisplayHandle,
        window: winit::raw_window_handle::XcbWindowHandle,
    ) -> Result<vk::SurfaceKHR, SurfaceError> {
        use crate::rhi::instance::load_fn_for;

        if !instance.extensions().khr_xcb_surface {
            return Err(SurfaceError::extension_not_enabled(khr::xcb_surface::NAME));
        }

        let mut load_fn = load_fn_for(instance.entry(), instance.vk_instance());
        let fns = khr::xcb_surface::InstanceFn::load(&mut load_fn);
        
        let create_info = vk::XcbSurfaceCreateInfoKHR::default()
            .connection(display.connection.unwrap().as_ptr())
            .window(window.window.get());
        
        let mut surface_khr = vk::SurfaceKHR::null();
        unsafe {
            (fns.create_xcb_surface_khr)(
                instance.vk_instance(),
                &create_info,
                std::ptr::null(),
                &mut surface_khr,
            ).result().unwrap();
        }
        
        Ok(surface_khr)
    }

    #[cfg(target_os = "windows")]
    unsafe fn create_win32_surface(
        instance: &Arc<Instance>,
        window: winit::raw_window_handle::Win32WindowHandle,
    ) -> Result<vk::SurfaceKHR, SurfaceError> {
        use crate::rhi::instance::load_fn_for;

        if !instance.extensions().khr_win32_surface {
            return Err(SurfaceError::extension_not_enabled(khr::win32_surface::NAME));
        }

        let mut load_fn = load_fn_for(instance.entry(), instance.vk_instance());
        let fns = khr::win32_surface::InstanceFn::load(&mut load_fn);
        
        let create_info = vk::Win32SurfaceCreateInfoKHR::default()
            .hinstance(window.hinstance.unwrap().get() as *const _)
            .hwnd(window.hwnd.get() as *const _);
        
        let mut surface_khr = vk::SurfaceKHR::null();
        unsafe {
            (fns.create_win32_surface_khr)(
                instance.vk_instance(),
                &create_info,
                std::ptr::null(),
                &mut surface_khr,
            ).result().unwrap();
        }
        
        Ok(surface_khr)
    }

    #[inline]
    pub fn handle(&self) -> vk::SurfaceKHR {
        self.surface
    }
    
    #[inline]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        // SAFETY: khr_surface MUST be loaded, otherwise the surface would not have been created
        let fns = self.instance.fns().khr_surface().unwrap();
        
        unsafe {
            (fns.destroy_surface_khr)(self.instance.vk_instance(), self.surface, std::ptr::null());
        }
    }
}
