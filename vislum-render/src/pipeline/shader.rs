use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, PoisonError, RwLock},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use vislum_dxc::{DxcCompiler, DxcIncludeHandler};

/// The source of a shader.
pub struct ShaderSource {
    source: String,
    hash: u64,
}

impl ShaderSource {
    pub fn new(source: String) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        source.hash(&mut hasher);
        let hash = hasher.finish();

        Self { source, hash }
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    pub fn source(&self) -> &str {
        &self.source
    }
}

impl From<String> for ShaderSource {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

impl Serialize for ShaderSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.source)
    }
}

impl<'a> Deserialize<'a> for ShaderSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let source = String::deserialize(deserializer)?;
        Ok(Self::new(source))
    }
}

impl Clone for ShaderSource {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            hash: self.hash,
        }
    }
}

impl Hash for ShaderSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl std::fmt::Debug for ShaderSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderSource")
            .field("source", &self.source)
            .field("hash", &self.hash)
            .finish()
    }
}

impl PartialEq for ShaderSource {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for ShaderSource {}

/// A cache for shaders.
pub struct ShaderCache {
    dxc: DxcCompiler,

    /// Caches the compiled shaders.
    storage: RwLock<HashMap<ShaderSource, Arc<[u32]>>>,
}

static_assertions::assert_impl_all!(ShaderCache: Send, Sync);

impl ShaderCache {
    pub fn new(dxc: DxcCompiler) -> Self {
        Self {
            dxc,
            storage: Default::default(),
        }
    }

    /// Compiles a shader to SPIR-V.
    ///
    /// If the shader is already cached, the cached version is returned.
    pub fn compile(&self, source: impl Into<ShaderSource>) -> Result<Arc<[u32]>, ()> {
        let source = source.into();

        let cached = self
            .storage
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&source)
            .cloned();

        if let Some(compiled) = cached {
            return Ok(compiled);
        }

        struct DefaultIncludeHandler;

        impl DxcIncludeHandler for DefaultIncludeHandler {
            fn load_source(&self, _filename: &str) -> Option<String> {
                None
            }
        }

        let compiled = self
            .dxc
            .compile(&source.source(), &DefaultIncludeHandler)
            .unwrap();

        // TODO: move chunking to DXC.
        let compiled = compiled
            .chunks(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Arc<[u32]>>();

        // Write the compiled shader to the cache.
        self.storage
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(source, compiled.clone());

        Ok(compiled)
    }
}
