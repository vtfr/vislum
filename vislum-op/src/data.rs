use std::{collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ErasedSlot, ExportableUUID, Graph, GraphError, InputIndex, InputSlots, NodeConnection, NodeId,
    Operator, OperatorTypeId, OperatorTypeRegistry, OutputIndex, Placement, TaggedValue, ValueType,
};

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub operator_type_id: OperatorTypeId<'static>,
    pub inputs: HashMap<String, Vec<InputSlotData>>,
    pub position: (f32, f32),
}

#[derive(Serialize, Deserialize)]
pub struct NodeConnectionData {
    pub node_id: ExportableUUID,
    pub output_index: OutputIndex,
}

#[derive(Serialize, Deserialize)]
pub enum InputSlotData {
    Constant(serde_json::Value),
    Connection(NodeConnectionData),
}

#[derive(Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: HashMap<ExportableUUID, NodeData>,
}

pub struct GraphDataDeserializer<'a> {
    pub registry: &'a OperatorTypeRegistry,
    pub graph: Graph,
}

#[derive(Debug, Error)]
pub enum GraphImportError {
    #[error("A reference to a node with stable id {0} was not found in graph")]
    NodeNotFound(ExportableUUID),
    #[error("Operator type {0} not found in registry")]
    OperatorNotFound(OperatorTypeId<'static>),
    #[error("Graph error: {0}")]
    GraphError(#[from] GraphError),
}

enum WiringSpecification {
    Connection {
        from_node_id: ExportableUUID,
        from_output_index: OutputIndex,
        to_node_id: ExportableUUID,
        to_input_index: InputIndex,
    },
    Constant {
        stable_node_id: ExportableUUID,
        input_index: InputIndex,
        value: TaggedValue,
    },
}

pub struct GraphImporter<'a> {
    /// The registry of operator types.
    registry: &'a OperatorTypeRegistry,

    /// The graph data to import.
    graph_data: &'a GraphData,

    /// The graph to import to.
    graph: Graph,

    /// A table mapping stable node ids to node ids.
    node_stable_id_mapping: HashMap<ExportableUUID, NodeId>,

    /// A table mapping operator types to input names to input indices.
    wiring_specification: Vec<WiringSpecification>,
}

impl<'a> GraphImporter<'a> {
    pub fn new(registry: &'a OperatorTypeRegistry, graph_data: &'a GraphData) -> Self {
        Self {
            registry,
            graph_data,
            graph: Graph::new(),
            node_stable_id_mapping: Default::default(),
            wiring_specification: Default::default(),
        }
    }

    pub fn import(mut self) -> Result<Graph, GraphImportError> {
        let mut wiring_specifications: Vec<WiringSpecification> = Vec::new();

        // Phase 1: Import all nodes.
        for (stable_node_id, node_data) in self.graph_data.nodes.iter() {
            let operator_type = self
                .registry
                .get(node_data.operator_type_id.clone())
                .ok_or_else(|| {
                    GraphImportError::OperatorNotFound(node_data.operator_type_id.clone())
                })?;

            let operator = operator_type.construct();

            // Generate wiring specifications for all inputs.
            for (input_name, input_slot_datas) in node_data.inputs.iter() {
                let Some((input_index, input_specification)) =
                    operator_type.get_input_specification(input_name)
                else {
                    continue;
                };

                for slot in input_slot_datas.iter() {
                    match slot {
                        InputSlotData::Constant(value) => {
                            // Deserialize the value.
                            let value = input_specification
                                .value_type
                                .serialization
                                .as_ref()
                                .unwrap()
                                .deserialize(value.clone())
                                .unwrap();

                            wiring_specifications.push(WiringSpecification::Constant {
                                stable_node_id: *stable_node_id,
                                input_index: input_index,
                                value: value,
                            });
                        }
                        InputSlotData::Connection(node_connection) => {
                            wiring_specifications.push(WiringSpecification::Connection {
                                from_node_id: node_connection.node_id,
                                from_output_index: node_connection.output_index,
                                to_node_id: *stable_node_id,
                                to_input_index: input_index,
                            });
                        }
                    }
                }
            }

            let node_id = self.graph.add_node(operator);

            self.node_stable_id_mapping.insert(*stable_node_id, node_id);
        }

        // Phase 2: Wire all nodes.
        for wiring_specification in wiring_specifications {
            match wiring_specification {
                WiringSpecification::Connection {
                    from_node_id,
                    from_output_index,
                    to_node_id,
                    to_input_index,
                } => {
                    let from_node_id = self
                        .node_stable_id_mapping
                        .get(&from_node_id)
                        .copied()
                        .unwrap();
                    let to_node_id = self
                        .node_stable_id_mapping
                        .get(&to_node_id)
                        .copied()
                        .unwrap();

                    self.graph.connect(
                        from_node_id,
                        from_output_index,
                        to_node_id,
                        to_input_index,
                        Placement::End,
                    )?;
                }
                WiringSpecification::Constant {
                    stable_node_id,
                    input_index,
                    value,
                } => {
                    let node_id = self
                        .node_stable_id_mapping
                        .get(&stable_node_id)
                        .copied()
                        .unwrap();

                    self.graph
                        .assign_constant(node_id, input_index, Placement::End, value)?;
                }
            }
        }

        Ok(self.graph)
    }
}

pub enum ExportableUuidAssignment {
    /// Keep the exportable uuid.
    Keep,
    /// Randomize the exportable uuid.
    Randomize,
}

pub struct GraphExporter<'a> {
    graph: &'a Graph,
}

impl<'a> GraphExporter<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    pub fn export(&self) -> GraphData {
        let mut nodes: HashMap<ExportableUUID, NodeData> =
            HashMap::with_capacity(self.graph.nodes.len());

        // Warm-up the node cache.
        let node_exportable_uuid_mapping = self
            .graph
            .iter()
            .map(|(node_id, node)| (node_id, node.exportable_uuid))
            .collect::<HashMap<_, _>>();

        // Write all the nodes.
        for (node_id, node) in self.graph.iter() {
            let mut input_datas: HashMap<String, Vec<InputSlotData>> =
                HashMap::with_capacity(node.operator.num_inputs());

            // Write all the inputs.
            for input in node.operator.inputs() {
                let mut input_slot_datas: Vec<InputSlotData> =
                    Vec::with_capacity(input.num_slots());

                for slot in InputSlots::new(input) {
                    let slot_data = match slot {
                        ErasedSlot::Constant(ref tagged_value) => {
                            if let Some(value) = tagged_value.type_info().serialization.as_ref() {
                                match value.serialize(tagged_value.clone()) {
                                    Ok(data) => InputSlotData::Constant(data),
                                    Err(_) => continue,
                                }
                            } else {
                                continue;
                            }
                        }
                        ErasedSlot::Connection(node_connection) => {
                            InputSlotData::Connection(NodeConnectionData {
                                node_id: node_exportable_uuid_mapping[&node_connection.node_id],
                                output_index: node_connection.output_index,
                            })
                        }
                        ErasedSlot::Dangling => {
                            continue;
                        }
                    };

                    input_slot_datas.push(slot_data);
                }

                let input_name = input.info().name.to_string();

                input_datas.insert(input_name, input_slot_datas);
            }

            // Write the node.
            let node_data = NodeData {
                operator_type_id: node.operator.type_id().into_owned(),
                inputs: input_datas,
                position: (0.0, 0.0),
            };

            nodes.insert(node_exportable_uuid_mapping[&node_id], node_data);
        }

        GraphData { nodes }
    }
}
