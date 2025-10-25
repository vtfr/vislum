// use std::ops::Deref;
// use std::sync::Arc;
// use std::{collections::HashMap, sync::RwLock};
// use std::hash::{Hash, Hasher};

// use vislum_dxc::{DxcCompilationError, DxcCompiler};

// pub struct PreHashed<T> {
//     hash: u64,
//     data: T,
// }

// impl<T> From<T> for PreHashed<T>
// where 
//     T: Hash,
// {
//     fn from(data: T) -> Self {
//         Self::new(data)
//     }
// }

// impl<T> PreHashed<T> 
// where 
//     T: Hash,
// {
//     pub fn new(data: T) -> Self {
//         let mut hasher = std::hash::DefaultHasher::new();
//         data.hash(&mut hasher);
//         let hash = hasher.finish();
//         Self { hash, data }
//     }
// }

// impl<T> Deref for PreHashed<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.data
//     }
// }

// impl<T> Hash for PreHashed<T>
// where 
//     T: Hash,
// {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         state.write_u64(self.hash);
//     }
// }

// /// A cache for shaders.
// /// 
// /// Shaders are identified by their source code and entry point.
// pub struct ShaderCache {
//     storage: RwLock<HashMap<PreHashed<String>, Arc<[u32]>>>,
//     compiler: DxcCompiler
// }

// static_assertions::assert_impl_all!(ShaderCache: Send, Sync);

// #[derive(thiserror::Error, Debug)]
// pub enum ShaderCompilationError {
//     #[error("compilation failed: {0}")]
//     CompilationError(DxcCompilationError),
// }

// impl ShaderCache {
//     pub fn compile(&self, shader: impl Into<PreHashed<String>>) -> Result<Arc<[u32]>, ShaderCompilationError> {
//         let shader = shader.into();
//         let compiled = self.compiler.compile(&shader.data, &DxcIncludeHandler::default())?;
//         Ok(Arc::new(compiled))
//     }
// }