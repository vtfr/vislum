use std::borrow::Cow;
use naga::{Module, ShaderStage, Type, TypeInner};
use thiserror::Error;

/// A shader compiler and preprocessor.
/// 
/// Compiles shaders from source code to validated WGSL with introspection data.
pub struct ShaderPreprocessor {
    /// Naga module for shader validation and introspection
    module: Module,
}

bitflags::bitflags! {
    /// Flags supported for shader compilation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ShaderCompilationOption: u32 {
        /// Instancing support. Enables instanced batch rendering
        const INSTANCED = 1 << 0;
    }
}

pub struct CompilationRequest<'a> {
    pub source: Cow<'a, str>,
    pub options: ShaderCompilationOption,
}

/// Represents a shader type
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderType {
    // Scalars
    Float,
    Int,
    UInt,
    Bool,
    
    // Vectors
    Vec2F,
    Vec3F,
    Vec4F,
    Vec2I,
    Vec3I,
    Vec4I,
    Vec2U,
    Vec3U,
    Vec4U,
    
    // Matrices
    Mat2x2F,
    Mat3x3F,
    Mat4x4F,
    
    // Arrays
    Array { base: Box<ShaderType>, size: Option<u32> },
    
    // Textures
    Texture1D,
    Texture2D,
    
    // Structs (named types)
    Struct(String),
}

/// Represents a field within a shader data structure
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderDataField {
    /// The name of the field in WGSL
    pub name: String,
    /// The type of the field
    pub field_type: ShaderType,
    /// The offset of the field in bytes (if available from naga)
    pub offset: Option<u32>,
}

/// Represents a shader data structure (like uniform buffers, storage buffers)
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderDataStruct {
    /// The name of the struct in WGSL
    pub name: String,
    /// The fields within this struct
    pub fields: Vec<ShaderDataField>,
}

/// Represents a texture binding
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderTextureBinding {
    /// The binding index
    pub binding: u32,
    /// The name of the texture in WGSL
    pub name: String,
    /// The texture type (only 1D and 2D supported)
    pub texture_type: ShaderType,
}

/// Represents a shader entry point
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderEntryPoint {
    /// The name of the entry point function
    pub name: String,
    /// The shader stage (vertex, fragment, compute)
    pub stage: ShaderStage,
}

/// Contains introspection data about a compiled shader
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderIntrospection {
    /// The validated WGSL source
    pub validated_wgsl: String,
    /// Data structures found in the shader
    pub data_structures: Vec<ShaderDataStruct>,
    /// Texture bindings found in the shader
    pub texture_bindings: Vec<ShaderTextureBinding>,
    /// Entry points found in the shader
    pub entry_points: Vec<ShaderEntryPoint>,
    /// All bindings (for compatibility)
    pub bindings: Vec<ShaderBinding>,
}

/// Represents a generic shader binding
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderBinding {
    /// The binding index
    pub binding: u32,
    /// The name of the binding in WGSL
    pub name: String,
    /// The type of binding (uniform, storage, texture, sampler)
    pub binding_type: String,
    /// The visibility (vertex, fragment, compute)
    pub visibility: Vec<ShaderStage>,
}

/// Errors that can occur during shader compilation
#[derive(Error, Debug)]
pub enum ShaderCompilationError {
    #[error("WGSL parsing error: {0}")]
    WgslParseError(String),
    #[error("Naga validation error: {0}")]
    ValidationError(String),
    #[error("Introspection error: {0}")]
    IntrospectionError(String),
}

impl ShaderPreprocessor {
    pub fn new() -> Self {
        Self {
            module: Module::default(),
        }
    }

