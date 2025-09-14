use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{Asset, AssetLoader, AssetPath, LoadContext, LoadError};

#[derive(Debug, Clone)]
pub struct ShaderAsset {
    pub source: String,
}

impl Asset for ShaderAsset {}

pub struct ShaderLoader;

impl AssetLoader for ShaderLoader {
    type Asset = ShaderAsset;

    fn extensions(&self) -> &'static [&'static str] {
        &["wgsl"]
    }

    fn load(
        &self,
        ctx: &mut LoadContext,
        path: AssetPath<'static>,
    ) -> Result<ShaderAsset, LoadError> {
        let mut ctx = ShaderLoaderContext {
            load_ctx: ctx,
            cycle_detector: CycleDetector::new(),
            parsed_result: String::new(),
            // cached_parsed_sources: HashMap::new(),
        };

        ctx.load_source(path)?;

        Ok(ShaderAsset {
            source: ctx.parsed_result,
        })
    }
}

#[derive(Default)]
struct CycleDetector {
    traversal_path: Vec<AssetPath<'static>>,
    traversal_seen: HashSet<AssetPath<'static>>,
}

impl CycleDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a path to the traversal path.
    pub fn push(&mut self, path: AssetPath<'static>) -> Result<(), LoadError> {
        if self.traversal_seen.contains(&path) {
            return Err(LoadError::DependencyCycle(self.traversal_path.clone()));
        }

        self.traversal_path.push(path.clone());
        self.traversal_seen.insert(path.clone());

        Ok(())
    }

    /// Pops a path from the traversal path.
    pub fn pop(&mut self) {
        match self.traversal_path.pop() {
            Some(path) => {
                self.traversal_seen.remove(&path);
            }
            None => {}
        }
    }
}

/// A context for loading shader assets.
struct ShaderLoaderContext<'a> {
    /// The context for loading assets.
    load_ctx: &'a mut LoadContext,

    /// Used to track cycles when loading shader sources.
    cycle_detector: CycleDetector,

    /// The result of the parsed shader source.
    parsed_result: String,
    // All cached parsed shader sources.
    // cached_parsed_sources: HashMap<AssetPath, String>,
}

impl<'a> ShaderLoaderContext<'a> {
    fn load_source(&mut self, path: AssetPath<'static>) -> Result<(), LoadError> {
        // Check for cycles.
        self.cycle_detector.push(path.clone())?;

        // Check if the source is already cached.
        // if let Some(source) = self.cached_parsed_sources.get(&path) {
        //     self.parsed_result.push_str(&*source);
        //     return Ok(());
        // }

        // Load the source.
        let bytes = self.load_ctx.read_file(path.clone())?;
        let source = String::from_utf8_lossy(&*bytes);

        for line in source.lines() {
            if line.starts_with("#include") {
                fn invalid_shader_source() -> LoadError {
                    LoadError::Custom("Invalid shader source".to_string())
                }

                let include_path = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.strip_prefix("\""))
                    .and_then(|s| s.strip_suffix("\""))
                    .map(|s| s.trim())
                    .ok_or_else(invalid_shader_source)?;

                let include_path = AssetPath::new_owned(include_path);

                self.load_source(include_path)?;
                if !self.parsed_result.ends_with('\n') {
                    self.parsed_result.push('\n');
                }
            } else {
                self.parsed_result.push_str(line);
                self.parsed_result.push('\n');
            }
        }

        self.cycle_detector.pop();

        Ok(())
    }
}
