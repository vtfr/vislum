use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{
    compile::{CompilationContext, CompileInput, GetInputDefinition, GetOutputDefinition},
    node::{InputBlueprint, InputId, NodeBlueprint, NodeId, OutputId},
    node_type::{AssignmentTypes, InputCardinality},
    prelude::{InputDefinition, OutputDefinition},
    value::{TaggedValue, Value},
};

pub struct EvalConnection {
    pub node: NodeRef,
    pub output_id: OutputId,
}

impl EvalConnection {
    pub fn eval<T>(&self, ctx: &EvalContext) -> Result<T, EvalError>
    where
        T: Value,
    {
        self.node.eval(ctx)?;

        let output = self.node.get_output(self.output_id);
        match output {
            Some(output) => match T::try_from(output) {
                Ok(output) => Ok(output),
                Err(_) => unreachable!(),
            },
            None => Err(EvalError::NoOutput {
                node_id: self.node.node_id,
                output_id: self.output_id,
            }),
        }
    }
}

/// An input that accepts a single value.
pub enum Single<T> {
    Constant(T),
    Connection(EvalConnection),
}

impl<T> Single<T>
where
    T: Value,
{
    pub fn eval(&self, ctx: &EvalContext) -> Result<T, EvalError> {
        match self {
            Single::Constant(value) => Ok(value.clone()),
            Single::Connection(connection) => connection.eval::<T>(ctx),
        }
    }
}

impl<T> CompileInput for Single<T>
where
    T: Value,
{
    fn compile_input(
        ctx: &mut CompilationContext,
        node: &NodeBlueprint,
        input_id: InputId,
    ) -> Result<Self, ()> {
        match node.get_input(input_id) {
            Ok(InputBlueprint::Constant(value)) => {
                Ok(Single::Constant(T::try_from(value.clone()).unwrap()))
            }
            Ok(InputBlueprint::Connection(connection)) => {
                let node = ctx.compile_node(connection.node_id)?;

                Ok(Single::Connection(EvalConnection {
                    node,
                    output_id: connection.output_id,
                }))
            }
            _ => Err(()),
        }
    }
}

impl<T> GetInputDefinition for Single<T>
where
    T: Value,
{
    fn get_input_definition(
        name: impl Into<String>,
        assignment_types: AssignmentTypes,
    ) -> InputDefinition {
        InputDefinition::new(name, &T::INFO, InputCardinality::Single, assignment_types)
    }
}

/// An input that accepts multiple connections.
pub struct Multiple<T> {
    pub values: Vec<EvalConnection>,
    pub phantom: PhantomData<T>,
}

impl<T> Multiple<T>
where
    T: Value,
{
    pub fn eval(&self, ctx: &EvalContext) -> Result<Vec<T>, EvalError> {
        self.values
            .iter()
            .map(|connection| connection.eval::<T>(ctx))
            .collect()
    }
}

impl<T> CompileInput for Multiple<T>
where
    T: Value,
{
    fn compile_input(
        ctx: &mut CompilationContext,
        node: &NodeBlueprint,
        input_id: InputId,
    ) -> Result<Self, ()> {
        let connection_blueprints = match node.get_input(input_id) {
            Ok(InputBlueprint::Connection(connection)) => {
                vec![*connection]
            }
            Ok(InputBlueprint::ConnectionVec(connections)) => connections.clone(),
            _ => return Err(()),
        };

        let mut connections = Vec::with_capacity(connection_blueprints.len());
        for blueprint in connection_blueprints {
            let node = ctx.compile_node(blueprint.node_id)?;

            connections.push(EvalConnection {
                node,
                output_id: blueprint.output_id,
            });
        }

        Ok(Multiple {
            values: connections,
            phantom: PhantomData,
        })
    }
}

impl<T> GetInputDefinition for Multiple<T>
where
    T: Value,
{
    fn get_input_definition(name: impl Into<String>, _: AssignmentTypes) -> InputDefinition {
        InputDefinition::new(
            name,
            &T::INFO,
            InputCardinality::Multiple,
            AssignmentTypes::CONNECTION,
        )
    }
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
        Self { value: None }
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