    /// Compiles and introspects a shader from source
    pub fn compile(
        &mut self,
        request: CompilationRequest,
    ) -> Result<ShaderIntrospection, ShaderCompilationError> {
        // Preprocess the shader source based on compilation options
        let preprocessed_source = self.preprocess_shader(&request.source, request.options)?;
        
        // Parse WGSL source using naga
        let module = naga::front::wgsl::parse_str(&preprocessed_source)
            .map_err(|e| ShaderCompilationError::WgslParseError(format!("{:?}", e)))?;
        
        // Perform validation
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator.validate(&module)
            .map_err(|e| ShaderCompilationError::ValidationError(e.to_string()))?;
        
        // Extract introspection data
        let introspection = self.introspect_module(&module, &preprocessed_source)?;
        
        Ok(introspection)
    }

    /// Preprocess shader source based on compilation options
    fn preprocess_shader(
        &self,
        source: &str,
        _options: ShaderCompilationOption,
    ) -> Result<String, ShaderCompilationError> {
        // For now, just return the original source
        // The preprocessing will be handled by naga's built-in preprocessor during parsing
        // when we pass the defines through the parse_str function
        Ok(source.to_string())
    }

    /// Extract introspection data from the validated module
    fn introspect_module(&self, module: &Module, original_source: &str) -> Result<ShaderIntrospection, ShaderCompilationError> {
        let mut data_structures = Vec::new();
        let mut texture_bindings = Vec::new();
        let mut entry_points = Vec::new();
        let mut bindings = Vec::new();

        // Extract data structures from types
        for (_type_handle, type_info) in module.types.iter() {
            if let Type { name: Some(name), inner: TypeInner::Struct { members, .. }, .. } = type_info {
                let mut fields = Vec::new();
                
                for member in members {
                    let field_name = member.name.clone().unwrap_or_else(|| "unnamed".to_string());
                    let field_type = self.convert_type(&module.types, member.ty);
                    let offset = member.offset;
                    
                    fields.push(ShaderDataField {
                        name: field_name,
                        field_type,
                        offset: Some(offset),
                    });
                }
                
                data_structures.push(ShaderDataStruct {
                    name: name.clone(),
                    fields,
                });
            }
        }

        // Extract bindings from global variables
        for (_var_handle, var_info) in module.global_variables.iter() {
            if let Some(binding) = var_info.binding {
                let name = var_info.name.clone().unwrap_or_else(|| "unnamed".to_string());
                let var_type = &module.types[var_info.ty];
                
                let binding_type = match var_info.space {
                    naga::AddressSpace::Uniform => "uniform",
                    naga::AddressSpace::Storage { .. } => "storage",
                    naga::AddressSpace::Handle => {
                        // Check if it's a texture or sampler
                        if let TypeInner::Image { .. } = var_type.inner {
                            if let Some(texture_type) = self.convert_image_type(var_type) {
                                texture_bindings.push(ShaderTextureBinding {
                                    binding: binding.binding,
                                    name: name.clone(),
                                    texture_type,
                                });
                            }
                            "texture"
                        } else {
                            "sampler"
                        }
                    }
                    _ => "unknown",
                };

                if binding_type != "texture" {
                    bindings.push(ShaderBinding {
                        binding: binding.binding,
                        name,
                        binding_type: binding_type.to_string(),
                        visibility: vec![ShaderStage::Vertex, ShaderStage::Fragment], // Default visibility
                    });
                }
            }
        }

        // Extract entry points
        for entry_point in &module.entry_points {
            entry_points.push(ShaderEntryPoint {
                name: entry_point.name.clone(),
                stage: entry_point.stage,
            });
        }

        Ok(ShaderIntrospection {
            validated_wgsl: original_source.to_string(),
            data_structures,
            texture_bindings,
            entry_points,
            bindings,
        })
    }

