use std::borrow::Cow;

use vislum_system::System;

use crate::{resource::{Handle, IntoResourceId, RenderResourceStorage}, types::RenderDevice};

/// A system for managing shader modules.
#[derive(System)]
pub struct ShaderModuleManager {
    device: RenderDevice,
    modules: RenderResourceStorage<ShaderModule>,
}

pub struct ShaderModuleDescriptor<'a> {
    /// The source code of the shader module.
    pub source: Cow<'a, str>,

    /// The entry point of the shader module.
    pub entry_point: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub enum ShaderModuleBinding {
    Float(String, f32)
}

pub struct ShaderModule {
    module: wgpu::ShaderModule,
    entry_point: String,
}

impl ShaderModule {
    /// Gets the inner wgpu shader module.
    pub fn get_inner(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    /// Gets the entry point of the shader module.
    pub fn get_entry_point(&self) -> &str {
        &self.entry_point
    }
}

impl ShaderModuleManager {
    /// Creates a new shader module system.
    pub fn new(device: RenderDevice) -> Self {
        Self { device, modules: RenderResourceStorage::new() }
    }

    /// Creates a new shader module.
    pub fn create(&mut self, descriptor: ShaderModuleDescriptor) -> Handle<ShaderModule> {
        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(descriptor.source),
        });

        let shader_module = ShaderModule {
            module,
            entry_point: descriptor.entry_point.into_owned(),
        };

        self.modules.insert(shader_module)
    }

    /// Gets a shader module by id.
    pub fn get(&self, id: impl IntoResourceId<ShaderModule>) -> Option<&ShaderModule> {
        let id = id.into_resource_id();
        self.modules.get(id)
    }
}