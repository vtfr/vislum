use std::{collections::HashMap, ops::Range, sync::{Arc, Mutex}};

use crate::rhi::{self, command::{AccessFlags, AccessFlags, CommandBuffer, ImageLayout}, device::Device};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u32);

pub(crate) enum Command {
    Draw {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
    SetVertexBuffer {
        buffer: BufferHandle,
        offset: u64,
    },
    SetIndexBuffer {
        buffer: BufferHandle,
        offset: u64,
    },
    UploadBuffer {
        buffer: BufferHandle,
        data: Vec<u8>,
    },
}

pub struct BufferState {
    pub access: AccessFlags,
}

pub struct TextureState {
    pub access: AccessFlags,
    pub layout: ImageLayout,
}

pub struct ResourceStateTracker {
    buffers: HashMap<BufferHandle, BufferState>,
    textures: HashMap<TextureHandle, TextureState>,
}
