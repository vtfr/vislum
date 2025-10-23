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

pub struct CommandEncoder {
    local_state: ResourceStateTracker,
    command_buffer: rhi::command::CommandBuffer,
}

impl CommandEncoder {
    pub fn new() -> Self {
        Self { local_state: ResourceStateTracker::default(), command_buffer: rhi::command::CommandBuffer::new() }
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.command_buffer.draw(vertices, instances);
    }

    pub fn set_vertex_buffer(&mut self, buffer: BufferHandle, offset: u64) {
        self.command_buffer.set_vertex_buffer(buffer, offset);
    }
    
    pub fn set_index_buffer(&mut self, buffer: BufferHandle, offset: u64) {
        self.command_buffer.set_index_buffer(buffer, offset);
    }

    pub fn upload_buffer(&mut self, buffer: BufferHandle, data: Vec<u8>) {
        self.command_buffer.upload_buffer(buffer, data);
    }
}

pub struct Queue {
    device: Arc<Device>,
    tracker: Mutex<ResourceStateTracker>,
    command_buffer: CommandBuffer,
}

impl Queue {
    pub fn submit(&self, encoder: CommandEncoder) {
        let mut tracker = self.tracker.lock().unwrap();
        for command in encoder.commands {
            match command {
                Command::Draw { vertices, instances } => {
                    self.command_buffer.draw(vertices, instances);
                }
                Command::SetVertexBuffer { buffer, offset } => {
                    // self.command_buffer.set_vertex_buffer(buffer, offset);
                }
                Command::SetIndexBuffer { buffer, offset } => {
                    // self.command_buffer.set_index_buffer(buffer, offset);
                }
                Command::UploadBuffer { buffer, data } => {
                    if let Some(buffer) = tracker.buffers.get_mut(&buffer) {
                    }
                }
            }
        }
    }
}

pub enum BufferUsage {
    Vertex,
    Index,
    HostAcces,
}
