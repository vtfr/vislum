use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{Fs, FsError, FsIterator};

pub struct PhysicalFs {
    root: PathBuf,
}

impl PhysicalFs {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Fs for PhysicalFs {
    fn read(&self, path: &Path) -> Result<Arc<[u8]>, FsError> {
        let path = self.root.join(path);

        match std::fs::read(&path) {
            Ok(data) => Ok(Arc::from(&*data)),
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => Err(FsError::NotFound(path)),
                _ => Err(FsError::PhysicalIo(path, error)),
            },
        }
    }

    fn write(&self, path: &Path, data: &[u8]) -> Result<(), FsError> {
        let path = self.root.join(path);
        match std::fs::write(&path, data) {
            Ok(_) => Ok(()),
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => Err(FsError::NotFound(path)),
                _ => Err(FsError::PhysicalIo(path, error)),
            },
        }
    }

    fn list(&self, path: Option<&Path>) -> FsIterator {
        let path = match path {
            Some(path) => Cow::Owned(self.root.join(path)),
            None => Cow::Borrowed(&self.root),
        };

        match std::fs::read_dir(&*path) {
            Ok(entries) => {
                let root = self.root.clone();
                let entries = entries.into_iter().filter_map(move |entry| {
                    let entry = entry.ok()?;

                    // Strip the root prefix from the entry path.
                    let path = entry.path().strip_prefix(&root).ok()?.to_path_buf();

                    Some(path)
                });

                Box::new(entries)
            }
            Err(_) => Box::new(std::iter::empty()),
        }
    }
}


