use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::directive::{Directive, DirectiveParseError};

type IncludeId = usize;

struct IncludeSource {
    /// A unique index for tracking the source.
    /// 
    /// This is used to prevent copies of the source string when
    /// adding it to the include stack.
    index: IncludeId,
    
    /// The source string.
    source: String,
}
pub struct ShaderComposer {
    define_identifiers: HashSet<String>,
    include_sources: HashMap<String, IncludeSource>,
}

/// I'll think later about how to handle errors.
#[derive(Debug, Error)]
pub enum ComposeError {
    #[error("directive parse error: {0}")]
    DirectiveParseError(#[from] DirectiveParseError),

    #[error("unmatched #ifdef directive")]
    UnmatchedIfDefError,

    #[error("cyclic reference in path: {0:?}")]
    CyclicReference(Vec<String>),

    #[error("include source not found: {0}")]
    IncludeSourceNotFound(String),
}

impl ShaderComposer {
    pub fn new() -> Self {
        Self {
            define_identifiers: Default::default(),
            include_sources: Default::default(),
        }
    }

    /// Adds a define identifier to the composer.
    pub fn add_define_identifier(&mut self, identifier: String) {
        self.define_identifiers.insert(identifier);
    }

    /// Adds an import source to the composer.
    pub fn add_import_source(&mut self, path: String, source: String) {
        self.include_sources.insert(path, IncludeSource {
            index: self.include_sources.len(),
            source,
        });
    }

