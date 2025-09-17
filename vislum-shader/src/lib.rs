pub mod composer;
pub mod directive;

pub mod prelude {
    pub use crate::composer::{ComposeError, ComposeErrorType, ShaderComposer};
    pub use crate::directive::collect_includes;
}