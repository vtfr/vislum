pub mod compiler;
pub mod composer;
pub mod directive;

pub mod prelude {
    pub use crate::compiler::{
        CompilerError, DescriptorBinding, DescriptorSet, DescriptorType, EntryPoint, PushConstant,
        ReflectorError, ShaderCompiler, ShaderReflection, ShaderReflector, ShaderStage, ShaderType,
    };
    pub use crate::composer::{ComposeError, ComposeErrorType, ShaderComposer};
    pub use crate::directive::collect_includes;
}
