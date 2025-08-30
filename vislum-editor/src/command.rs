use std::cell::RefCell;

use downcast_rs::{impl_downcast, Downcast};

use crate::editor::Editor;

/// A command that can be applied to the editor.
pub trait Command: Downcast + 'static {
    /// Applies the command to the editor.
    fn apply(&self, editor: &mut Editor);

    /// Whether the command can be merged with another command.
    ///
    /// Returns the unmerged previous command if the merge was not successful.
    fn merge(&mut self, previous: Box<dyn Command>) -> Result<(), Box<dyn Command>> {
        Err(previous)
    }

    /// Whether the command can be undone.
    fn undoable(&self) -> bool {
        false
    }
}

impl_downcast!(Command);

/// Helper function to merge two commands of the same type, or
/// return the unmerged previous command if the merge was not successful.
pub fn merge_same<T: Command>(
    command: &mut T,
    previous: Box<dyn Command>,
    can_merge: impl FnOnce(&T, &T) -> bool,
    merge: impl FnOnce(&mut T, T),
) -> Result<(), Box<dyn Command>> {
    let previous = previous.downcast::<T>()?;
    
    if !can_merge(command, &*previous) {
        return Err(previous);
    }

    merge(command, *previous);
    Ok(())
}

pub trait CommandDispatcher {
    /// Dispatch a boxed dyn command.
    fn dispatch_dyn(&self, command: Box<dyn Command>);
}

impl dyn CommandDispatcher {
    /// Dispatch a command of type `T`.
    #[inline]
    pub fn dispatch<T>(&self, command: T) 
    where 
        T: Command + 'static
    {
        self.dispatch_dyn(Box::new(command));
    }
}

struct Undo {
    pub command: Box<dyn Command>,
}

#[derive(Default)]
pub struct History {
    undo_stack: Vec<Undo>,
    queue: RefCell<Vec<Box<dyn Command>>>,

}

impl History {
    /// Add a command to the history.
    pub fn add(&self, command: Box<dyn Command>) {
        self.queue.borrow_mut().push(command);
    }
    
    pub fn process_commands(&mut self, editor: &mut Editor) {
        let mut queue = self.queue.borrow_mut();

        for mut command in queue.drain(..) {
            command.apply(editor);

            // If there's any command in the undo stack, try to merge it with the new command.
            if let Some(last) = self.undo_stack.pop() {
                // If the merge is successful, push the new command to the undo stack.
                match command.merge(last.command) {
                    Ok(_) => {
                        // Merge successful. Push the new command to the undo stack.
                        self.undo_stack.push(Undo { command: command });
                    },
                    Err(previous) => {
                        // Cannot merge the commands. 
                        //
                        // Re-add the previous command to the undo stack.
                        self.undo_stack.push(Undo { command: previous });

                        // Snapshot the current state of the editor.

                        // Push the new command to the undo stack.
                        self.undo_stack.push(Undo { command: command });
                    },
                }
            } else {
                self.undo_stack.push(Undo { command: command });
            }
        }

        dbg!(&self.undo_stack.len());
    }
}

impl CommandDispatcher for History {
    fn dispatch_dyn(&self, command: Box<dyn Command>) {
        self.add(command);
    }
}