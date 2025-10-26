use std::{collections::HashMap, rc::Rc};

use thiserror::Error;
use vislum_math::{Vector2, Vector2I};

use crate::{
    compile::OutputDefinition,
    new_uuid_type,
    node_type::{InputCardinality, InputDefinition, NodeType},
    value::TaggedValue,
};

pub type OutputId = usize;
pub type InputId = usize;

new_uuid_type! {
    pub struct NodeId;
}

new_uuid_type! {
    pub struct GraphId;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Connection {
    pub(crate) node_id: NodeId,
    pub(crate) output_id: OutputId,
}

impl Connection {
    pub fn new(node_id: NodeId, output_id: OutputId) -> Self {
        Self { node_id, output_id }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionPlacement {
    End,
}

#[derive(Debug, Clone, Default)]
pub enum InputBlueprint {
    Constant(TaggedValue),
    Connection(Connection),
    ConnectionVec(Vec<Connection>),
    #[default]
    Unset,
}

impl InputBlueprint {
    pub fn connected_to(&self, node_id: NodeId) -> bool {
        match self {
            InputBlueprint::Connection(connection) => connection.node_id == node_id,
            InputBlueprint::ConnectionVec(connections) => connections
                .iter()
                .any(|connection| connection.node_id == node_id),
            InputBlueprint::Constant(_) | InputBlueprint::Unset => false,
        }
    }
}

pub struct NodeBlueprint {
    pub node_type: Rc<NodeType>,
    pub inputs: Vec<InputBlueprint>,
    pub position: Vector2I,
}

impl NodeBlueprint {
    #[inline]
    pub fn new(node_type: Rc<NodeType>) -> Self {
        node_type.instantiate()
    }

    #[inline]
    pub fn new_with_position(node_type: Rc<NodeType>, position: Vector2I) -> Self {
        let mut node = Self::new(node_type);
        node.position = position;
        node
    }

    #[inline]
    pub fn position(&self) -> Vector2I {
        self.position
    }

    /// Returns a reference to the node type.
    #[inline]
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    /// Returns a reference to an input.
    ///
    /// Returns an error if the input does not exist.
    pub fn get_input(&self, input_id: InputId) -> Result<&InputBlueprint, NodeError> {
        self.inputs
            .get(input_id)
            .ok_or(NodeError::InputNotFound(input_id))
    }

    #[inline]
    pub fn set_position(&mut self, position: Vector2I) {
        self.position = position;
    }

    /// Assigns a constant value to an input.
    pub fn assign_constant(
        &mut self,
        input_id: InputId,
        value: TaggedValue,
    ) -> Result<(), NodeError> {
        let (input, input_def) = self.get_input_mut_with_def(input_id)?;

        // Check if the input accepts constants.
        if !input_def.flags.accepts_constants() {
            return Err(NodeError::InputDoesNotAcceptConstants(input_id));
        }

        // Check if the value type is compatible with the input type.
        if input_def.value_type.id != value.type_info().id {
            return Err(NodeError::InputDoesNotAcceptConstants(input_id));
        }

        *input = InputBlueprint::Constant(value);
        Ok(())
    }

    /// Assigns a connection to an input.
    ///
    /// Connections are validated in the Graph.
    pub fn assign_connection(
        &mut self,
        input_id: InputId,
        placement: ConnectionPlacement,
        connection: Connection,
    ) -> Result<(), NodeError> {
        let (input, input_def) = self.get_input_mut_with_def(input_id)?;

        // Check if the input accepts connections.
        if !input_def.flags.accepts_connections() {
            return Err(NodeError::InputDoesNotAcceptConnections(input_id));
        }

        match input_def.cardinality {
            // If the input is single, we simply replace the connection.
            //
            // The placement is ignored.
            InputCardinality::Single => {
                *input = InputBlueprint::Connection(connection);
            }
            // If the input is multiple, we add the connection to the vector.
            InputCardinality::Multiple => {
                let mut connections = match input {
                    // If the input is a connection, upcast it to a connection vec.
                    InputBlueprint::ConnectionVec(connections) => connections.clone(),
                    InputBlueprint::Connection(connection) => {
                        vec![*connection]
                    }
                    InputBlueprint::Constant(_) | InputBlueprint::Unset => {
                        vec![]
                    }
                };

                match placement {
                    ConnectionPlacement::End => {
                        connections.push(connection);
                    }
                }

                *input = InputBlueprint::ConnectionVec(connections);
            }
        }

        Ok(())
    }

    pub fn reset_inputs_connected_to(&mut self, node_id: NodeId) {
        for (input, blueprint) in self.inputs.iter_mut().zip(self.node_type.inputs.iter()) {
            if input.connected_to(node_id) {
                *input = blueprint.instantiate();
            }
        }
    }

    pub fn get_input_with_def(
        &self,
        input_id: InputId,
    ) -> Result<(&InputBlueprint, &InputDefinition), NodeError> {
        let input_def = self
            .node_type
            .get_input(input_id)
            .ok_or(NodeError::InputNotFound(input_id))?;

        let input = self
            .inputs
            .get(input_id)
            .ok_or(NodeError::InputNotFound(input_id))?;

        Ok((input, input_def))
    }

    fn get_input_mut_with_def(
        &mut self,
        input_id: InputId,
    ) -> Result<(&mut InputBlueprint, &InputDefinition), NodeError> {
        let input_def = self
            .node_type
            .get_input(input_id)
            .ok_or(NodeError::InputNotFound(input_id))?;

        let input = self
            .inputs
            .get_mut(input_id)
            .ok_or(NodeError::InputNotFound(input_id))?;

        Ok((input, input_def))
    }

    pub fn inputs(&self) -> impl Iterator<Item = (&InputBlueprint, &InputDefinition)> {
        self.inputs.iter().zip(self.node_type.inputs.iter())
    }

    pub fn outputs(&self) -> impl Iterator<Item = &OutputDefinition> {
        self.node_type.outputs.iter()
    }
}

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("The input does not accept constants.")]
    InputDoesNotAcceptConstants(InputId),

    #[error("The input does not accept connections.")]
    InputDoesNotAcceptConnections(InputId),

    #[error("The value type is incompatible with the input type.")]
    IncompatibleValueType(InputId),

    #[error("The input was not found.")]
    InputNotFound(InputId),
}

#[derive(Default)]
pub struct GraphBlueprint {
    pub id: GraphId,
    pub nodes: HashMap<NodeId, NodeBlueprint>,
}

impl GraphBlueprint {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the graph.
    pub fn add_node_of_type(&mut self, node_type: Rc<NodeType>) -> NodeId {
        let id = NodeId::new();
        let node = NodeBlueprint::new(node_type);

        self.nodes.insert(id, node);
        id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&NodeBlueprint> {
        self.nodes.get(&id)
    }

    /// Iterates over the node ids in the graph.
    pub fn iter_node_ids(&self) -> impl Iterator<Item = NodeId> {
        self.nodes.keys().copied()
    }

    pub fn connect(
        &mut self,
        node_id: NodeId,
        input_id: InputId,
        placement: ConnectionPlacement,
        connection: Connection,
    ) -> Result<(), GraphError> {
        if !self.can_connect(node_id, input_id, connection) {
            return Err(GraphError::InvalidConnection {
                node_id,
                input_id,
                connection,
            });
        }

        // Unwrap is safe because we checked that the connection is valid, therefore all
        // nodes are guaranteed to exist.
        let node = self.nodes.get_mut(&node_id).unwrap();

        // Safety: we validated the connection.
        //
        // Value types are guaranteed to be compatible. Still, better safe than sorry.
        match node.assign_connection(input_id, placement, connection) {
            Ok(_) => Ok(()),
            Err(err) => Err(GraphError::NodeError {
                node_id,
                error: err,
            }),
        }
    }

    pub fn assign_constant(
        &mut self,
        node_id: NodeId,
        input_id: InputId,
        value: TaggedValue,
    ) -> Result<(), GraphError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(GraphError::NodeNotFound(node_id))?;

        match node.assign_constant(input_id, value) {
            Ok(_) => Ok(()),
            Err(err) => Err(GraphError::NodeError {
                node_id,
                error: err,
            }),
        }
    }

    /// Checks if a connection can be from to an input.
    pub fn can_connect(&self, node_id: NodeId, input_id: InputId, connection: Connection) -> bool {
        #[inline(always)]
        fn can_connect_inner(
            graph: &GraphBlueprint,
            node_id: NodeId,
            input_id: InputId,
            connection: Connection,
        ) -> Option<()> {
            let input_node = graph.nodes.get(&node_id)?;
            let input_def = input_node.node_type().get_input(input_id)?;

            let output_node = graph.nodes.get(&connection.node_id)?;
            let output_def = output_node.node_type().get_output(connection.output_id)?;

            // Check if the input accepts connections.
            if !input_def.flags.accepts_connections() {
                return None;
            }

            // Check if the value type is compatible.
            if input_def.value_type.id != output_def.value_type.id {
                return None;
            }

            Some(())
        }

        can_connect_inner(self, node_id, input_id, connection).is_some()
    }

    pub fn update_node_positions_with_offset(
        &mut self,
        node_ids: impl Iterator<Item = NodeId>,
        offset: Vector2I,
    ) {
        for node_id in node_ids {
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.position += offset;
            }
        }
    }

    pub fn remove_node(&mut self, node_id: NodeId) {
        // If the node was removed, we need to remove all reset to it.
        if self.nodes.remove(&node_id).is_some() {
            for node in self.nodes.values_mut() {
                node.reset_inputs_connected_to(node_id);
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("The node was not found.")]
    NodeNotFound(NodeId),

    #[error("The input was not found.")]
    InputNotFound(NodeId, InputId),

    #[error("Node error: {node_id:?}: {error}")]
    NodeError { node_id: NodeId, error: NodeError },

    #[error("Invalid connection: {node_id:?}: {input_id:?}: {connection:?}")]
    InvalidConnection {
        node_id: NodeId,
        input_id: InputId,
        connection: Connection,
    },
}
