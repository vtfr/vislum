use std::{borrow::Borrow, ops::Deref};

use atomicow::CowArc;

use crate::path::AssetPath;

pub mod editor;
pub mod memory;

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("The file was not found")]
    NotFound,
    #[error("The file is in an incompatible namespace")]
    IncompatibleNamespace,
}

/// A trait for filesystem operations.
pub trait Fs: Send + Sync {
    /// Reads a file from the filesystem.
    /// 
    /// Implementations must resolve the path to a valid file path.
    fn read(&self, path: &AssetPath) -> Result<Bytes, ReadError>;

    // /// Iterates over the files in the filesystem.
    // fn iter(&self) -> Box<dyn Iterator<Item = AssetPath> + Send + Sync>;
}

/// A wrapper around a byte array.
#[derive(Clone)]
pub struct Bytes(CowArc<'static, [u8]>);

impl Bytes {
    pub const fn new_static(bytes: &'static [u8]) -> Self {
        Self(CowArc::Static(bytes))
    }

    pub fn new_owned(bytes: Vec<u8>) -> Self {
        Self(CowArc::new_owned_from_arc(bytes))
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}