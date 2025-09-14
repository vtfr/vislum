use std::{collections::HashMap, path::PathBuf};

use crate::{fs::{Bytes, Fs, ReadError}, path::AssetPath};

/// A filesystem implementation that stores files that are embedded in the binary.
pub struct MemoryFs {
    files: HashMap<PathBuf, Bytes>,
}

impl MemoryFs {
    pub fn new() -> Self {
        Self { files: HashMap::new() }
    }
}

impl Fs for MemoryFs {
    fn read(&self, path: &AssetPath) -> Result<Bytes, ReadError> {
        match self.files.get(path.path()) {
            Some(file) => Ok(file.clone()),
            None => Err(ReadError::NotFound),
        }
    }
}
