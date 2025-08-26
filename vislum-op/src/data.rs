use std::{collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ErasedSlot, ExportableUUID, Graph, GraphError, InputIndex, InputSlots, NodeConnection, NodeId, Operator, OperatorTypeId, OperatorTypeRegistry, OutputIndex, Placement, TaggedValue
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

/// A table mapping operator types to input names to input indices.
#[derive(Default)]
struct NodeInputMappingTable {
    mapping: HashMap<OperatorTypeId<'static>, Rc<HashMap<String, InputIndex>>>,
}

impl NodeInputMappingTable {
    pub fn get_or_compute(
        &mut self,
        operator: &Box<dyn Operator>,
    ) -> Rc<HashMap<String, InputIndex>> {
        let operator_type_id = operator.type_id();

        self.mapping
            .entry(operator_type_id.into_owned())
            .or_insert_with(|| {
                let table = operator
                    .inputs()
                    .enumerate()
                    .map(|(index, input)| (input.info().name.to_string(), index))
                    .collect();

                Rc::new(table)
            })
            .clone()
    }
}

pub struct GraphImporter<'a> {
    /// The registry of operator types.
    registry: &'a OperatorTypeRegistry,

    /// The graph data to import.
    graph_data: &'a GraphData,

    /// The graph to import to.
    graph: Graph,

    /// A table mapping operator types to input names to input indices.
    node_input_mapping_table: NodeInputMappingTable,

    /// A table mapping stable node ids to node ids.
    node_stable_id_mapping: HashMap<ExportableUUID, NodeId>,
}

impl<'a> GraphImporter<'a> {
    pub fn new(registry: &'a OperatorTypeRegistry, graph_data: &'a GraphData) -> Self {
        Self {
            registry,
            graph_data,
            graph: Graph::new(),
            node_input_mapping_table: Default::default(),
            node_stable_id_mapping: Default::default(),
        }
    }

    pub fn import(mut self) -> Result<Graph, GraphImportError> {
        // Phase 1: Import all nodes.
        for (stable_node_id, node_data) in self.graph_data.nodes.iter() {
            self.import_node(*stable_node_id, node_data)?;
        }

        // Phase 2: Assign all inputs.
        for (stable_node_id, node_data) in self.graph_data.nodes.iter() {
            self.wire_node(*stable_node_id, node_data)?;
        }

        Ok(self.graph)
    }

    pub fn wire_node(
        &mut self,
        stable_node_id: ExportableUUID,
        node_data: &NodeData,
    ) -> Result<(), GraphImportError> {
        // SAFETY: All nodes were added to the mapping table in the previous phase.
        let node_id = self
            .node_stable_id_mapping
            .get(&stable_node_id)
            .copied()
            .unwrap();

        // SAFETY: All nodes were added to the graph in the previous phase.
        let node = self.graph.get_node_mut(node_id).unwrap();

        // Get the input mapping table for the operator.
        let input_mapping_table = self.node_input_mapping_table.get_or_compute(&node.operator);

        // This can be drastically simplified by separating wiring information 
        // from materialization itself.
        //
        // Phase 1: Creates nodes with no values and the WiringSpecification
        // Phase 2: Applies the WiringSpecification to the all nodes by following
        struct MaterializedInput {
            input_index: InputIndex,
            data: Vec<ErasedSlot>,
        }

        let mut inputs_materialized: Vec<MaterializedInput> = Vec::new();
        
        // Materialize all inputs.
        for (input_name, input_slot_datas) in node_data.inputs.iter() {
            // Find the input index for the input name.
            //
            // Unknown inputs are ignored.
            if let Some(input_index) = input_mapping_table.get(input_name) {
                let mut input_materialized = MaterializedInput {
                    input_index: *input_index,
                    data: Vec::new(),
                };

                for input_slot_data in input_slot_datas.iter() {
                    match input_slot_data {
                        InputSlotData::Constant(value) => {
                            let value = node.operator.get_input(*input_index)
                                .unwrap()
                                .type_info()
                                .serialization
                                .as_ref()
                                .unwrap()
                                .deserialize(value.clone())
                                .unwrap();

                            input_materialized.data.push(ErasedSlot::Constant(value));
                        }
                        InputSlotData::Connection(NodeConnectionData {
                            node_id: from_node_stable_id,
                            output_index,
                        }) => {
                            let from_node_id = self
                                .node_stable_id_mapping
                                .get(from_node_stable_id)
                                .copied()
                                .unwrap();

                            input_materialized.data.push(ErasedSlot::Connection(NodeConnection {
                                node_id: from_node_id,
                                output_index: *output_index,
                            }));
                        }
                    }
                }

                inputs_materialized.push(input_materialized);
            }
        }

        // Wire all inputs.
        for MaterializedInput { input_index, data: slots } in inputs_materialized {
            for slot in slots {
                match slot {
                    ErasedSlot::Constant(tagged_value) => {
                        self.graph.assign_constant(node_id, input_index, Placement::End, tagged_value)?;
                    }
                    ErasedSlot::Connection(node_connection) => {
                        self.graph.connect(
                            node_connection.node_id,
                            node_connection.output_index,
                            node_id,
                            input_index,
                            Placement::End,
                        )?;
                    }
                    ErasedSlot::Dangling => unreachable!(),
                }
            }
        }

        Ok(())
    }

    pub fn import_node(
        &mut self,
        stable_node_id: ExportableUUID,
        node_data: &NodeData,
    ) -> Result<(), GraphImportError> {
        let operator_type_info = self
            .registry
            .get(node_data.operator_type_id.clone())
            .ok_or_else(|| {
                GraphImportError::OperatorNotFound(node_data.operator_type_id.clone())
            })?;

        let operator = operator_type_info.construct();
        let node_id = self.graph.add_node(operator);

        self.node_stable_id_mapping.insert(stable_node_id, node_id);
        Ok(())
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
