use std::sync::Arc;

use crate::{
    fs::{File, Fs, ReadError},
    path::{AssetNamespace, AssetUri},
};

/// A virtual filesystem.
#[derive(Default, Clone)]
pub struct VirtualFs {
    /// The filesystem for the "vislum://" namespace, storing
    /// engine-backed assets.
    pub vislum: Option<Arc<dyn Fs>>,
    
    /// The filesystem for the "project://" namespace, storing
    /// project-backed assets.
    pub project: Option<Arc<dyn Fs>>,
}

static_assertions::assert_impl_all!(VirtualFs: Send, Sync);

impl Fs for VirtualFs {
    fn read(&self, path: &AssetUri) -> Result<File, ReadError> {
        let fs = match path.namespace() {
            AssetNamespace::Vislum => self.vislum.clone(),
            AssetNamespace::Project => self.project.clone(),
        };

        let fs = fs.ok_or(ReadError::NotFound)?;

        // Read the file from the backing filesystem.
        fs.read(path)
    }
}