    /// Convert a naga type to a ShaderType
    fn convert_type(&self, types: &naga::UniqueArena<naga::Type>, type_handle: naga::Handle<naga::Type>) -> ShaderType {
        let type_info = &types[type_handle];
        match &type_info.inner {
            TypeInner::Scalar(scalar) => match scalar.kind {
                naga::ScalarKind::Float => ShaderType::Float,
                naga::ScalarKind::Sint => ShaderType::Int,
                naga::ScalarKind::Uint => ShaderType::UInt,
                naga::ScalarKind::Bool => ShaderType::Bool,
                naga::ScalarKind::AbstractFloat => ShaderType::Float,
                naga::ScalarKind::AbstractInt => ShaderType::Int,
            },
            TypeInner::Vector { size, scalar } => {
                match (size, scalar.kind) {
                    (naga::VectorSize::Bi, naga::ScalarKind::Float) => ShaderType::Vec2F,
                    (naga::VectorSize::Tri, naga::ScalarKind::Float) => ShaderType::Vec3F,
                    (naga::VectorSize::Quad, naga::ScalarKind::Float) => ShaderType::Vec4F,
                    (naga::VectorSize::Bi, naga::ScalarKind::Sint) => ShaderType::Vec2I,
                    (naga::VectorSize::Tri, naga::ScalarKind::Sint) => ShaderType::Vec3I,
                    (naga::VectorSize::Quad, naga::ScalarKind::Sint) => ShaderType::Vec4I,
                    (naga::VectorSize::Bi, naga::ScalarKind::Uint) => ShaderType::Vec2U,
                    (naga::VectorSize::Tri, naga::ScalarKind::Uint) => ShaderType::Vec3U,
                    (naga::VectorSize::Quad, naga::ScalarKind::Uint) => ShaderType::Vec4U,
                    _ => ShaderType::Vec4F, // Default fallback
                }
            }
            TypeInner::Matrix { columns, rows, scalar: _ } => {
                match (*columns, *rows) {
                    (naga::VectorSize::Bi, naga::VectorSize::Bi) => ShaderType::Mat2x2F,
                    (naga::VectorSize::Tri, naga::VectorSize::Tri) => ShaderType::Mat3x3F,
                    (naga::VectorSize::Quad, naga::VectorSize::Quad) => ShaderType::Mat4x4F,
                    _ => ShaderType::Mat4x4F, // Default fallback
                }
            }
            TypeInner::Array { base, size, stride: _ } => {
                let base_type = self.convert_type(types, *base);
                let array_size = match size {
                    naga::ArraySize::Constant(size) => Some(size.get()),
                    naga::ArraySize::Dynamic => None,
                    naga::ArraySize::Pending(_) => None,
                };
                ShaderType::Array { base: Box::new(base_type), size: array_size }
            }
            TypeInner::Struct { .. } => {
                ShaderType::Struct(type_info.name.clone().unwrap_or_else(|| "struct".to_string()))
            }
            _ => ShaderType::Float, // Default fallback
        }
    }

