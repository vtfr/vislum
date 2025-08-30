use std::{cell::RefCell, collections::{HashMap, HashSet}};

mod commands;
mod node;
mod pin;

use eframe::{
    egui::{self, Color32, InnerResponse, Rect, Scene, Sense, Stroke, UiBuilder, Widget},
    epaint::RectShape,
};
use slotmap::SecondaryMap;
use vislum_op::{prelude::*, system::NodeGraphSystem};

use crate::{
    command::{CommandDispatcher, History},
    graph::{
        self, commands::{AddNodeCommand, DeleteNodesCommand, MoveNodesCommand}, node::{NodeAction, NodeInputVirtualSlotKey, NodeOutputKey, NodeView}
    }, util::IntoVector2I,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Interaction {
    /// The user isn't doing anything interesting.
    #[default]
    Hover,
    /// The user is performing a connection.
    Connecting,
}

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
    pub fn resolve<'a>(&'a self, op_system: &'a NodeGraphSystem) -> Option<&'a GraphBlueprint> {
        match self {
            OpenedGraph::None => None,
            OpenedGraph::Some => Some(op_system.get_graph()),
        }
    }
}

pub struct GraphViewContext<'a> {
    pub op_system: &'a NodeGraphSystem,
    pub dispatcher: &'a dyn CommandDispatcher,
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
    pub node_rects: HashMap<NodeId, Rect>,
}

impl GraphElementPositioning {
    pub fn clear(&mut self) {
        self.node_input_virtual_slot_rects.clear();
        self.node_output_rects.clear();
        self.node_rects.clear();
    }
}

pub struct GraphView {
    opened_graph: OpenedGraph,
    graph_element_positioning: GraphElementPositioning,
    interaction: Interaction,
    scene_rect: Rect,
}

impl Default for GraphView {
    fn default() -> Self {
        Self {
            opened_graph: Default::default(),
            graph_element_positioning: Default::default(),
            scene_rect: Rect::ZERO,
            interaction: Default::default(),
        }
    }
}

impl GraphView {
    pub fn ui(&mut self, ui: &mut egui::Ui, context: GraphViewContext) {
        // If no graph is opened, return.
        if self.opened_graph.resolve(&context.op_system).is_none() {
            return;
        };

        // Clear the graph element positioning.
        self.graph_element_positioning.clear();

        // Main central panel.
        egui::containers::CentralPanel::default().show_inside(ui, |ui| {
            self.nodes_ui(ui, context);
        });
    }

    fn nodes_ui(&mut self, ui: &mut egui::Ui, context: GraphViewContext) {
        let mut node_responses = Vec::new();

        let scene_response = Scene::new().show(ui, &mut self.scene_rect, |ui| {
            let graph = self.opened_graph.resolve(&context.op_system).unwrap();

            for (node_id, node) in graph.nodes.iter() {
                let node_view = NodeView::new(
                    *node_id,
                    node,
                    &mut self.graph_element_positioning,
                );

                node_responses.push(node_view.ui(ui));
            }
        });

        for node_response in node_responses {
            for action in node_response.actions {
                match action {
                    NodeAction::TitleDragged(delta) => {
                        context.dispatcher.dispatch_dyn(Box::new(MoveNodesCommand {
                            node_ids: HashSet::from([node_response.node_id]),
                            delta: delta.into_vector2i(),
                        }));
                    }
                    NodeAction::Delete => {
                        context.dispatcher.dispatch_dyn(Box::new(DeleteNodesCommand {
                            node_ids: HashSet::from([node_response.node_id]),
                        }));
                    }
                    _ => {}
                }
            }
        }

        // Open the context menu UI when clicked on the background.
        scene_response
            .response
            .context_menu(|ui| {
                ui.menu_button("New operator", |ui| {
                    for node_type in context.op_system.get_node_type_registry().iter() {
                        if ui.button(&*node_type.id).clicked() {
                            context.dispatcher.dispatch_dyn(Box::new(AddNodeCommand {
                                node_type_id: node_type.id.clone(),
                            }));
                        }
                    }
                });
            });
    }

    /// Opens a new graph for editing.
    pub fn open(&mut self) {
        self.opened_graph = OpenedGraph::Some;
    }
}
