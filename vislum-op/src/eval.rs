use std::{any::Any, cell::{Ref, RefCell}, collections::{HashMap}, rc::Rc};

use crate::{node::{GraphBlueprint, InputBlueprint, InputId, NodeBlueprint as NodeBlueprint, NodeId, OutputId}, node_type::{AssignmentTypes, EvaluationStrategy, InputCardinality}, prelude::{InputDefinition, OutputDefinition}, value::Value};

pub trait CompileInput {
    fn compile_input(ctx: &mut CompilationContext, node: &NodeBlueprint, input_id: InputId) -> Result<Self, ()>
    where 
        Self: Sized;
}

/// A trait for getting the input definition of a node.
/// 
/// Used by the derive macro to create the appropriate [`InputDefinition`] for the input.
pub trait GetInputDefinition {
    /// Get the input definition for the input.
    /// 
    /// The macro is free to add whatever assignment types the user added for their
    /// node, but these will be filtered out by the [`Value`] trait when appropriate.
    fn get_input_definition(name: impl Into<String>, assignment_types: AssignmentTypes) -> InputDefinition
    where 
        Self: Sized;
}

/// An input that accepts a single value.
pub enum Single<T> {
    Constant(T),
    Connection {
        node: NodeCell,
        output_id: OutputId,
    }
}

impl<T> CompileInput for Single<T>
where 
    T: Value,
{
    fn compile_input(ctx: &mut CompilationContext, node: &NodeBlueprint, input_id: InputId) -> Result<Self, ()> {
        match node.get_input(input_id) {
            Ok(InputBlueprint::Constant(value)) => Ok(Single::Constant(T::try_from(value.clone()).unwrap())),
            Ok(InputBlueprint::Connection(connection)) => {
                let node = ctx.compile_node(connection.node_id)?;

                Ok(Single::Connection { node, output_id: connection.output_id })
            },
            _ => Err(()),
        }
    }
}

impl<T> GetInputDefinition for Single<T>
where 
    T: Value,
{
    fn get_input_definition(name: impl Into<String>, assignment_types: AssignmentTypes) -> InputDefinition {
        InputDefinition::new(
            name, 
            &T::INFO, 
            InputCardinality::Single, 
            assignment_types,
        )
    }
}

pub trait GetOutputDefinition {
    /// Get the output definition for the output.
    /// 
    /// The macro is free to add whatever assignment types the user added for their
    /// node, but these will be filtered out by the [`Value`] trait when appropriate.
    fn get_output_definition(name: impl Into<String>) -> OutputDefinition
    where 
        Self: Sized;
}

pub struct Output<T> {
    pub value: RefCell<Option<T>>,
}

impl<T> Default for Output<T> {
    fn default() -> Self {
        Self {
            value: RefCell::new(None),
        }
    }
}

impl<T> GetOutputDefinition for Output<T> 
where 
    T: Value,
{
    fn get_output_definition(name: impl Into<String>) -> OutputDefinition {
        OutputDefinition::new(name, &T::INFO)
    }
}

/// The context for evaluating a graph.
pub struct EvalContext;

/// A cell for an evaluatable node.
/// 
/// Handles internal mutability of the node.
#[derive(Clone)]
pub struct NodeCell(Rc<RefCell<dyn Node>>);

impl NodeCell {
    pub fn new(node: impl Node + 'static) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }

    pub fn eval(&self, ctx: &EvalContext) -> Result<(), ()> {
        self.0.borrow_mut().eval(ctx)
    }

    pub fn get_output_ref(&self, output_id: OutputId) -> Option<Ref<dyn Any>> {
        let borrow = self.0.borrow();

        // Short-cut the borrow.
        if borrow.get_output_ref(output_id).is_none() {
            return None;
        }

        Some(Ref::map(borrow, |node| {
            let output = node.get_output_ref(output_id);
            
            // SAFETY: We know the output is not None because we checked earlier.
            unsafe { &*output.unwrap_unchecked() }
        }))
    }
}

/// The context for compiling a graph.
pub struct CompilationContext<'a> {
    pub graph: &'a GraphBlueprint,
    pub compiled: HashMap<NodeId, NodeCell>,
}

impl<'a> CompilationContext<'a> {
    pub fn new(graph: &'a GraphBlueprint) -> Self {
        Self {
            graph,
            compiled: HashMap::new(),
        }
    }

    pub fn compile_node(&mut self, node_id: NodeId) -> Result<NodeCell, ()> {
        // Check if the node is already compiled.
        if let Some(node) = self.compiled.get(&node_id) {
            return Ok(node.clone());
        }

        let node = self.graph.get_node(node_id).ok_or(())?;
        let eval_node = match node.node_type().evaluation {
            EvaluationStrategy::Compile(compile_node_fn) => {
                compile_node_fn(self, node)?
            },
        };

        self.compiled.insert(node_id, eval_node.clone());
        Ok(eval_node)
    }
}

pub trait CompileNode {
    fn compile_node(ctx: &mut CompilationContext, node: &NodeBlueprint) -> Result<NodeCell, ()>;
}

/// A function that compiles a node.
pub type CompileNodeFn = fn(&mut CompilationContext, node: &NodeBlueprint) -> Result<NodeCell, ()>;

/// An evaluatable node in the node graph.
pub trait Eval: GetOutputRef {
    /// Evaluate the node.
    fn eval(&mut self, ctx: &EvalContext) -> Result<(), ()>;
}

/// A node that can be evaluated and has outputs.
pub trait Node: Eval + GetOutputRef {}

pub trait GetOutputRef {
    /// Get a reference to an output.
    fn get_output_ref(&self, output_id: OutputId) -> Option<&dyn Any>;
}