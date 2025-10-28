// use std::{marker::PhantomData, sync::{Arc, Mutex, atomic::AtomicBool}};

// use vulkano::{
//     buffer::{BufferContents, BufferUsage, Subbuffer},
//     memory::allocator::{AllocationCreateInfo, MemoryAllocator},
// };

// enum TransferState<T> {
//     Transferred,
//     Pending {
//         staging: Subbuffer<T>,
//     },
// }

// impl<T> Default for TransferState<T> {
//     fn default() -> Self {
//         Self::Transferred
//     }
// }

// pub trait ImmutableBuffer {
//     /// Returns true if the buffer has been transferred.
//     fn transfered(&self) -> bool;
// } 

// /// An immutable buffer.
// ///
// /// This buffer is immutable and can only be uploaded to once, at creation time.
// struct RawImmutableBuffer<T> {
//     buffer: vulkano::buffer::Subbuffer<T>,
//     transfered: AtomicBool,
//     state: Mutex<TransferState<T>>,
// }

// impl<T> RawImmutableBuffer<T>
// where
//     T: BufferContents,
// {
//     pub fn new(
//         allocator: Arc<dyn MemoryAllocator>,
//         buffer_usage: BufferUsage,
//         data: T,
//     ) -> Arc<Self> {
//         let staging = vulkano::buffer::Buffer::from_data(
//             allocator.clone(),
//             vulkano::buffer::BufferCreateInfo {
//                 usage: BufferUsage::TRANSFER_SRC,
//                 ..Default::default()
//             },
//             AllocationCreateInfo::default(),
//             data,
//         )
//         .unwrap();

//         let buffer = vulkano::buffer::Buffer::new_sized::<T>(
//             allocator.clone(),
//             vulkano::buffer::BufferCreateInfo {
//                 size: std::mem::size_of::<T>() as u64,
//                 // Ensure the buffer is usable for transfer and the specified usage.
//                 usage: BufferUsage::TRANSFER_DST | buffer_usage,
//                 ..Default::default()
//             },
//             AllocationCreateInfo::default(),
//         )
//         .unwrap();

//         Arc::new(Self { 
//             buffer, 
//             transfered: AtomicBool::new(false), 
//             state: Mutex::new(TransferState::Pending { staging }),
//         }) 
//     }
// }

// /// A uniform buffer.
// pub struct Uniform<T> {
//     buffer: RawImmutableBuffer<T>,
// }

// impl<T> Uniform<T>
// where
//     T: BufferContents,
// {
//     pub fn new(
//         allocator: Arc<dyn MemoryAllocator>,
//         data: T,
//     ) -> Self {
//         Self {
//             buffer: RawImmutableBuffer::new(allocator, BufferUsage::UNIFORM_BUFFER, data),
//         }
//     }
// }

// struct RingBuffer<T> {
//     buffers: Vec<Subbuffer<T>>,
// }