use std::marker::PhantomData;

use encase::{ShaderType, internal::WriteInto};
use wgpu::util::DeviceExt;

/// A uniform buffer.
pub struct Uniform<T> {
    buffer: wgpu::Buffer,
    staging: Vec<u8>,
    phantom: PhantomData<T>,
}

impl<T> std::fmt::Debug for Uniform<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Uniform<{}>", std::any::type_name::<T>())
    }
}

impl<T> Uniform<T> 
where 
    T: ShaderType + WriteInto 
{
    /// Creates a new uniform buffer with the given data.
    pub fn new(device: &wgpu::Device, item: &T) -> Self {
        let mut staging_buffer = Vec::new();
        Self::write(&mut staging_buffer, item);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: &staging_buffer,
            usage: wgpu::BufferUsages::UNIFORM,
        });

        Self { buffer, staging: staging_buffer, phantom: PhantomData }
    }

    /// Creates a new uniform buffer with the default value of the type.
    #[inline]
    pub fn new_with_default(device: &wgpu::Device) -> Self
    where
        T: Default,
    {
        Self::new(device, &T::default())
    }

    /// Updates the uniform buffer with the new data.
    pub fn update(&mut self, queue: &wgpu::Queue, item: &T) {
        self.staging.clear();
        Self::write(&mut self.staging, item);
        queue.write_buffer(&self.buffer, 0, &self.staging);
    }

    #[inline]
    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn write(staging_buffer: &mut Vec<u8>, item: &T) {
        let mut writer = encase::UniformBuffer::new(staging_buffer);
        writer.write(item).unwrap();
    }
}