pub struct Shader {
    /// The SPIR-V code for the shader.
    spirv: Vec<u32>,
    
    /// The hash of the SPIR-V code.
    spirv_hash: u64,

    /// The source file for the shader.
    /// 
    /// Used for tracking the origin of the shader, and performing
    /// pipeline warmup.
    source: Option<AssetPath>,

    /// The shader module.
    shader_module: ash::vk::ShaderModule,
}

impl Shader {
    pub fn new(device: RenderDevice) -> Self {
        Self { device }
    }
}

pub struct Material {
    /// The shader program.
    shader: Arc<Shader>,

    /// The material properties.
    properties: HashMap<String, Property>,
}
