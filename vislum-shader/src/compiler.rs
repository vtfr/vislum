use spirv_cross2::{Compiler, Module, targets::None as SpirvCrossNone};
use std::fs;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Failed to create temporary file: {0}")]
    TempFileCreationFailed(std::io::Error),
    #[error("Failed to write shader source to temporary file: {0}")]
    TempFileWriteFailed(std::io::Error),
    #[error("DXC executable not found or failed to execute: {0}")]
    DxcExecutionFailed(String),
    #[error("Failed to read compiled SPIRV output: {0}")]
    OutputReadFailed(std::io::Error),
    #[error("DXC compilation failed with exit code {0}: {1}")]
    CompilationFailed(i32, String),
    #[error("Failed to create SPIRV reflection: {0}")]
    ReflectionFailed(String),
}

#[derive(Error, Debug)]
pub enum ReflectorError {
    #[error("Invalid SPIRV data: {0}")]
    InvalidSpirv(String),
    #[error("Failed to create SPIRV module: {0}")]
    ModuleCreationFailed(String),
    #[error("Failed to create compiler: {0}")]
    CompilerCreationFailed(String),
    #[error("Failed to enumerate entry points: {0}")]
    EntryPointEnumerationFailed(String),
    #[error("Failed to get shader resources: {0}")]
    ResourceEnumerationFailed(String),
    #[error("Failed to get decoration: {0}")]
    DecorationFailed(String),
}

impl From<ReflectorError> for CompilerError {
    fn from(err: ReflectorError) -> Self {
        CompilerError::ReflectionFailed(err.to_string())
    }
}

pub struct ShaderCompiler {
    dxc_path: String,
}

pub struct ShaderReflector;

impl ShaderReflector {
    pub fn new() -> Self {
        Self
    }

    pub fn reflect_spirv(
        &self,
        spirv_bytes: &[u8],
        shader_type: ShaderType,
    ) -> Result<ShaderReflection, ReflectorError> {
        let shader_stage: ShaderStage = shader_type.into();

        // Validate SPIRV data
        if spirv_bytes.len() < 4 {
            return Err(ReflectorError::InvalidSpirv(
                "SPIRV data too short".to_string(),
            ));
        }

        // Check SPIRV magic number
        let magic = u32::from_le_bytes([
            spirv_bytes[0],
            spirv_bytes[1],
            spirv_bytes[2],
            spirv_bytes[3],
        ]);
        if magic != 0x07230203 {
            return Err(ReflectorError::InvalidSpirv(format!(
                "Invalid SPIRV magic number: 0x{:08X}",
                magic
            )));
        }

        // Convert bytes to words for spirv-cross2
        let words: Vec<u32> = spirv_bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        // Create SPIRV module
        let module = Module::from_words(&words);
        let compiler = Compiler::<SpirvCrossNone>::new(module)
            .map_err(|e| ReflectorError::CompilerCreationFailed(e.to_string()))?;

        // Extract entry points
        let entry_points = compiler
            .entry_points()
            .map_err(|e| ReflectorError::EntryPointEnumerationFailed(e.to_string()))?
            .into_iter()
            .map(|ep| EntryPoint {
                name: ep.name.to_string(),
                stage: shader_stage,
            })
            .collect();

        // Extract shader resources
        let resources = compiler
            .shader_resources()
            .map_err(|e| ReflectorError::ResourceEnumerationFailed(e.to_string()))?;

        // Extract descriptor sets
        let mut descriptor_sets = Vec::new();
        let mut sets: std::collections::HashMap<u32, Vec<DescriptorBinding>> =
            std::collections::HashMap::new();

        // Process sampled images
        if let Ok(sampled_images) =
            resources.resources_for_type(spirv_cross2::reflect::ResourceType::SampledImage)
        {
            for resource in sampled_images {
                let set = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::DescriptorSet)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(s) => Some(s),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed(
                            "Descriptor set decoration not found".to_string(),
                        )
                    })?;

