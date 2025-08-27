use std::collections::HashSet;

use vislum_op::NodeId;

use crate::{command::{merge_same, Command}, editor::Editor};

/// Updates the positions of the given nodes.
pub struct UpdateNodePositions {
    pub node_ids: HashSet<NodeId>,
    pub delta: (f32, f32)
}

impl Command for UpdateNodePositions {
    fn apply(&self, editor: &mut Editor) {
        let mut graph = editor.runtime.get_operator_system_mut().get_graph_mut();

        for node_id in &self.node_ids {
            if let Some(node) = graph.get_node_mut(*node_id) {
                // TODO: Update the node position.
            }
        }
    }

    fn merge(&mut self, previous: Box<dyn Command>) -> Result<(), Box<dyn Command>> {
        merge_same::<Self>(self, previous, 
            // Can merge if the node ids are the same.
            |command, previous| {
                command.node_ids == previous.node_ids
            },
            // Merge the deltas.
            |command, previous| {
                command.delta = (
                    command.delta.0 + previous.delta.0,
                    command.delta.1 + previous.delta.1
                );
        })
    }
    
    fn undoable(&self) -> bool {
        true
    }
}

/// Deletes the given nodes.
pub struct DeleteNodes {
    pub node_ids: HashSet<NodeId>,
}

impl Command for DeleteNodes {
    fn apply(&self, editor: &mut Editor) {
        let graph = editor.runtime.get_operator_system_mut().get_graph_mut();
        for node_id in &self.node_ids {
            graph.remove_node(*node_id);
        }
    }

    fn undoable(&self) -> bool {
        true
    }
}