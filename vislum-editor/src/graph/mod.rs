use std::{cell::RefCell, collections::HashMap};

mod node;
mod commands;

use eframe::egui::{self, Rect, Widget};
use slotmap::SecondaryMap;
use vislum_op::{Graph, InputIndex, NodeId, OperatorSystem, OutputIndex, Placement};

use crate::{command::{CommandDispatcher, History}, graph::{
    self,
    node::{NodeInputVirtualSlotKey, NodeOutputKey, NodeView},
}};

#[derive(Default)]
pub enum OpenedGraph {
    None,
    #[default]
    Some,
}

impl OpenedGraph {
    /// Resolves the graph from the operator system.
    ///
    /// Returns `None` if no graph is opened or the graph is not found.
    pub fn resolve<'a>(&'a self, op_system: &'a OperatorSystem) -> Option<&'a Graph> {
        match self {
            OpenedGraph::None => None,
            OpenedGraph::Some => Some(op_system.get_graph()),
        }
    }
}

pub struct GraphViewContext<'a> {
    pub op_system: &'a OperatorSystem,
    pub command_dispatcher: &'a dyn CommandDispatcher,
}

// pub(crate) struct GraphVisualInfo {
//     pub node_input_placement_rects: HashMap<(NodeId, Placement), Rect>,
//     pub node_output_placement_rects: HashMap<(NodeId, Placement), Rect>,
//     pub node_rects: SecondaryMap<NodeId, Rect>,
// }

#[derive(Default)]
pub struct GraphElementPositioning {
    pub node_input_virtual_slot_rects: HashMap<NodeInputVirtualSlotKey, Rect>,
    pub node_output_rects: HashMap<NodeOutputKey, Rect>,
    pub node_rects: SecondaryMap<NodeId, Rect>,
}

impl GraphElementPositioning {
    pub fn clear(&mut self) {
        self.node_input_virtual_slot_rects.clear();
        self.node_output_rects.clear();
        self.node_rects.clear();
    }
}

#[derive(Default)]
pub struct GraphView {
    /// Tracks which graph is currently opened.
    opened_graph: OpenedGraph,
    graph_element_positioning: GraphElementPositioning,
}

impl GraphView {
    pub fn ui(&mut self, ui: &mut egui::Ui, context: GraphViewContext) {
        self.graph_element_positioning.clear();

        let Some(graph) = self.opened_graph.resolve(&context.op_system) else {
            return;
        };

        for (node_id, node) in graph.iter() {
            let mut node_view = NodeView {
                node_id,
                node,
                element_positioning: &mut self.graph_element_positioning,
            };

            node_view.inner_ui(ui);
        }
    }

    /// Opens a new graph for editing.
    pub fn open(&mut self) {
        self.opened_graph = OpenedGraph::Some;
    }
}
