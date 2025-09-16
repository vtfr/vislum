use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use crate::directive::{is_else, is_endif, maybe_parse_ifdef, maybe_parse_include};

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
    include_stack: IncludeStack,
}

/// I'll think later about how to handle errors.
#[derive(Debug)]
pub struct Error;

impl ShaderComposer {
    pub fn new() -> Self {
        Self {
            define_identifiers: Default::default(),
            include_sources: Default::default(),
            include_stack: Default::default(),
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
    pub fn compose(self, shader_source: &str) -> Result<String, Error> {
        let mut output = String::with_capacity(shader_source.len());
        self.process_source(&mut output, shader_source)?;
        Ok(output)
    }

    /// Composes the shader source into a single string.
    ///
    /// This function will parse the shader source and compose it into a single string.
    /// It will also handle the include directives and the define directives.
    fn process_source(&self, output: &mut String, shader_source: &str) -> Result<(), Error> {
        // Tracks the number of `#ifdef` directives.
        // 
        // Each source needs have an equal number of `#ifdef` and `#endif` directives.
        let mut ifdef_counter = IfDefCounter::default();
        
        for line in shader_source.lines() {
            if is_endif(line) {
                // Decrement the counter.
                // 
                // If we can't decrement the counter, then the `#endif` directive is not matched with
                // a previous `#ifdef` directive.
                if !ifdef_counter.dec() {
                    return Err(Error);
                }

                // Continue to the next line.
                continue;
            }

            if let Some(identifier) = maybe_parse_ifdef(line).map_err(|_| Error)? {
                // Increment the counter.
                ifdef_counter.inc();

                // If the identifier is not defined, then we need to skip the lines until
                // we find the matching `#endif` directive.
                if !self.define_identifiers.contains(identifier) {
                    ifdef_counter.set_skipping();
                    continue;
                }
            }

            // If we're in an unmatching `#ifdef` block, then we can skip the line.
            if ifdef_counter.is_skipping() {
                continue;
            }

            // If the line is an include directive.
            if let Some(include_path) = maybe_parse_include(line).map_err(|_| Error)? {
                self.process_include(output, include_path)?;
                continue;
            }

            // If the line is not a directive, then we simply add it to the output.
            output.push_str(line);
            output.push('\n');
        }

        if !ifdef_counter.is_zero() {
            // If the counter is not 0, then we have unmatched `#ifdef` directives.
            return Err(Error);
        }

        Ok(())
    }

    fn process_include(&self, output: &mut String, include_path: &str) -> Result<(), Error> {
        let IncludeSource { index, source } = self.include_sources.get(include_path)
            .ok_or(Error)?;

        // Check for circular includes
        if !self.include_stack.push(*index) {
            return Err(Error); // Circular include detected
        }
        
        // Process the included source
        let result = self.process_source(output, &source);
        
        // Remove from include stack
        self.include_stack.pop();
        
        result
    }
}

#[derive(Default)]
struct IncludeStack {
    stack: RefCell<Vec<IncludeId>>,
}

impl IncludeStack {
    pub fn push(&self, include_id: IncludeId) -> bool {
        let mut stack = self.stack.borrow_mut();
        if stack.contains(&include_id) {
            return false;
        }

        stack.push(include_id);
        true
    }

    pub fn pop(&self) { 
        // Pop the path from the stack.
        let mut stack = self.stack.borrow_mut();
        let _ = stack.pop();
    }
}


#[derive(Default)]
struct IfDefCounter {
    count: u8,
    skip_until: Option<u8>,
}

impl IfDefCounter {
    pub fn inc(&mut self) {
        self.count += 1;
    }
    
    /// Decrements the counter.
    /// 
    /// Returns false if decrementing would result in a negative value.
    pub fn dec(&mut self) -> bool {
        if self.count == 0 {
            return false;
        }

        // If we're skipping, and the counter is the same as the skip until, 
        // then we can resume reading lines.
        if self.is_same_block_as_unmatched_ifdef() {
            self.skip_until = None;
        }

        self.count -= 1;
        true
    }

    pub fn is_same_block_as_unmatched_ifdef(&self) -> bool {
        Some(self.count) == self.skip_until
    }

    pub fn is_skipping(&self) -> bool {
        self.skip_until.is_some()
    }

    pub fn set_skipping(&mut self) {
        self.skip_until = Some(self.count);
    }

    pub fn is_zero(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_composition() {
        let mut composer = ShaderComposer::new();
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
        let mut composer = ShaderComposer::new();
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



