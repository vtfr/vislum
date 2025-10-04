pub mod composer;
pub mod directive;
pub mod compiler;

pub mod prelude {
    pub use crate::composer::{ComposeError, ComposeErrorType, ShaderComposer};
    pub use crate::directive::collect_includes;
    pub use crate::compiler::{
        ShaderCompiler, ShaderReflector, CompilerError, ReflectorError, ShaderType, ShaderReflection, 
        EntryPoint, DescriptorSet, DescriptorBinding, PushConstant,
        ShaderStage, DescriptorType
    };
}