use vislum_core::prelude::*;

pub trait Command {
    fn apply(&self, engine: &mut Engine);

    fn undoable(&self) -> bool { false }

    fn undo(&self, engine: &mut Engine) { }
}

pub struct CreateNodeCommand {
    pub node_type: NodeTypeId,
    pub created_node_id: Option<NodeId>,
}

impl Command for CreateNodeCommand {
    fn apply(&self, engine: &mut Engine) {
        self.created_node_id = Some(engine.get_node_graph_system().create_node(self.node_type));
    }

    fn undoable(&self) -> bool {
        true
    }

    fn undo(&self, engine: &mut Engine) {
        if let Some(node_id) = self.created_node_id {
            engine.get_node_graph_system().remove_node(node_id);
        }
    }
}

#[derive(Default)]
pub struct History {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    queue: Vec<Box<dyn Command>>,
}

impl History {
    /// Pushes a command to the queue, to be applied at the end of the frame.
    pub fn push(&mut self, command: Box<dyn Command>) {
        self.queue.push(command);
    }

    /// Applies all commands in the queue to the engine.
    pub fn apply(&mut self, engine: &mut Engine) {
        // Clear the redo stack when applying a command, as these commands are no longer valid.
        self.redo_stack.clear();

        while let Some(command) = self.queue.pop() {
            command.apply(engine);

            if !command.undoable() {
                // If the command is not undoable, clear the redo stack, as we can't undo it,
                // so the redo stack is no longer valid.
                self.redo_stack.clear();
            } else {
                // If the command is undoable, add it to the undo stack.
                self.undo_stack.push(command);
            }
        }
    }

    /// Undoes the last command.
    pub fn undo(&mut self, engine: &mut Engine) {
        if let Some(command) = self.undo_stack.pop() {
            command.undo(engine);
            self.redo_stack.push(command);
        }
    }

    /// Redoes the last command.
    pub fn redo(&mut self, engine: &mut Engine) {
        if let Some(command) = self.redo_stack.pop() {
            command.apply(engine);
            self.undo_stack.push(command);
        }
    }

    /// Returns true if we can undo the last command.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if we can redo the last command.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}