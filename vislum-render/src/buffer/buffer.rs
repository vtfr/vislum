use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
};

/// A trait for objects that own a buffer.
pub trait TypedBufferOwner<T> {
    /// Returns the buffer associated with the object.
    fn buffer(&self) -> &Subbuffer<T>;
}

/// A typed uniform buffer.
pub struct Uniform<T> {
    device: Arc<Device>,
    buffer: Subbuffer<T>,
}


impl<T> TypedBufferOwner<T> for Uniform<T> 
where 
    T: BufferContents, 
    {
    fn buffer(&self) -> &Subbuffer<T> {
        &self.buffer
    }
}

impl<T> Uniform<T> 
where 
    T: BufferContents,
{
    pub fn new(device: Arc<Device>, allocator: Arc<dyn MemoryAllocator>) -> Arc<Self> {
        let buffer = create_device_buffer::<T>(device.clone(), allocator);

        Arc::new(Self {
            device,
            buffer,
        })
    }
}

fn create_device_buffer<T>(
    device: Arc<Device>,
    allocator: Arc<dyn MemoryAllocator>,
) -> Subbuffer<[u8]>
where
    T: BufferContents,
{
    let buffer = Buffer::new_sized::<T>(
        allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST | BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .unwrap();

    // Tag this object.
    if device.instance().enabled_extensions().ext_debug_utils {
        device
            .set_debug_utils_object_name(buffer.buffer(), Some("Buffer"))
            .unwrap();
    }

    buffer.into_bytes()
}