    /// Convert image type to ShaderType (only D1 and D2 supported, float only)
    fn convert_image_type(&self, type_info: &Type) -> Option<ShaderType> {
        if let TypeInner::Image { dim, arrayed: _, class } = &type_info.inner {
            // Only support float sample types
            let is_float = match class {
                naga::ImageClass::Sampled { kind, .. } => matches!(kind, naga::ScalarKind::Float | naga::ScalarKind::AbstractFloat),
                naga::ImageClass::Depth { .. } => true, // Depth textures are always float
                naga::ImageClass::Storage { .. } => false, // No storage textures for now
            };
            
            if !is_float {
                return None; // Skip non-float textures
            }
            
            match dim {
                naga::ImageDimension::D1 => Some(ShaderType::Texture1D),
                naga::ImageDimension::D2 => Some(ShaderType::Texture2D),
                _ => None, // Only support D1 and D2
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_vertex_shader() {
        let shader_source = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}
"#;

        let mut preprocessor = ShaderPreprocessor::new();
        let request = CompilationRequest {
            source: std::borrow::Cow::Borrowed(shader_source),
            options: ShaderCompilationOption::empty(),
        };

        let result = preprocessor.compile(request);
        assert!(result.is_ok());

        let introspection = result.unwrap();
        assert_eq!(introspection.entry_points.len(), 1);
        assert_eq!(introspection.entry_points[0].name, "vs_main");
        assert_eq!(introspection.entry_points[0].stage, ShaderStage::Vertex);
    }

    #[test]
    fn test_shader_with_uniforms() {
        let shader_source = r#"
struct Uniforms {
    mvp: mat4x4<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}
"#;

        let mut preprocessor = ShaderPreprocessor::new();
        let request = CompilationRequest {
            source: std::borrow::Cow::Borrowed(shader_source),
            options: ShaderCompilationOption::empty(),
        };

        let result = preprocessor.compile(request);
        assert!(result.is_ok());

        let introspection = result.unwrap();
        assert_eq!(introspection.data_structures.len(), 1);
        assert_eq!(introspection.data_structures[0].name, "Uniforms");
        assert_eq!(introspection.data_structures[0].fields.len(), 2);
        
        // Check field types
        assert_eq!(introspection.data_structures[0].fields[0].name, "mvp");
        assert_eq!(introspection.data_structures[0].fields[0].field_type, ShaderType::Mat4x4F);
        assert_eq!(introspection.data_structures[0].fields[1].name, "time");
        assert_eq!(introspection.data_structures[0].fields[1].field_type, ShaderType::Float);
    }

    #[test]
    fn test_shader_with_textures() {
        let shader_source = r#"
@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, uv);
}
"#;

        let mut preprocessor = ShaderPreprocessor::new();
        let request = CompilationRequest {
            source: std::borrow::Cow::Borrowed(shader_source),
            options: ShaderCompilationOption::empty(),
        };

        let result = preprocessor.compile(request);
        assert!(result.is_ok());

        let introspection = result.unwrap();
        assert_eq!(introspection.texture_bindings.len(), 1);
        assert_eq!(introspection.texture_bindings[0].name, "texture");
        assert_eq!(introspection.texture_bindings[0].binding, 0);
        assert_eq!(introspection.texture_bindings[0].texture_type, ShaderType::Texture2D);
    }

    #[test]
    fn test_invalid_shader() {
        let shader_source = r#"
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return invalid_syntax_here
}
"#;

        let mut preprocessor = ShaderPreprocessor::new();
        let request = CompilationRequest {
            source: std::borrow::Cow::Borrowed(shader_source),
            options: ShaderCompilationOption::empty(),
        };

        let result = preprocessor.compile(request);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ShaderCompilationError::WgslParseError(_) => {
                // Expected error type
            }
            _ => panic!("Expected WgslParseError"),
        }
    }

    #[test]
    fn test_shader_type_conversion() {
        // Test various type conversions
        let shader_source = r#"
struct TestTypes {
    scalar_float: f32,
    scalar_int: i32,
    scalar_uint: u32,
    scalar_bool: bool,
    vec2_float: vec2<f32>,
    vec3_int: vec3<i32>,
    vec4_uint: vec4<u32>,
    mat2_float: mat2x2<f32>,
    mat3_float: mat3x3<f32>,
    mat4_float: mat4x4<f32>,
}

@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let mut preprocessor = ShaderPreprocessor::new();
        let request = CompilationRequest {
            source: std::borrow::Cow::Borrowed(shader_source),
            options: ShaderCompilationOption::empty(),
        };

        let result = preprocessor.compile(request);
        assert!(result.is_ok());

        let introspection = result.unwrap();
        assert_eq!(introspection.data_structures.len(), 1);
        
        let fields = &introspection.data_structures[0].fields;
        assert_eq!(fields[0].field_type, ShaderType::Float);
        assert_eq!(fields[1].field_type, ShaderType::Int);
        assert_eq!(fields[2].field_type, ShaderType::UInt);
        assert_eq!(fields[3].field_type, ShaderType::Bool);
        assert_eq!(fields[4].field_type, ShaderType::Vec2F);
        assert_eq!(fields[5].field_type, ShaderType::Vec3I);
        assert_eq!(fields[6].field_type, ShaderType::Vec4U);
        assert_eq!(fields[7].field_type, ShaderType::Mat2x2F);
        assert_eq!(fields[8].field_type, ShaderType::Mat3x3F);
        assert_eq!(fields[9].field_type, ShaderType::Mat4x4F);
    }
}