    /// Composes the shader source into a single string, 
    /// consuming the [`ShaderComposer`].
    pub fn compose(
        &self, 
        shader_source: &str,
    ) -> Result<String, ComposeError> {
        let mut output = String::with_capacity(shader_source.len());
        let mut directive_frame_stack = DirectiveFrameStack::default();
        let mut include_stack = IncludeStack::default();
        let mut source_stack = vec![shader_source.lines()];
        
        'outer: while let Some(source) = source_stack.last_mut() {
            while let Some(line) = source.next() {
                let directive = Directive::parse(line)?;

                match directive {
                    Directive::Ifdef(identifier) => {
                        let defined = self.define_identifiers.contains(identifier);
                        directive_frame_stack.push(defined);
                    }
                    Directive::Else => {
                        directive_frame_stack.branch_if_active_and_not_taken()
                            .map_err(|_| ComposeError::UnmatchedIfDefError)?;
                    }
                    Directive::Endif => {
                        directive_frame_stack.pop()
                            .map_err(|_| ComposeError::UnmatchedIfDefError)?;
                    }
                    Directive::Include(include_path) if directive_frame_stack.active() => {
                        let IncludeSource { index, source: include_source } = self.include_sources
                            .get(include_path)
                            .ok_or(ComposeError::IncludeSourceNotFound(include_path.into()))?;

                            if !include_stack.push(*index) {
                                let include_path_vec = include_stack.iter()
                                    .map(|id| self.include_sources
                                            .iter()
                                            .find_map(|(_, include_source)| {
                                                if include_source.index == id {
                                                    Some(include_source.source.clone())
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap()
                                    )
                                    .collect();

                                return Err(ComposeError::CyclicReference(include_path_vec)); // Circular include detected
                            }

                            source_stack.push(include_source.lines());
                            continue 'outer;
                    }
                    Directive::Raw(line) if directive_frame_stack.active() => {
                        output.push_str(line);
                        output.push('\n');
                    }
                    _ => {}
                };
            }

            // Pop the source.
            include_stack.pop();
            source_stack.pop();
        }

        if !directive_frame_stack.is_empty() {
            // If the counter is not 0, then we have unmatched `#ifdef` directives.
            return Err(ComposeError::UnmatchedIfDefError);
        }

        Ok(output)
    }
}

#[derive(Default)]
struct IncludeStack {
    stack: Vec<IncludeId>,
}

impl IncludeStack {
    pub fn push(&mut self, include_id: IncludeId) -> bool {
        if self.stack.contains(&include_id) {
            return false;
        }

        self.stack.push(include_id);
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = IncludeId> {
        self.stack.iter().copied()
    }

    pub fn pop(&mut self) { 
        // Pop the path from the stack.
        let _ = self.stack.pop();
    }
}

struct DirectiveFrame {
    /// Whether the current frame contains any branch that was taken.
    taken: bool,
    
    /// Whether the previous frame was active.
    parent_active: bool,
}

struct UnmatchedIfDefError;

#[derive(Default)]
struct DirectiveFrameStack {
    stack: Vec<DirectiveFrame>,
}

impl DirectiveFrameStack {
    /// Pushes a new frame.
    pub fn push(&mut self, branch_taken: bool) {
        let parent_active = self.stack.last()
            .map(|frame| frame.parent_active)
            .unwrap_or(true);

        self.stack.push(DirectiveFrame {
            taken: branch_taken,
            parent_active,
        });
    }

    /// Branches the current frame if it is active and hasn't been branched yet.
    pub fn branch_if_active_and_not_taken(&mut self) -> Result<(), UnmatchedIfDefError> {
        let last = self.stack.last_mut()
            .ok_or(UnmatchedIfDefError)?;

        if last.parent_active && !last.taken {
            last.taken = true;
        }

        Ok(())
    }

    /// Returns whether the current frame is active.
    pub fn active(&self) -> bool {
        self.stack.last()
            .map(|frame| frame.parent_active && frame.taken)
            .unwrap_or(true)
    }

    /// Pops the current frame.
    /// 
    /// Retruns [`UnmatchedIfDefError`] if the stack is empty.
    pub fn pop(&mut self) -> Result<(), UnmatchedIfDefError> {
        self.stack.pop()
            .ok_or(UnmatchedIfDefError)?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_composition() {
        let composer = ShaderComposer::new();
        let shader_source = r#"
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let result = composer.compose(shader_source);
        assert!(result.is_ok());
        
        let composed = result.unwrap();
        assert!(composed.contains("@vertex"));
        assert!(composed.contains("vs_main"));
    }

    #[test]
    fn test_ifdef_defined() {
        let mut composer = ShaderComposer::new();
        composer.add_define_identifier("DEBUG".to_string());
        
        let shader_source = r#"
#ifdef DEBUG
    let debug_value = 1.0;
#endif
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let result = composer.compose(shader_source);
        assert!(result.is_ok());
        
        let composed = result.unwrap();
        assert!(composed.contains("debug_value = 1.0"));
    }

    #[test]
    fn test_ifdef_not_defined() {
        let composer = ShaderComposer::new();
        // Don't add DEBUG define
        
        let shader_source = r#"
#ifdef DEBUG
    let debug_value = 1.0;
#endif
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let result = composer.compose(shader_source);
        assert!(result.is_ok());
        
        let composed = result.unwrap();
        assert!(!composed.contains("debug_value"));
        assert!(composed.contains("vs_main"));
    }

    #[test]
    fn test_nested_ifdef() {
        let mut composer = ShaderComposer::new();
        composer.add_define_identifier("INSTANCED".to_string());
        
        let shader_source = r#"
#ifdef INSTANCED
    #ifdef DEBUG
        let debug_instanced = 1.0;
    #endif
    let instanced_value = 1.0;
#endif
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let result = composer.compose(shader_source);
        assert!(result.is_ok());
        
        let composed = result.unwrap();
        assert!(composed.contains("instanced_value"));
        assert!(!composed.contains("debug_instanced")); // DEBUG not defined
        assert!(composed.contains("vs_main"));
    }

    #[test]
    fn test_include_basic() {
        let mut composer = ShaderComposer::new();
        
        let vertex_source = r#"
#include "common.wgsl"
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let common_source = r#"
struct VertexData {
    position: vec3<f32>,
}
"#;

        composer.add_import_source("common.wgsl".to_string(), common_source.to_string());

        let result = composer.compose(vertex_source);
        assert!(result.is_ok());
        
        let composed = result.unwrap();
        assert!(composed.contains("struct VertexData"));
        assert!(composed.contains("vs_main"));
    }

    #[test]
    fn test_include_with_defines() {
        let mut composer = ShaderComposer::new();
        composer.add_define_identifier("INSTANCED".to_string());
        
        let vertex_source = r#"
#include "instanced.wgsl"
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let instanced_source = r#"
#ifdef INSTANCED
struct InstanceData {
    transform: mat4x4<f32>,
}
#endif
"#;

        composer.add_import_source("instanced.wgsl".to_string(), instanced_source.to_string());

        let result = composer.compose(vertex_source);
        assert!(result.is_ok());
        
        let composed = result.unwrap();
        assert!(composed.contains("struct InstanceData"));
        assert!(composed.contains("vs_main"));
    }

    #[test]
    fn test_unmatched_ifdef() {
        let composer = ShaderComposer::new();
        
        let shader_source = r#"
#ifdef DEBUG
    let debug_value = 1.0;
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let result = composer.compose(shader_source);
        assert!(result.is_err()); // Should fail due to unmatched #ifdef
    }

    #[test]
    fn test_unmatched_endif() {
        let composer = ShaderComposer::new();
        
        let shader_source = r#"
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
#endif
"#;

        let result = composer.compose(shader_source);
        assert!(result.is_err()); // Should fail due to unmatched #endif
    }

    #[test]
    fn test_missing_include() {
        let composer = ShaderComposer::new();
        
        let shader_source = r#"
#include "missing.wgsl"
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

        let result = composer.compose(shader_source);
        assert!(result.is_err()); // Should fail due to missing include
    }

    #[test]
    fn test_circular_include() {
        let mut composer = ShaderComposer::new();
        
        let source_a = r#"
#include "b.wgsl"
struct A { value: f32; }
"#;

        let source_b = r#"
#include "a.wgsl"
struct B { value: f32; }
"#;

        composer.add_import_source("a.wgsl".to_string(), source_a.to_string());
        composer.add_import_source("b.wgsl".to_string(), source_b.to_string());

        let result = composer.compose(source_a);
        assert!(result.is_err()); // Should fail due to circular include
    }
}



