use eframe::egui;
use vislum_op::node::{GraphBlueprint, InputBlueprint, NodeId};

use crate::{command::CommandDispatcher, graph::pin::pin_ui};

pub struct IntrospectView<'a> {
    node_id: NodeId,
    graph: &'a GraphBlueprint,
    dispatcher: &'a dyn CommandDispatcher,
}

impl<'a> IntrospectView<'a> {
    pub fn new(graph: &'a GraphBlueprint, node_id: NodeId, dispatcher: &'a dyn CommandDispatcher) -> Self {
        Self { dispatcher, node_id, graph }
    }

    pub fn ui(mut self, ui: &mut egui::Ui) {
        let Some(node) = self.graph.nodes.get(&self.node_id) else {
            return;
        };

        ui.heading("Introspect");

        ui.heading("Inputs");

        for (input, desc) in node.inputs() {
            ui.horizontal(|ui| {
                pin_ui(ui, desc.value_type);
                ui.label(&desc.name);

                let label = match input {
                    InputBlueprint::Constant(tagged_value) => "constant",
                    InputBlueprint::Connection(connection) => "connection",
                    InputBlueprint::ConnectionVec(connections) => "connection",
                    InputBlueprint::Unset => "unset",
                };

                ui.label(label);
            });
        }
    }
}
