use std::{
    fmt::Debug, ops::Deref, sync::Arc
};

/// A render device.
/// 
/// This is a wrapper around the wgpu::Device that provides shared ownership of the device.
#[derive(Clone)]
pub struct RenderDevice(Arc<wgpu::Device>);

static_assertions::assert_impl_all!(RenderDevice: Send, Sync);

impl Debug for RenderDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RenderDevice")
    }
}

impl From<wgpu::Device> for RenderDevice {
    fn from(device: wgpu::Device) -> Self {
        Self(Arc::new(device))
    }
}

impl From<Arc<wgpu::Device>> for RenderDevice {
    fn from(device: Arc<wgpu::Device>) -> Self {
        Self(device)
    }
}

impl Deref for RenderDevice {
    type Target = wgpu::Device;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
