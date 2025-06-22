use std::marker::PhantomData;

use slotmap::{SecondaryMap, SlotMap};
use thiserror::Error;

use crate::{
    ErasedSlot, InputIndex, Inputs, InputsMut, Operator, OutputIndex, Outputs, Placement,
    SValueTypeInfo, SlotError, TaggedValue, ValueTypeId,
};

/// Represents a connection to an output of a given node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeConnection {
    pub node_id: NodeId,
    pub output_index: OutputIndex,
}

impl NodeConnection {
    pub const fn new(node_id: NodeId, output_index: OutputIndex) -> Self {
        Self {
            node_id,
            output_index,
        }
    }
}

slotmap::new_key_type! {
    pub struct NodeId;
}

/// A node in the graph.
pub struct Node {
    pub operator: Box<dyn Operator>,
}

impl Node {
    pub fn new(operator: Box<dyn Operator>) -> Self {
        Self { operator }
    }

    pub fn inputs(&self) -> Inputs {
        Inputs::new(&*self.operator)
    }

    pub fn inputs_mut(&mut self) -> InputsMut {
        InputsMut::new(&mut *self.operator)
    }

    pub fn outputs(&self) -> Outputs {
        Outputs::new(&*self.operator)
    }
}

/// A graph of nodes.
#[derive(Default)]
pub struct Graph {
    pub nodes: SlotMap<NodeId, Node>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new node to the graph and returns its ID.
    pub fn add_node(&mut self, operator: Box<dyn Operator>) -> NodeId {
        self.nodes.insert(Node::new(operator))
    }

    /// Returns a reference to the node with the given ID.
    pub fn get_node(&self, node_id: NodeId) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    /// Returns a mutable reference to the node with the given ID.
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(node_id)
    }

    /// Removes the node with the given ID from the graph.
    pub fn remove_node(&mut self, removed_node_id: NodeId) {
        self.nodes.remove(removed_node_id);

        // Remove all connections to the node.
        for (_, node) in self.nodes.iter_mut() {
            for input in node.inputs_mut() {
                input.invalidate_connection(removed_node_id);
            }
        }
    }

    /// Returns an iterator over all nodes in the graph.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes.iter()
    }

    pub fn connect(
        &mut self,
        from_node_id: NodeId,
        from_output_index: OutputIndex,
        to_node_id: NodeId,
        to_input_index: InputIndex,
        placement: Placement,
    ) -> Result<(), GraphError> {
        self.can_connect(from_node_id, from_output_index, to_node_id, to_input_index)?;

        let to_node = self
            .nodes
            .get_mut(to_node_id)
            .expect("The node must be in the graph");

        to_node
            .operator
            .get_input_mut(to_input_index)
            .expect("The input must be in the node")
            .set_slot(
                placement,
                ErasedSlot::Connection(NodeConnection::new(from_node_id, from_output_index)),
            )
            .map_err(|error| GraphError::SlotError {
                node_id: to_node_id,
                input_index: to_input_index,
                placement,
                error,
            })?;

        Ok(())
    }

    /// Assigns a constant value to an input.
    ///
    /// Returns `Ok(())` if the constant value is assigned, `Err(GraphError)` otherwise.
    pub fn assign_constant(
        &mut self,
        node_id: NodeId,
        input_index: InputIndex,
        placement: Placement,
        value: TaggedValue,
    ) -> Result<(), GraphError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or(GraphError::NodeNotFound(node_id))?;

        node.operator
            .get_input_mut(input_index)
            .ok_or(GraphError::InputNotFound(node_id, input_index))?
            .set_slot(placement, ErasedSlot::Constant(value))
            .map_err(|error| GraphError::SlotError {
                node_id,
                input_index,
                placement,
                error,
            })?;

        Ok(())
    }

    /// Checks if nodes can be connected.
    ///
    /// Returns `Ok(())` if the nodes can be connected, `Err(GraphError)` otherwise.
    pub fn can_connect(
        &self,
        from_node_id: NodeId,
        from_output_index: OutputIndex,
        to_node_id: NodeId,
        to_input_index: InputIndex,
    ) -> Result<(), GraphError> {
        let from_node = self
            .get_node(from_node_id)
            .ok_or(GraphError::NodeNotFound(from_node_id))?;

        let to_node = self
            .get_node(to_node_id)
            .ok_or(GraphError::NodeNotFound(to_node_id))?;

        let from_output = from_node
            .operator
            .get_output(from_output_index)
            .ok_or(GraphError::OutputNotFound(from_node_id, from_output_index))?;

        let to_input = to_node
            .operator
            .get_input(to_input_index)
            .ok_or(GraphError::InputNotFound(to_node_id, to_input_index))?;

        if from_output.type_info().id != to_input.type_info().id {
            return Err(GraphError::IncompatibleValueTypes {
                expected: from_output.type_info().id,
                actual: to_input.type_info().id,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("The node is not in the graph: {0:?}")]
    NodeNotFound(NodeId),

    #[error("The output is not in the node: {0:?} (output {1})")]
    OutputNotFound(NodeId, OutputIndex),

    #[error("The input is not in the node: {0:?} (input {1})")]
    InputNotFound(NodeId, InputIndex),

    #[error("The value types are not compatible: {expected:?} != {actual:?}")]
    IncompatibleValueTypes {
        expected: ValueTypeId<'static>,
        actual: ValueTypeId<'static>,
    },

    #[error("Failed to set slot at {placement} for node {node_id:?} input {input_index}: {error}")]
    SlotError {
        node_id: NodeId,
        input_index: InputIndex,
        placement: Placement,
        error: SlotError,
    },
}
