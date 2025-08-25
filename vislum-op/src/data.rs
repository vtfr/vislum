use std::{collections::HashMap, rc::Rc};

use serde::{
    Deserialize, Serialize,
};
use thiserror::Error;

use crate::{
    ErasedSlot, ExportableUUID, Graph, GraphError, InputIndex, InputSlots, NodeId, Operator, OperatorTypeId, OperatorTypeRegistry, OutputIndex, Placement
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
    pub fn get_or_compute(&mut self, operator: &Box<dyn Operator>) -> Rc<HashMap<String, InputIndex>> {
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
            self.import_node_inputs(*stable_node_id, node_data)?;
        }

        Ok(self.graph)
    }

    pub fn import_node_inputs(
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
        let node = self.graph
            .get_node(node_id)
            .unwrap();

        // Get the input mapping table for the operator.
        let input_mapping_table = self.node_input_mapping_table.get_or_compute(&node.operator);

        // Assign all inputs.
        for (input_name, input_slot_datas) in node_data.inputs.iter() {
            // Find the input index for the input name.
            //
            // Unknown inputs are ignored.
            if let Some(input_index) = input_mapping_table.get(input_name) {
                for input_slot_data in input_slot_datas.iter() {
                    match input_slot_data {
                        InputSlotData::Constant(value) => {
                            todo!()
                        }
                        InputSlotData::Connection(NodeConnectionData {
                            node_id: from_node_stable_id,
                            output_index,
                        }) => {
                            let from_node_id =
                                self.node_stable_id_mapping
                                    .get(from_node_stable_id)
                                    .copied()
                                    .ok_or_else(|| GraphImportError::NodeNotFound(*from_node_stable_id))?;

                            // Perform the connection.
                            self.graph.connect(
                                from_node_id,
                                *output_index,
                                node_id,
                                *input_index,
                                Placement::End,
                            )?;
                        }
                    };
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
        let mut nodes: HashMap<ExportableUUID, NodeData> = HashMap::with_capacity(self.graph.nodes.len());

        // Warm-up the node cache.
        let node_exportable_uuid_mapping = self.graph.iter()
            .map(|(node_id, node)| {
                (node_id, node.exportable_uuid)
            })
            .collect::<HashMap<_, _>>();

        // Write all the nodes.
        for (node_id, node) in self.graph.iter() {
            let mut input_datas: HashMap<String, Vec<InputSlotData>> = HashMap::with_capacity(node.operator.num_inputs());

            // Write all the inputs.
            for input in node.operator.inputs() {
                let mut input_slot_datas: Vec<InputSlotData> = Vec::with_capacity(input.num_slots());

                for slot in InputSlots::new(input) {
                    let slot_data = match slot {
                        ErasedSlot::Constant(_tagged_value) => {
                            InputSlotData::Constant(serde_json::Value::Null)
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

                let input_name = input.info()
                    .name
                    .to_string();

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

        GraphData {
            nodes,
        }
    }
}