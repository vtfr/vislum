use std::collections::HashSet;

use vislum_op::{prelude::{NodeId, NodeTypeId}, system::NodeGraphSystem};

use crate::{command::{merge_same, Command}, editor::Editor};

/// Updates the positions of the given nodes.
pub struct UpdateNodePositions {
    pub node_ids: HashSet<NodeId>,
    pub delta: (f32, f32)
}

impl Command for UpdateNodePositions {
    fn apply(&self, editor: &mut Editor) {
        todo!()
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
        let mut system = editor.runtime.get_system_mut::<NodeGraphSystem>();
        for node_id in &self.node_ids {
            system.remove_node(*node_id);
        }
    }

    fn undoable(&self) -> bool {
        true
    }
}

pub struct AddNodeCommand {
    pub node_type_id: NodeTypeId,
}

impl Command for AddNodeCommand {
    fn apply(&self, editor: &mut Editor) {
        let mut system = editor.runtime.get_system_mut::<NodeGraphSystem>();
        system.add_node(&self.node_type_id);
    }

    fn undoable(&self) -> bool {
        true
    }
}
