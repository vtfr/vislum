use std::collections::HashMap;

use thiserror::Error;
use vislum_math::Vector2I;
use vislum_system::Resource;

use crate::{
    compile::CompilationContext,
    eval::{EvalContext, EvalError, NodeRef},
    node::{Connection, ConnectionPlacement, GraphBlueprint, GraphError, InputId, NodeId},
    node_type::{NodeTypeRegistry, RegisterNodeType},
    prelude::NodeType,
    value::TaggedValue,
};

#[derive(Resource, Default)]
pub struct NodeGraphSystem {
    /// The registry of node types.
    node_type_registry: NodeTypeRegistry,

    /// The graph of nodes.
    graph: GraphBlueprint,

    /// Whether the graph needs recompilation.
    needs_recompilation: bool,

    /// The compiled nodes.
    ///
    /// If the graph:
    /// - Is not compiled, this will be `None`.
    /// - Is compiled without errors, this will be `Some(Result<HashMap<NodeId, NodeRef>, ()>)`.
    /// - Is compiled with errors, this will be `Some(Err(()))`.
    compiled_nodes: Option<Result<HashMap<NodeId, NodeRef>, ()>>,
}

impl NodeGraphSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_node_type_registry(&self) -> &NodeTypeRegistry {
        &self.node_type_registry
    }

    /// Add a node type to the registry.
    pub fn add_node_type(&mut self, node_type: NodeType) {
        self.node_type_registry.add(node_type);
    }

    /// Register a node type.
    #[inline(always)]
    pub fn register_node_types<T: RegisterNodeType>(&mut self) {
        self.node_type_registry.register::<T>();
    }

    /// Get the graph.
    pub fn get_graph(&self) -> &GraphBlueprint {
        &self.graph
    }

    /// Add a node of the given type to the graph.
    pub fn add_node(&mut self, node_type_id: &str) -> NodeId {
        let node_type = self.node_type_registry.get(node_type_id).unwrap();

        self.graph.add_node_of_type(node_type)
    }

    /// Remove a node from the graph.
    pub fn remove_node(&mut self, node_id: NodeId) {
        self.graph.remove_node(node_id);
        self.needs_recompilation = true;
    }

    pub fn update_node_positions_with_offset(
        &mut self,
        node_ids: impl Iterator<Item = NodeId>,
        offset: Vector2I,
    ) {
        self.graph
            .update_node_positions_with_offset(node_ids, offset);
    }

    /// Get the compiled nodes.
    pub fn assign_constant(
        &mut self,
        node_id: NodeId,
        input_id: InputId,
        value: TaggedValue,
    ) -> Result<(), GraphError> {
        self.graph.assign_constant(node_id, input_id, value)?;
        self.needs_recompilation = true;
        Ok(())
    }

    pub fn connect(
        &mut self,
        node_id: NodeId,
        input_id: InputId,
        placement: ConnectionPlacement,
        connection: Connection,
    ) -> Result<(), GraphError> {
        self.graph
            .connect(node_id, input_id, placement, connection)?;
        self.needs_recompilation = true;
        Ok(())
    }

    pub fn eval(
        &mut self,
        ctx: &EvalContext,
        entry_point: NodeId,
    ) -> Result<&NodeRef, NodeGraphSystemError> {
        if self.needs_recompilation {
            self.compile_graph();
            self.needs_recompilation = false;
        }

        match &self.compiled_nodes {
            Some(Ok(compiled_nodes)) => {
                let entry_point = compiled_nodes
                    .get(&entry_point)
                    .ok_or(NodeGraphSystemError::EntryPointNotFound(entry_point))?;

                entry_point.eval(ctx)?;
                Ok(entry_point)
            }
            Some(Err(_)) => Err(NodeGraphSystemError::CompilationError),
            None => Err(NodeGraphSystemError::CompilationError),
        }
    }

    /// Compile the graph.
    pub fn compile_graph(&mut self) {
        let ctx = CompilationContext::new(&self.graph);
        self.compiled_nodes = Some(ctx.compile_all());
    }
}

#[derive(Error, Debug)]
pub enum NodeGraphSystemError {
    #[error("Eval error: {0}")]
    EvalError(#[from] EvalError),

    #[error("Compilation error")]
    CompilationError,

    #[error("Entry point not found: {0:?}")]
    EntryPointNotFound(NodeId),
}
