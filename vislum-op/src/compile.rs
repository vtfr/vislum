use std::collections::HashMap;

use crate::{
    node::{GraphBlueprint, InputId, NodeBlueprint, NodeId},
    node_type::EvaluationStrategy,
    eval::NodeRef,
};

/// A trait for compiling inputs during the blueprint to runtime transformation.
pub trait CompileInput {
    fn compile_input(ctx: &mut CompilationContext, node: &NodeBlueprint, input_id: InputId) -> Result<Self, ()>
    where 
        Self: Sized;
}

/// A trait for getting the input definition of a node.
/// 
/// Used by the derive macro to create the appropriate [`InputDefinition`] for the input.
pub trait GetInputDefinition {
    /// Get the input definition for the input.
    /// 
    /// The macro is free to add whatever assignment types the user added for their
    /// node, but these will be filtered out by the [`Value`] trait when appropriate.
    fn get_input_definition(name: impl Into<String>, assignment_types: AssignmentTypes) -> InputDefinition
    where 
        Self: Sized;
}

/// A trait for getting the output definition of a node.
/// 
/// Used by the derive macro to create the appropriate [`OutputDefinition`] for the output.
pub trait GetOutputDefinition {
    /// Get the output definition for the output.
    fn get_output_definition(name: impl Into<String>) -> OutputDefinition
    where 
        Self: Sized;
}

/// The context for compiling a graph from blueprint to runtime.
pub struct CompilationContext<'a> {
    pub graph: &'a GraphBlueprint,
    pub compiled: HashMap<NodeId, NodeRef>,
}

impl<'a> CompilationContext<'a> {
    pub fn new(graph: &'a GraphBlueprint) -> Self {
        Self {
            graph,
            compiled: HashMap::new(),
        }
    }

    pub fn compile_node(&mut self, node_id: NodeId) -> Result<NodeRef, ()> {
        // Check if the node is already compiled.
        if let Some(node) = self.compiled.get(&node_id) {
            return Ok(node.clone());
        }

        let node = self.graph.get_node(node_id).ok_or(())?;
        let eval_node = match node.node_type().evaluation {
            EvaluationStrategy::Compile(compile_node_fn) => {
                compile_node_fn(self, node_id, node)?
            },
        };

        self.compiled.insert(node_id, eval_node.clone());
        Ok(eval_node)
    }

    /// Compile the graph.
    /// 
    /// Returns a map of node ids to their compiled nodes.
    pub fn compile_all(mut self) -> Result<HashMap<NodeId, NodeRef>, ()> {
        for node_id in self.graph.iter_node_ids() {
            self.compile_node(node_id)?;
        };

        Ok(self.compiled)
    }
}

/// A trait for compiling nodes during the blueprint to runtime transformation.
pub trait CompileNode {
    fn compile_node(ctx: &mut CompilationContext, node_id: NodeId, node: &NodeBlueprint) -> Result<NodeRef, ()>;
}

/// A function that compiles a node.
pub type CompileNodeFn = fn(&mut CompilationContext, node_id: NodeId, node: &NodeBlueprint) -> Result<NodeRef, ()>;

// Re-export types needed by other modules
pub use crate::node_type::{AssignmentTypes, InputDefinition, OutputDefinition};
pub use crate::value::Value; 