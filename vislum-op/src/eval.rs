use std::{cell::UnsafeCell, marker::PhantomData};

use slotmap::SecondaryMap;
use thiserror::Error;

use crate::{Graph, Node, NodeId, OutputIndex, TaggedValue};

struct EvaluatableGraphRef<'a> {
    pub(crate) graph: *mut Graph,
    #[cfg(debug_assertions)]
    pub(crate) node_borrows: UnsafeCell<SecondaryMap<NodeId, ()>>,
    _phantom: PhantomData<&'a Graph>,
}

impl<'a> EvaluatableGraphRef<'a> {
    pub fn new(graph: &'a mut Graph) -> Self {
        Self {
            graph: graph as *mut Graph,
            #[cfg(debug_assertions)]
            node_borrows: UnsafeCell::new(SecondaryMap::with_capacity(graph.nodes.len())),
            _phantom: PhantomData,
        }
    }

    pub fn borrow_node_mut(&self, node_id: NodeId) -> Option<NodeMutRef> {
        unsafe {
            let graph = &mut *(self.graph as *mut Graph);
            let node = graph.get_node_mut(node_id)?;

            #[cfg(debug_assertions)]
            self.track_borrow(node_id);

            Some(NodeMutRef {
                #[cfg(debug_assertions)]
                cell: self,
                #[cfg(debug_assertions)]
                node_id,
                node,
            })
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn track_borrow(&self, node_id: NodeId) {
        let node_borrows = unsafe { &mut *self.node_borrows.get() };
        if node_borrows.insert(node_id, ()).is_some() {
            panic!("Node {:?} already borrowed", node_id);
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn untrack_borrow(&self, node_id: NodeId) {
        let node_borrows = unsafe { &mut *self.node_borrows.get() };
        node_borrows.remove(node_id);
    }
}

struct NodeMutRef<'a> {
    #[cfg(debug_assertions)]
    pub(crate) cell: &'a EvaluatableGraphRef<'a>,
    #[cfg(debug_assertions)]
    pub(crate) node_id: NodeId,
    pub(crate) node: &'a mut Node,
}

#[cfg(debug_assertions)]
impl<'a> Drop for NodeMutRef<'a> {
    fn drop(&mut self) {
        self.cell.untrack_borrow(self.node_id);
    }
}

impl<'a> std::ops::Deref for NodeMutRef<'a> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        self.node
    }
}

impl<'a> std::ops::DerefMut for NodeMutRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node
    }
}

pub struct Evaluator<'a> {
    evaluatable_graph: EvaluatableGraphRef<'a>,
    systems: EvaluationSystems<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn new(graph: &'a mut Graph, systems: EvaluationSystems<'a>) -> Self {
        Self {
            evaluatable_graph: EvaluatableGraphRef::new(graph),
            systems,
        }
    }

    /// Gets the output value of a node.
    ///
    /// Evaluates the node if it has not been evaluated yet.
    pub fn get_node_output(
        &self,
        node_id: NodeId,
        output_index: OutputIndex,
    ) -> Result<TaggedValue, EvalError> {
        let mut node = self
            .evaluatable_graph
            .borrow_node_mut(node_id)
            .ok_or(EvalError)?;

        // Evaluate the node.
        node.operator.evaluate(EvaluateContext {
            node_id,
            evaluator: self,
        })?;

        // Get the output value.
        let output = node
            .operator
            .get_output(output_index)
            .and_then(|output| output.get_value())
            .ok_or(EvalError)?;

        Ok(output)
    }
}

/// The context in which an evaluation is performed.
///
/// Tracks all systems that are used in the evaluation.
pub struct EvaluationSystems<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> EvaluationSystems<'a> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// The context in which an operator is evaluated.
#[derive(Copy, Clone)]
pub struct EvaluateContext<'a> {
    pub node_id: NodeId,
    pub evaluator: &'a Evaluator<'a>,
}

#[derive(Debug, Error)]
#[error("Evaluation error")]
pub struct EvalError;