                let binding = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::Binding)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(b) => Some(b),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed("Binding decoration not found".to_string())
                    })?;

                let desc_binding = DescriptorBinding {
                    binding,
                    name: resource.name.to_string(),
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    count: 1,
                    stage_flags: 0,
                };
                sets.entry(set).or_insert_with(Vec::new).push(desc_binding);
            }
        }

        // Process storage images
        if let Ok(storage_images) =
            resources.resources_for_type(spirv_cross2::reflect::ResourceType::StorageImage)
        {
            for resource in storage_images {
                let set = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::DescriptorSet)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(s) => Some(s),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed(
                            "Descriptor set decoration not found".to_string(),
                        )
                    })?;

                let binding = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::Binding)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(b) => Some(b),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed("Binding decoration not found".to_string())
                    })?;

                let desc_binding = DescriptorBinding {
                    binding,
                    name: resource.name.to_string(),
                    descriptor_type: DescriptorType::StorageImage,
                    count: 1,
                    stage_flags: 0,
                };
                sets.entry(set).or_insert_with(Vec::new).push(desc_binding);
            }
        }

        // Process uniform buffers
        if let Ok(uniform_buffers) =
            resources.resources_for_type(spirv_cross2::reflect::ResourceType::UniformBuffer)
        {
            for resource in uniform_buffers {
                let set = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::DescriptorSet)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(s) => Some(s),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed(
                            "Descriptor set decoration not found".to_string(),
                        )
                    })?;

                let binding = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::Binding)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(b) => Some(b),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed("Binding decoration not found".to_string())
                    })?;

                let desc_binding = DescriptorBinding {
                    binding,
                    name: resource.name.to_string(),
                    descriptor_type: DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: 0,
                };
                sets.entry(set).or_insert_with(Vec::new).push(desc_binding);
            }
        }

        // Process storage buffers
        if let Ok(storage_buffers) =
            resources.resources_for_type(spirv_cross2::reflect::ResourceType::StorageBuffer)
        {
            for resource in storage_buffers {
                let set = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::DescriptorSet)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(s) => Some(s),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed(
                            "Descriptor set decoration not found".to_string(),
                        )
                    })?;

                let binding = compiler
                    .decoration(resource.id, spirv_cross2::spirv::Decoration::Binding)
                    .map_err(|e| ReflectorError::DecorationFailed(e.to_string()))?
                    .and_then(|d| match d {
                        spirv_cross2::reflect::DecorationValue::Literal(b) => Some(b),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ReflectorError::DecorationFailed("Binding decoration not found".to_string())
                    })?;

                let desc_binding = DescriptorBinding {
                    binding,
                    name: resource.name.to_string(),
                    descriptor_type: DescriptorType::StorageBuffer,
                    count: 1,
                    stage_flags: 0,
                };
                sets.entry(set).or_insert_with(Vec::new).push(desc_binding);
            }
        }

        // Convert sets to descriptor sets
        for (set, bindings) in sets {
            descriptor_sets.push(DescriptorSet { set, bindings });
        }

        // Extract push constants
        let mut push_constants = Vec::new();
        if let Ok(push_constant_buffers) =
            resources.resources_for_type(spirv_cross2::reflect::ResourceType::PushConstant)
        {
            for resource in push_constant_buffers {
                push_constants.push(PushConstant {
                    name: resource.name.to_string(),
                    offset: 0, // spirv-cross2 doesn't provide offset directly
                    size: 0,   // spirv-cross2 doesn't provide size directly
                    stage_flags: 0,
                });
            }
        }

        Ok(ShaderReflection {
            entry_points,
            descriptor_sets,
            push_constants,
            shader_stage,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ShaderReflection {
    pub entry_points: Vec<EntryPoint>,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub push_constants: Vec<PushConstant>,
    pub shader_stage: ShaderStage,
}

#[derive(Debug, Clone)]
pub struct EntryPoint {
    pub name: String,
    pub stage: ShaderStage,
}

#[derive(Debug, Clone)]
pub struct DescriptorSet {
    pub set: u32,
    pub bindings: Vec<DescriptorBinding>,
}

#[derive(Debug, Clone)]
pub struct DescriptorBinding {
    pub binding: u32,
    pub name: String,
    pub descriptor_type: DescriptorType,
    pub count: u32,
    pub stage_flags: u32,
}

#[derive(Debug, Clone)]
pub struct PushConstant {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub stage_flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorType {
    Sampler,
    CombinedImageSampler,
    SampledImage,
    StorageImage,
    UniformTexelBuffer,
    StorageTexelBuffer,
    UniformBuffer,
    StorageBuffer,
    UniformBufferDynamic,
    StorageBufferDynamic,
    InputAttachment,
    AccelerationStructure,
    Unknown,
}

impl ShaderCompiler {
    pub fn new() -> Result<Self, CompilerError> {
        // Try to find DXC in common locations
        let dxc_paths = [
            "dxc",
            "dxc.exe",
            "/usr/bin/dxc",
            "/usr/local/bin/dxc",
            "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\x64\\dxc.exe",
            "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\x86\\dxc.exe",
        ];

        for path in &dxc_paths {
            if Command::new(path).arg("--version").output().is_ok() {
                return Ok(Self {
                    dxc_path: path.to_string(),
                });
            }
        }

        Err(CompilerError::DxcExecutionFailed(
            "DXC executable not found. Please ensure DXC is installed and in your PATH."
                .to_string(),
        ))
    }

    pub fn compile(
        &self,
        shader_source: &str,
        entry_point: &str,
        shader_type: ShaderType,
    ) -> Result<Vec<u8>, CompilerError> {
        // Create a temporary file for the shader source
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!("shader_{}.hlsl", uuid::Uuid::new_v4()));
        let temp_output_file_path =
            temp_dir.join(format!("{}.spv", temp_file_path.to_string_lossy()));

        fs::write(&temp_file_path, shader_source).map_err(CompilerError::TempFileWriteFailed)?;

        // DXC
        let output = Command::new(&self.dxc_path)
            .arg(&temp_file_path)
            .arg("-E")
            .arg(entry_point)
            .arg("-T")
            .arg(shader_type.target_profile())
            .arg("-Fo")
            .arg(&temp_output_file_path)
            .arg("-spirv")
            .arg("-fspv-target-env=vulkan1.3")
            .output()
            .map_err(|e| {
                CompilerError::DxcExecutionFailed(format!("Failed to execute DXC: {}", e))
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(CompilerError::CompilationFailed(
                output.status.code().unwrap_or(-1),
                error_msg.to_string(),
            ));
        }

        // Read the compiled SPIRV output
        let spirv_bytes =
            fs::read(&temp_output_file_path).map_err(CompilerError::OutputReadFailed)?;

        // Clean up temporary files
        let _ = fs::remove_file(&temp_file_path);
        let _ = fs::remove_file(&temp_output_file_path);

        Ok(spirv_bytes)
    }

    pub fn compile_vertex(
        &self,
        shader_source: &str,
        entry_point: &str,
    ) -> Result<Vec<u8>, CompilerError> {
        self.compile(shader_source, entry_point, ShaderType::Vertex)
    }

    pub fn compile_fragment(
        &self,
        shader_source: &str,
        entry_point: &str,
    ) -> Result<Vec<u8>, CompilerError> {
        self.compile(shader_source, entry_point, ShaderType::Fragment)
    }

    pub fn compile_compute(
        &self,
        shader_source: &str,
        entry_point: &str,
    ) -> Result<Vec<u8>, CompilerError> {
        self.compile(shader_source, entry_point, ShaderType::Compute)
    }

    pub fn compile_with_reflection(
        &self,
        shader_source: &str,
        entry_point: &str,
        shader_type: ShaderType,
    ) -> Result<(Vec<u8>, ShaderReflection), CompilerError> {
        let spirv_bytes = self.compile(shader_source, entry_point, shader_type)?;
        let reflector = ShaderReflector::new();
        let reflection = reflector.reflect_spirv(&spirv_bytes, shader_type)?;
        Ok((spirv_bytes, reflection))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

impl ShaderType {
    fn target_profile(self) -> &'static str {
        match self {
            ShaderType::Vertex => "vs_6_0",
            ShaderType::Fragment => "ps_6_0",
            ShaderType::Compute => "cs_6_0",
        }
    }
}

impl Into<ShaderStage> for ShaderType {
    fn into(self) -> ShaderStage {
        match self {
            ShaderType::Vertex => ShaderStage::Vertex,
            ShaderType::Fragment => ShaderStage::Fragment,
            ShaderType::Compute => ShaderStage::Compute,
        }
    }
}
