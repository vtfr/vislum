use std::{any::Any, cell::{Ref, RefCell}, collections::{HashMap}, rc::Rc};

use crate::{node::{GraphBlueprint, InputBlueprint, InputId, NodeBlueprint as NodeBlueprint, NodeId, OutputId}, node_type::{AssignmentTypes, EvaluationStrategy, InputCardinality}, prelude::{InputDefinition, OutputDefinition}, value::{TaggedValue, Value}};

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
        node: NodeRef,
        output_id: OutputId,
    }
}

impl<T> Single<T> 
where 
    T: Value,
{
    pub fn eval(&self, ctx: &EvalContext) -> Result<T, EvalError> {
        match self {
            Single::Constant(value) => Ok(value.clone()),
            Single::Connection { node, output_id } => {
                node.eval(ctx)?;
                
                match node.get_output(*output_id) {
                    Some(output) => {
                        match  T::try_from(output) {
                            Ok(output) => Ok(output),
                            Err(_) => {
                                // This should never happen. 
                                //
                                // We validated the output type when compiling the node.
                                unreachable!()
                            }
                        }
                    },
                    None => Err(EvalError::NoOutput {
                        node_id: node.node_id,
                        output_id: *output_id,
                    }),
                }
            },
        }
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

pub trait GetOutputValue {
    fn get_output_value(&self) -> Option<TaggedValue>;
}

pub struct Output<T> {
    pub value: Option<T>,
}

impl<T> Output<T>
where 
    T: Value,
{
    pub fn set(&mut self, value: T) {
        self.value = Some(value);
    }
}


impl<T> Default for Output<T> {
    fn default() -> Self {
        Self {
            value: None,
        }
    }
}

impl<T> GetOutputValue for Output<T>
where 
    T: Value,
{
    fn get_output_value(&self) -> Option<TaggedValue> {
        self.value.clone().map(|value| value.into())
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
pub struct NodeRef {
    node_id: NodeId,
    node: Rc<RefCell<dyn Node>>,
}

impl NodeRef {
    pub fn new(node_id: NodeId, node: impl Node + 'static) -> Self {
        Self {
            node_id,
            node: Rc::new(RefCell::new(node)),
        }
    }

    pub fn eval(&self, ctx: &EvalContext) -> Result<(), EvalError> {
        self.node.borrow_mut().eval(ctx)
    }

    pub fn get_output(&self, output_id: OutputId) -> Option<TaggedValue> {
        let borrow = self.node.borrow();
        borrow.get_output(output_id)
    }
}

/// The context for compiling a graph.
pub struct CompilationContext<'a> {
    pub graph: &'a GraphBlueprint,
    pub compiled: HashMap<NodeId, NodeRef>,
}

impl<'a> CompilationContext<'a> {
    pub fn new(graph: &'a GraphBlueprint) -> Self {
        Self {
            graph,
            compiled: HashMap::new(),
        }
    }

    pub fn compile_node(&mut self, node_id: NodeId) -> Result<NodeRef, ()> {
        // Check if the node is already compiled.
        if let Some(node) = self.compiled.get(&node_id) {
            return Ok(node.clone());
        }

        let node = self.graph.get_node(node_id).ok_or(())?;
        let eval_node = match node.node_type().evaluation {
            EvaluationStrategy::Compile(compile_node_fn) => {
                compile_node_fn(self, node_id, node)?
            },
        };

        self.compiled.insert(node_id, eval_node.clone());
        Ok(eval_node)
    }
}

pub trait CompileNode {
    fn compile_node(ctx: &mut CompilationContext, node_id: NodeId, node: &NodeBlueprint) -> Result<NodeRef, ()>;
}

/// A function that compiles a node.
pub type CompileNodeFn = fn(&mut CompilationContext, node_id: NodeId, node: &NodeBlueprint) -> Result<NodeRef, ()>;

/// An evaluatable node in the node graph.
pub trait Eval: GetOutput {
    /// Evaluate the node.
    fn eval(&mut self, ctx: &EvalContext) -> Result<(), EvalError>;
}

/// A node that can be evaluated and has outputs.
pub trait Node: Eval + GetOutput {}

pub trait GetOutput {
    /// Get an output out of the node.
    fn get_output(&self, output_id: OutputId) -> Option<TaggedValue>;
}

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("The node {node_id:?} has no output with the given id {output_id}.")]
    NoOutput {
        node_id: NodeId,
        output_id: OutputId,
    },
}